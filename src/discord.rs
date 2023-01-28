use serde::Deserialize;

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
