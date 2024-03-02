use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    IMAGE = 0,
    VIDEO = 1,
    VIDEO_LOOPING = 2,
    VIDEO_LOOPING_RANDOM = 3,
    GIF = 4,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvazArtist {
    pub uri: String,
    pub name: String,
    pub avatar: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Canvaz {
    pub id: String,
    pub url: String,
    pub file_id: String,
    pub type_: Type,
    pub entity_uri: String,
    pub artist: CanvazArtist,
    pub explicit: bool,
    pub uploaded_by: String,
    pub etag: String,
    pub canvas_uri: String,
    pub storylines_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvazResponse {
    pub canvases: Vec<Canvaz>,
    pub ttl_in_seconds: i64,
}
