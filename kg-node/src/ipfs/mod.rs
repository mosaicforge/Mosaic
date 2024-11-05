use std::{fs, path::Path};

use futures::{stream, StreamExt, TryStreamExt};
use prost::Message;

use crate::grc20;

const IPFS_CACHE_DIR: &str = "ipfs-cache";

pub fn deserialize<T: Message + Default>(buf: &[u8]) -> Result<T, prost::DecodeError> {
    T::decode(buf)
}

pub struct IpfsClient {
    url: String,
    client: reqwest::Client,
}

impl IpfsClient {
    pub fn from_url(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, hash: &str) -> anyhow::Result<Vec<u8>> {
        let cache_path = Path::new(IPFS_CACHE_DIR).join(hash);

        if cache_path.exists() {
            let cached_data = fs::read(&cache_path)?;
            return Ok(cached_data);
        }

        let url = format!("{}{}", self.url, hash);
        let res = self.client.get(&url).send().await?;
        let bytes = res.bytes().await?;

        // Cache the result
        fs::create_dir_all(IPFS_CACHE_DIR)?;
        fs::write(&cache_path, &bytes)?;

        Ok(bytes.to_vec())
    }

    pub async fn import_blob(&self, hash: &str) -> anyhow::Result<Vec<grc20::Op>> {
        let buf = self.get(&hash).await?;
        let import = deserialize::<grc20::Import>(&buf)?;

        let mut edits = stream::iter(import.edits)
            .map(|edit| async move {
                let hash = edit.replace("ipfs://", "");
                let bytes = self.get(&hash).await?;
                anyhow::Ok(deserialize::<grc20::ImportEdit>(&bytes)?)
            })
            .buffer_unordered(10)
            .try_collect::<Vec<_>>()
            .await?;

        edits.sort_by_key(|edit| edit.block_number.clone());

        Ok(edits.into_iter().flat_map(|edit| edit.ops).collect())
    }
}
