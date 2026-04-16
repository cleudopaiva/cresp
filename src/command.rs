use crate::{frame::RespFrame, store::Store};

use std::sync::{Arc, RwLock};

use tokio::io::AsyncWriteExt;

pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
}

impl Command {
    pub async fn execute<W>(
        self,
        store: &Arc<RwLock<Store>>,
        writer: &mut W,
    ) -> tokio::io::Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        match self {
            Command::Set { key, value } => {
                {
                    let mut store_guard = store.write().unwrap();
                    store_guard.write(key, value);
                }
                writer.write_all(b"Ok\n").await?;
            }
            Command::Get { key } => {
                let response;
                {
                    let store_guard = store.read().unwrap();
                    response = match store_guard.read(&key) {
                        Some(result) => result.to_string(),
                        _ => "Key does not exists\n".to_string(),
                    };
                }
                writer.write_all(response.as_bytes()).await?;
            }
        }
        Ok(())
    }
}

impl TryFrom<RespFrame> for Command {
    type Error = String;
    fn try_from(frame: RespFrame) -> Result<Self, Self::Error> {
        match frame {
            RespFrame::Array(elements) => {
                let mut frame_iter = elements.into_iter();
                let cmd_frame = frame_iter.next().ok_or("empty command array.")?;
                let cmd_name = cmd_frame
                    .as_string()
                    .ok_or("command name must be a string")?;
                match cmd_name.to_uppercase().as_str() {
                    "GET" => {
                        let key = frame_iter
                            .next()
                            .and_then(|frame| frame.as_string())
                            .ok_or("GET requires 1 argument (key)")?;
                        Ok(Command::Get { key })
                    }

                    "SET" => {
                        let key = frame_iter
                            .next()
                            .and_then(|frame| frame.as_string())
                            .ok_or("SET requires 2 arguments (key, value)")?;
                        let value = frame_iter
                            .next()
                            .and_then(|frame| frame.as_string())
                            .ok_or("SET requires 2 arguments (key, value)")?;

                        Ok(Command::Set { key, value })
                    }
                    _ => Err(format!("unknown command {}", cmd_name)),
                }
            }
            _ => Err("Expected array".into()),
        }
    }
}
