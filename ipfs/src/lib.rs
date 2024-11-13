use std::{fs, path::Path};

use prost::Message;

const IPFS_CACHE_DIR: &str = "ipfs-cache";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("prost error: {0}")]
    Prost(#[from] prost::DecodeError),
}

type Result<T> = std::result::Result<T, Error>;

pub fn deserialize<T: Message + Default>(buf: &[u8]) -> std::result::Result<T, prost::DecodeError> {
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

    pub async fn get<T: prost::Message + Default>(&self, hash: &str, cache: bool) -> Result<T> {
        let bytes = self.get_bytes(hash, cache).await?;
        let data = deserialize(&bytes)?;
        Ok(data)
    }

    pub async fn get_bytes(&self, hash: &str, cache: bool) -> Result<Vec<u8>> {
        let cache_path = Path::new(IPFS_CACHE_DIR).join(hash);

        if cache {
            if cache_path.exists() {
                let cached_data = fs::read(&cache_path)?;
                return Ok(cached_data);
            }
        }

        let url = format!("{}{}", self.url, hash);
        let res = self.client.get(&url).send().await?;
        let bytes = res.bytes().await?;

        // Cache the result
        if cache {
            fs::create_dir_all(IPFS_CACHE_DIR)?;
            fs::write(&cache_path, &bytes)?;
        }

        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_import() {
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
        )]
        #[repr(i32)]
        pub enum ActionType {
            DefaultActionType = 0,
            AddEdit = 1,
            ImportSpace = 2,
            AddSubspace = 3,
            RemoveSubspace = 4,
            AddEditor = 5,
            RemoveEditor = 6,
            AddMember = 7,
            RemoveMember = 8,
        }
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
        )]
        #[repr(i32)]
        pub enum OpType {
            DefaultOpType = 0,
            SetTriple = 1,
            DeleteTriple = 2,
        }
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Edit {
            #[prost(enumeration = "ActionType", tag = "1")]
            pub r#type: i32,
            #[prost(string, tag = "2")]
            pub version: ::prost::alloc::string::String,
            #[prost(string, tag = "3")]
            pub id: ::prost::alloc::string::String,
            #[prost(string, tag = "4")]
            pub name: ::prost::alloc::string::String,
            #[prost(message, repeated, tag = "5")]
            pub ops: ::prost::alloc::vec::Vec<Op>,
            #[prost(string, repeated, tag = "6")]
            pub authors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        }

        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Op {
            #[prost(enumeration = "OpType", tag = "1")]
            pub r#type: i32,
            #[prost(message, optional, tag = "2")]
            pub triple: ::core::option::Option<Triple>,
        }
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Triple {
            #[prost(string, tag = "1")]
            pub entity: ::prost::alloc::string::String,
            #[prost(string, tag = "2")]
            pub attribute: ::prost::alloc::string::String,
            #[prost(message, optional, tag = "3")]
            pub value: ::core::option::Option<Value>,
        }
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
        )]
        #[repr(i32)]
        pub enum ValueType {
            DefaultValueType = 0,
            Text = 1,
            Number = 2,
            Entity = 3,
            Uri = 4,
            Checkbox = 5,
            Time = 6,
            GeoLocation = 7,
        }
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Value {
            #[prost(enumeration = "ValueType", tag = "1")]
            pub r#type: i32,
            #[prost(string, tag = "2")]
            pub value: ::prost::alloc::string::String,
        }

        let ipfs_hash = "bafkreibuhzyc5qs4y6pc2tk6qx3zf7mvicd7xxyuhyg75s5a53p6g44gwq";

        let client = super::IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");

        let edit = client.get::<Edit>(ipfs_hash, false).await.unwrap();
    }
}
