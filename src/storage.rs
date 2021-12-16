use std::{time::SystemTime, marker::PhantomData};

use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt};


pub struct JsonStorage<'a, T> where T: Serialize + Deserialize<'a>  {
    filename: String,

    // lifetime and type parameter are actually used!
    _data: &'a PhantomData<T>,
    
    // to bypass lifetime checks (:D)
    contents: Box<&'a str>,
    content: String 
}

impl<'a, T> JsonStorage<'a, T> where T: 'a + Serialize + Deserialize<'a> {
    pub async fn new(filename: String) -> JsonStorage<'a, T> {
        JsonStorage { filename, _data: &PhantomData, contents: Box::new(""), content: "".to_string() }
    }

    pub async fn store(self, data: &T) -> Result<(), ()> {
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

    unsafe fn u8_vec_to_str(&self, v: Vec<u8>) -> String {
        return std::str::from_utf8_unchecked(v.as_slice()).to_string()
    }

    async unsafe fn read_data(&'a mut self) -> Option<T> {
        let contents = tokio::fs::read(&self.filename).await;

        if contents.is_err() {
            return None
        }

        let contents: Vec<u8> = contents.unwrap();

        if contents.len() == 0 {
            return None
        }
        else {
            self.content = self.u8_vec_to_str(contents);
            self.contents = Box::new(&self.content);

            if *self.contents == "" {
                return None;
            }

            let deserialized = serde_json::from_str(*self.contents);
            if let Ok(data) = deserialized {
                data
            } else {
                None
            }
        }


    }

    pub async fn get_stored_data(&'a mut self) -> Option<T> {
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
                    let data = unsafe { self.read_data().await };
                    if data.is_none() {
                        None
                    }
                    else {
                        Some(data.unwrap())
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