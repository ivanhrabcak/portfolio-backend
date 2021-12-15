use std::time::SystemTime;

use tokio::{fs::File, io::AsyncWriteExt};

use crate::github::Repository;


pub struct TemporaryJsonStorage {
    filename: String,
    repositories: Vec<Repository>
}

impl TemporaryJsonStorage {
    pub async fn new(filename: String) -> Self {
        let contents = tokio::fs::read(&filename).await.unwrap_or(Vec::new());

        if contents.len() == 0 {
            let file = File::create(&filename).await;

            // if the file cannot be created we want to panic - we have nothing else to do
            file.unwrap();

            TemporaryJsonStorage { filename, repositories: Vec::new() }
        }
        else {
            let contents = std::str::from_utf8(contents.as_slice()).unwrap_or("");

            if contents == "" {
                TemporaryJsonStorage { filename, repositories: Vec::new() }
            }
            else {
                match serde_json::from_str(contents) {
                    Ok(repositories) => TemporaryJsonStorage { filename, repositories },
                    Err(_) => TemporaryJsonStorage { filename, repositories: Vec::new() }
                }
            }
        }
    }

    pub async fn store(self, data: &Vec<Repository>) -> Result<(), ()> {
        let serialized = serde_json::to_string(data);

        
        if serialized.is_err() {
            return Err(());
        }

        let serialized = serialized.unwrap();

        let file = File::create(&self.filename).await;

        if file.is_err() {
            return Err(());
        }

        let mut file = file.unwrap();
        
        let write_result = file.write_all(serialized.as_bytes()).await;

        if write_result.is_err() {
            return Err(());
        }

        Ok(())
    } 

    pub async fn get_repositories(self) -> Option<Vec<Repository>> {
        let file = File::open(&self.filename).await;

        if file.is_err() {
            return None;
        }

        let file = file.unwrap();
        
        let file_metadata = file.metadata().await;

        if file_metadata.is_err() {
            return None;
        }

        let file_metadata = file_metadata.unwrap();
        match file_metadata.modified() {
            Ok(modified) => {
                // if this fails then we are f'd, we want to unwrap
                let duration_since_modified = SystemTime::now().duration_since(modified).unwrap();

                if duration_since_modified.as_secs() < 60 * 60 { // each hour
                    if self.repositories.len() == 0 {
                        None
                    }
                    else {
                        Some(self.repositories.to_vec())
                    }
                }
                else {
                    None
                }
            },
            Err(_) => None
        }
    }
}