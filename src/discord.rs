use serde::{Deserialize, Serialize};
use worker::console_log;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum InteractionCallback {
    #[serde(rename = "1")]
    Pong {},
}

// Work around until interger enum tags are implemented in serde
#[derive(Deserialize)]
#[serde(from = "RawInterectionCallback")]
pub struct InteractionCallbackWrapper(pub InteractionCallback);

#[derive(Deserialize)]
struct RawInterectionCallback {
    #[serde(rename = "type")]
    type_: i8,
    #[serde(flatten)]
    data: Option<serde_json::Value>,
}

impl From<RawInterectionCallback> for InteractionCallbackWrapper {
    fn from(value: RawInterectionCallback) -> Self {
        let type_ = value.type_.to_string();
        let callback: InteractionCallback = serde_json::from_value(serde_json::json!({
            "type": type_,
            "data": value.data
        }))
        .unwrap();
        Self(callback)
    }
}

const BASE_URL: &str = "https://discord.com/api/v10";

#[derive(Serialize)]
pub struct Message {
    pub content: String,
}

pub async fn send_message(token: &str, channel_id: &str, msg: &Message) {
    let url = format!("{BASE_URL}/channels/{channel_id}/messages");
    console_log!("URL: {url}");

    let body = serde_wasm_bindgen::to_value(&serde_json::to_string(&msg).unwrap()).unwrap();
    let mut headers = worker::Headers::new();
    headers
        .append("Authorization", &format!("Bot {token}"))
        .unwrap();
    headers.append("Content-Type", "application/json").unwrap();

    let mut request_config = worker::RequestInit::new();
    request_config.with_body(Some(body));
    request_config.with_method(worker::Method::Post);
    request_config.with_headers(headers);

    let request = worker::Request::new_with_init(&url, &request_config).unwrap();
    let mut response = worker::Fetch::Request(request).send().await.unwrap();

    let text = response.text().await.unwrap();
    console_log!(" DISCORD RESPONSE: {text}");
}
