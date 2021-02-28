use serde::{Deserialize, Serialize};
use type_uuid::*;
use std::sync::Arc;

#[derive(TypeUuid, Serialize, Deserialize, Clone)]
#[uuid = "197bfd7a-3df9-4440-86f0-8e10756c714e"]
pub struct FontAssetData {
    pub data_hash: u64,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub scale: f32,
}

pub struct FontAssetInner {
    pub data_hash: u64,
    pub font: fontdue::Font
}

#[derive(TypeUuid, Clone)]
#[uuid = "398689ef-4bf1-42ad-8fc4-5080c1b8293a"]
pub struct FontAsset {
    pub inner: Arc<FontAssetInner>
}
