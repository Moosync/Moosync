use std::{
    collections::HashMap,
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    sync::Arc,
    thread,
};

use futures::{
    channel::mpsc::{channel, Receiver, Sender, UnboundedReceiver, UnboundedSender},
    executor::block_on,
    SinkExt, StreamExt,
};

use interprocess::local_socket::tokio::Stream as LocalSocketStream;
use serde_json::Value;
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    join, select,
    sync::Mutex,
};
use types::errors::errors::{MoosyncError, Result};

pub type CommandSender = Sender<Result<Value>>;

pub struct SocketHandler {
    read_conn: Arc<Mutex<ReadHalf<LocalSocketStream>>>,
    write_conn: Arc<Mutex<WriteHalf<LocalSocketStream>>>,
    tx_ext_command: Arc<Mutex<UnboundedSender<(Value, Sender<Vec<u8>>)>>>,
    rx_main_command: Arc<Mutex<UnboundedReceiver<(Sender<Result<Value>>, Value)>>>,
    reply_map: Arc<Mutex<HashMap<String, CommandSender>>>,
}

impl<'a> SocketHandler {
    pub fn new(
        conn: LocalSocketStream,
        rx_main_command: UnboundedReceiver<(Sender<Result<Value>>, Value)>,
        tx_ext_command: UnboundedSender<(Value, Sender<Vec<u8>>)>,
    ) -> SocketHandler {
        let (read_conn, write_conn) = split(conn);
        SocketHandler {
            read_conn: Arc::new(Mutex::new(read_conn)),
            write_conn: Arc::new(Mutex::new(write_conn)),
            tx_ext_command: Arc::new(Mutex::new(tx_ext_command)),
            rx_main_command: Arc::new(Mutex::new(rx_main_command)),
            reply_map: Default::default(),
        }
    }

    async fn write_command(&self, mut tx_reply: Sender<Result<Value>>, value: &mut Value) {
        let channel = uuid::Uuid::new_v4().to_string();
        if let Some(value) = value.as_object_mut() {
            value.insert("channel".to_string(), Value::String(channel.clone()));
            match serde_json::to_vec(value) {
                Ok(bytes) => {
                    let mut reply_map = self.reply_map.lock().await;
                    reply_map.insert(channel, tx_reply);
                    Self::write_data(self.write_conn.clone(), bytes).await
                }
                Err(e) => tx_reply.send(Err(e.into())).await.unwrap(),
            }
        }
    }

    async fn read_fixed_buf(&self) -> Result<(Vec<u8>, usize)> {
        let mut buf = [0u8; 1024];

        let mut conn = self.read_conn.lock().await;

        let res = conn.read(&mut buf).await;

        if let Err(e) = res {
            if e.kind() == ErrorKind::WouldBlock {
                return Ok((vec![], 0));
            }
            println!("{:?}", e);
            return Err(MoosyncError::String("Failed to read from socket".into()));
        }

        let n = res.unwrap();
        Ok((buf[..n].to_vec(), n))
    }

    fn read_lines(&self, buf: &[u8], old_buf: &[u8]) -> (Vec<Vec<u8>>, Vec<u8>) {
        let mut reader = BufReader::new(buf);

        let mut lines = vec![];
        let mut remaining = vec![];

        let i = 0;

        loop {
            let mut parsed_buf = vec![];
            let read = reader.read_until(b'\n', &mut parsed_buf).unwrap();
            if read == 0 {
                break;
            }

            if i == 0 && !old_buf.is_empty() {
                parsed_buf = [old_buf, &parsed_buf].concat();
            }

            if !parsed_buf.ends_with(b"\n") {
                remaining = parsed_buf;
                break;
            }

            lines.push(parsed_buf);
        }

        (lines, remaining)
    }

    async fn send_reply(&self, data: &Value) -> bool {
        if let Some(channel) = data.get("channel") {
            let channel = channel.as_str().unwrap();
            let reply_map = self.reply_map.lock().await;
            let reply = reply_map.get(channel);
            if let Some(reply) = reply {
                reply
                    .clone()
                    .send(Ok(data.get("data").unwrap_or(&Value::Null).clone()))
                    .await
                    .unwrap();
                return true;
            }
        }

        false
    }

    pub async fn write_data(conn: Arc<Mutex<WriteHalf<LocalSocketStream>>>, mut data: Vec<u8>) {
        let mut conn = conn.lock().await;
        data.push(b'\n');
        conn.write_all(&data).await.unwrap();
        conn.flush().await.unwrap();
    }

    pub async fn handle_main_command(&self) {
        loop {
            let mut rx_main_command = self.rx_main_command.lock().await;
            let main_command = rx_main_command.next().await;
            if let Some((tx_reply, mut value)) = main_command {
                self.write_command(tx_reply, &mut value).await;
            }
        }
    }

    pub async fn handle_connection(&self) {
        let mut old_buf = vec![];
        loop {
            let ext_resp = self.read_fixed_buf().await;

            if ext_resp.is_err() {
                println!("Failed to read from socket {}", ext_resp.unwrap_err());
                break;
            }

            let (buf, n) = ext_resp.unwrap();
            if n == 0 {
                continue;
            }

            let (lines, remaining) = self.read_lines(&buf, &old_buf);
            old_buf = remaining;

            for line in lines {
                let parsed: std::result::Result<Value, serde_json::Error> =
                    serde_json::from_slice(&line);

                match parsed {
                    Ok(data) => {
                        // TODO: Validate request object
                        if !self.send_reply(&data).await {
                            let (tx, mut rx) = channel(1);
                            let conn = self.write_conn.clone();

                            tokio::spawn(async move {
                                let conn = conn.clone();
                                let res = rx.next().await.unwrap();
                                Self::write_data(conn, res).await;
                            });

                            let tx_ext_command = self.tx_ext_command.lock().await;
                            tx_ext_command.clone().send((data, tx)).await.unwrap();
                        }
                    }
                    Err(e) => {
                        println!("Failed to parsed response as json {:?}", e);
                    }
                }
            }
        }
    }
}
