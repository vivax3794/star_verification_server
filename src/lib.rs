use worker::*;

mod discord;
mod stars;
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    let router = Router::new();

    router
        .get("/", |_, _| Response::ok("SYSTEM ONLINE"))
        .post_async("/api/post_stars", post_stars)
        .post_async("/discord", handle_interaction)
        .run(req, env)
        .await
}

async fn post_stars(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // parse the star data
    let data: stars::StarData = match req.json().await {
        Err(err) => return Response::error(format!("{err:?}"), 400),
        Ok(data) => data,
    };
    console_log!("lenght of stars: {}", data.stars.len());

    // store the stars in KV
    let star_store = ctx.kv("stars").unwrap();
    let id_to_use = uuid::Uuid::new_v4();
    star_store
        .put(&id_to_use.to_string(), data)
        .unwrap()
        .expiration_ttl(60 * 60)
        .execute()
        .await
        .unwrap();
    console_log!("Saved using: {id_to_use}");

    Response::ok("Stars Saved")
}

async fn handle_interaction(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let raw_data = req.text().await.unwrap();

    // Verify signature
    let public_key = ctx.secret("DISCORD_PUBLIC_KEY").unwrap();
    let headers = req.headers();
    console_log!("{headers:?}");
    let signature = if let Ok(Some(sig)) = headers.get("X-Signature-Ed25519") {
        sig
    } else {
        return Response::error("INVALID SIGNATURE", 401);
    };
    let timestamp = if let Ok(Some(tim)) = headers.get("X-Signature-Timestamp") {
        tim
    } else {
        return Response::error("INVALID SIGNATURE", 401);
    };

    console_log!("VERIFYING");
    let public_key =
        ed25519_dalek::PublicKey::from_bytes(&hex::decode(public_key.to_string()).unwrap())
            .unwrap();
    let signature = ed25519_dalek::Signature::from_bytes(&hex::decode(signature).unwrap()).unwrap();

    if let Err(err) =
        public_key.verify_strict(format!("{timestamp}{raw_data}").as_bytes(), &signature)
    {
        console_log!("{err:?}");
        return Response::error("INVALID SIGNATURE", 401);
    }

    // Parse data
    let data: discord::InteractionCallbackWrapper = match serde_json::from_str(&raw_data) {
        Err(err) => return Response::error(format!("{err:#?}"), 400),
        Ok(data) => data,
    };
    let data = data.0;

    console_log!("INTERACTION: {data:#?}");
    match data {
        discord::InteractionCallback::Pong {} => Response::from_json(&serde_json::json!({
            "type": 1
        })),
    }
}
