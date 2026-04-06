use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use turingosv3::kernel::File as TuringFile;

#[derive(Serialize, Deserialize)]
pub struct WalRecord {
    pub id: String,
    pub author: String,
    pub payload: String,
    pub citations: Vec<String>,
    pub stake: u64,
    pub intrinsic_reward: f64,
    pub price: f64,
}

impl From<&TuringFile> for WalRecord {
    fn from(tf: &TuringFile) -> Self {
        WalRecord {
            id: tf.id.clone(),
            author: tf.author.clone(),
            payload: tf.payload.clone(),
            citations: tf.citations.clone(),
            stake: tf.stake,
            intrinsic_reward: tf.intrinsic_reward,
            price: tf.price,
        }
    }
}

impl Into<TuringFile> for WalRecord {
    fn into(self) -> TuringFile {
        TuringFile {
            id: self.id,
            author: self.author,
            payload: self.payload,
            citations: self.citations,
            stake: self.stake,
            intrinsic_reward: self.intrinsic_reward,
            price: self.price,
            created_at: 0,
            completion_tokens: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TapeDelta {
    pub files: Vec<TuringFile>,
}

#[derive(Clone)]
pub struct WalSentinel {
    tx: UnboundedSender<TapeDelta>,
}

impl WalSentinel {
    pub fn new(wal_path: String) -> Self {
        let (tx, mut rx) = unbounded_channel::<TapeDelta>();

        tokio::spawn(async move {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&wal_path)
                .await
                .expect("Failed to open WAL file");

            while let Some(delta) = rx.recv().await {
                for turing_file in delta.files {
                    let record: WalRecord = (&turing_file).into();
                    if let Ok(mut json) = serde_json::to_string(&record) {
                        json.push('\n');
                        if let Err(e) = file.write_all(json.as_bytes()).await {
                            log::error!("WAL write error: {}", e);
                        }
                    }
                }
                let _ = file.flush().await;
            }
        });

        WalSentinel { tx }
    }

    pub fn record_delta(&self, delta: TapeDelta) {
        let _ = self.tx.send(delta);
    }
}

pub async fn recover_tape(wal_path: &str) -> Vec<TuringFile> {
    let mut files = Vec::new();
    if let Ok(file) = File::open(wal_path).await {
        let mut reader = BufReader::new(file).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(record) = serde_json::from_str::<WalRecord>(&line) {
                files.push(record.into());
            }
        }
    }
    files
}
