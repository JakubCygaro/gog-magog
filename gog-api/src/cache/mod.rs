use std::{collections::HashMap, io};

use thiserror::Error;
use tokio::io::AsyncReadExt;

type CacheResult<T> = Result<T, CacheError>;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("File at specified path `{path}` is already present in the cache")]
    AlreadyPresent { path: String },
    #[error("Io error")]
    IoError {
        #[from]
        source: io::Error,
    },
}

pub struct ResourceCache {
    resources: HashMap<String, Vec<u8>>,
}

impl ResourceCache {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
    /// Loads a resource from a file path and caches it
    pub async fn load(&mut self, path: &str) -> CacheResult<()> {
        if self.resources.contains_key(path) {
            return Err(CacheError::AlreadyPresent {
                path: path.to_owned(),
            });
        };
        let mut buf = vec![];
        let f = tokio::fs::File::open(path).await?;
        let mut reader = tokio::io::BufReader::new(f);
        reader.read_to_end(&mut buf).await?;
        self.resources.insert(path.to_owned(), buf);
        Ok(())
    }
    pub async fn get_or_load(&mut self, path: &str) -> CacheResult<Vec<u8>> {
        match self.load(path).await {
            Err(CacheError::AlreadyPresent { path: _ }) | Ok(_) => {
                return Ok(self.resources.get(path).unwrap().to_owned())
            }
            Err(e) => Err(e),
        }
    }
    pub async fn get_or_load_ref(&mut self, path: &str) -> CacheResult<&Vec<u8>> {
        match self.load(path).await {
            Err(CacheError::AlreadyPresent { path: _ }) | Ok(_) => {
                return Ok(&self.resources.get(path).unwrap())
            }
            Err(e) => Err(e),
        }
    }
    pub fn get(&self, path: &str) -> Option<Vec<u8>> {
        self.resources.get(path).map(|s| s.to_owned())
    }
    pub fn get_ref(&self, path: &str) -> Option<&Vec<u8>> {
        self.resources.get(path)
    }
    pub fn drop_from_cache(&mut self, path: &str) -> Option<Vec<u8>> {
        self.resources.remove(path)
    }
    pub fn clear_cache(&mut self) {
        self.resources.clear();
    }
}
