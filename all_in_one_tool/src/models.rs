use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    #[serde(with = "humantime_serde")]
    pub modified: std::time::SystemTime,
    #[serde(flatten)]
    pub specific: Option<SpecificMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "media_type")]
pub enum SpecificMetadata {
    Image {
        exif: BTreeMap<String, String>,
        png_chunks: Option<BTreeMap<String, String>>,
    },
    Audio {
        artist: Option<String>,
        album: Option<String>,
        title: Option<String>,
        year: Option<i32>,
        extended_tags: BTreeMap<String, Vec<String>>,
    },
    Video {
        artist: Option<String>,
        album: Option<String>,
        title: Option<String>,
        year: Option<i32>,
        extended_tags: BTreeMap<String, Vec<String>>,
    },
}
