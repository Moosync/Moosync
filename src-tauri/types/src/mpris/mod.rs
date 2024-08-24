use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MprisPlayerDetails {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "artistName")]
    pub artist_name: Option<String>,
    #[serde(rename = "albumName")]
    pub album_name: Option<String>,
    #[serde(rename = "albumArtist")]
    pub album_artist: Option<String>,
    pub genres: Option<Vec<String>>,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
}
