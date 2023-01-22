use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    #[serde(rename = "currentStar")]
    pub color: i8,
}

#[derive(Deserialize, Serialize)]
pub struct StarData {
    pub jwt: String,
    pub stars: Vec<Star>,
}
