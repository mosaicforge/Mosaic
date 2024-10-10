use futures::{stream, StreamExt, TryStreamExt};
use kg_node::grc20;
use kg_node::kg_client::{self, Entity};
use prost::Message;
use std::fs;
use std::path::Path;

const IPFS_CACHE_DIR: &str = "ipfs-cache";

pub fn deserialize<T: Message + Default>(buf: &[u8]) -> Result<T, prost::DecodeError> {
    T::decode(buf)
}

struct IpfsClient {
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let kg_client = kg_client::KgClient::new("neo4j://localhost:7687", "", "").await?;
    kg_client.bootstrap().await?;

    let buf = include_bytes!("../bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy");
    // // let buf = include_bytes!("../bafkreih3oxxoenvhrcb747ib6rh7gpnho2rzopdljrtiyafoesyxnrhziq");
    let import = deserialize::<grc20::Import>(buf)?;
    // // println!("{:?}", import);

    let client = &IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");

    let mut edits = stream::iter(import.edits)
        .map(|edit| async move {
            let hash = edit.replace("ipfs://", "");
            let bytes = client.get(&hash).await?;
            anyhow::Ok(deserialize::<grc20::ImportEdit>(&bytes)?)
        })
        .buffer_unordered(10)
        .try_collect::<Vec<_>>()
        .await?;

    edits.sort_by_key(|edit| edit.block_number.clone());

    for edit in edits {
        // for op in edit.ops {
        //     kg_client.handle_op(op).await?;
        // }
        stream::iter(edit.ops)
            .for_each_concurrent(1, |op| async {
                kg_client.handle_op(op).await.expect("Failed to handle op");
            })
            .await;
        // break;
    }

    Ok(())
}
