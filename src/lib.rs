use worker::*;

mod database;
mod discord;
mod stars;
mod utils;

fn log_request(req: &Request) {
    console_log!("{} - [{}]", Date::now().to_string(), req.path(),);
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
    let mut kv = crate::database::CFWorkersKV::new(&ctx);
    kv.store_stars(data).await;

    // Send discord msg
    let discord_token = ctx.secret("DISCORD_BOT_TOKEN").unwrap().to_string();
    let discord_channel = ctx.var("DISCORD_STAR_CHANNEL").unwrap().to_string();

    let msg = crate::discord::Message {
        content: String::from("Some stars were posted, but I dont wanna show you!"),
    };

    crate::discord::send_message(&discord_token, &discord_channel, &msg).await;

    Response::ok("Stars Saved")
}

fn verify_signature(
    req: &mut Request,
    ctx: &RouteContext<()>,
    raw_data: &str,
) -> Option<Result<Response>> {
    // Get info
    let public_key = ctx.secret("DISCORD_PUBLIC_KEY").unwrap();
    let headers = req.headers();

    let signature = if let Ok(Some(sig)) = headers.get("X-Signature-Ed25519") {
        sig
    } else {
        return Some(Response::error("INVALID SIGNATURE", 401));
    };

    let timestamp = if let Ok(Some(tim)) = headers.get("X-Signature-Timestamp") {
        tim
    } else {
        return Some(Response::error("INVALID SIGNATURE", 401));
    };

    // Parse data
    let public_key =
        ed25519_dalek::PublicKey::from_bytes(&hex::decode(public_key.to_string()).unwrap())
            .unwrap();
    let signature = ed25519_dalek::Signature::from_bytes(&hex::decode(signature).unwrap()).unwrap();

    // Verify
    if let Err(err) =
        public_key.verify_strict(format!("{timestamp}{raw_data}").as_bytes(), &signature)
    {
        console_log!("{err:?}");
        return Some(Response::error("INVALID SIGNATURE", 401));
    }

    None
}

async fn handle_interaction(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let raw_data = req.text().await.unwrap();

    // Verify signature
    if let Some(resp) = verify_signature(&mut req, &ctx, &raw_data) {
        return resp;
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
