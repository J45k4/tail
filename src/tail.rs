use std::{path::{Path, PathBuf}, time::Duration, fs::Metadata};

use tokio::io::{AsyncReadExt};


pub struct Tail {
    messages_path: PathBuf,
    current_meta: Option<Metadata>,
    file: Option<tokio::fs::File>,
    runing: bool
}

impl Tail {
    pub async fn new<P>(messages_path: P) -> Tail
    where
        P: AsRef<Path>,
    {
        let messages_path = messages_path.as_ref().to_path_buf();

        if messages_path.exists() {
            let file = tokio::fs::File::open(&messages_path).await.unwrap();

            let metadata = file.metadata().await.unwrap();
            
            Tail {
                messages_path: messages_path,
                current_meta: Some(metadata),
                file: Some(file),
                runing: false
            }
        } else {
            Tail {
                messages_path: messages_path,
                current_meta: None,
                file: None,
                runing: false
            }
        }
    }

    pub fn start(&mut self) {
        self.runing = true;
    }

    pub fn stop(&mut self) {
        self.runing = false;
    }

    pub async fn get_lines(&mut self) -> Vec<String> {
        let mut buff = [0; 1024];

        loop {
            if !self.runing {
                tokio::time::sleep(Duration::from_millis(1000)).await;

                continue;
            }

            let file = match self.file {
                Some(ref mut file) => file,
                None => {
                    if self.messages_path.exists() {
                        let file = tokio::fs::File::open(&self.messages_path)
                            .await.unwrap();

                        self.current_meta = Some(file.metadata().await.unwrap());

                        self.file = Some(file);

                        continue;
                    } else {
                        tokio::time::sleep(Duration::from_millis(1000)).await;

                        continue;
                    }
                }
            };

            let read_bytes = file.read(&mut buff).await.unwrap();

            if read_bytes == 0 {
                if self.messages_path.exists() {
                    let new_metadata = tokio::fs::metadata(&self.messages_path)
                        .await.unwrap();

                    let current_created = match &self.current_meta {
                        Some(m) => m.created().unwrap(),
                        None => {
                            continue;
                        },
                    };

                    if current_created < new_metadata.created().unwrap() {
                        let file = tokio::fs::File::open(&self.messages_path)
                            .await.unwrap();

                        self.current_meta = Some(new_metadata);
                        self.file = Some(file);

                        continue;
                    }
                }

                tokio::time::sleep(Duration::from_millis(1000)).await;

                continue;
            }

            let buff = &buff[..read_bytes];

            let s = String::from_utf8_lossy(&buff[..read_bytes]);

            break s.lines().skip_while(|p| p.is_empty()).map(|s| s.to_string()).collect()
        }
    }
}