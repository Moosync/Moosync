use std::{
    collections::HashMap,
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    sync::mpsc::{Receiver, Sender},
};

use interprocess::local_socket::{traits::Stream, Stream as LocalSocketStream};
use serde_json::Value;
use tauri::AppHandle;
use types::errors::errors::{MoosyncError, Result};

use crate::request_handler::ReplyHandler;

pub type CommandSender = Sender<Result<Value>>;

pub struct SocketHandler<'a> {
    conn: LocalSocketStream,
    request_handler: ReplyHandler,
    rx_command: &'a Receiver<(Sender<Result<Value>>, Value)>,
    reply_map: HashMap<String, CommandSender>,
}

impl<'a> SocketHandler<'a> {
    pub fn new(
        conn: LocalSocketStream,
        app_handle: AppHandle,
        rx_command: &'a Receiver<(Sender<Result<Value>>, Value)>,
    ) -> SocketHandler<'a> {
        SocketHandler {
            conn,
            request_handler: ReplyHandler::new(app_handle),
            rx_command,
            reply_map: HashMap::new(),
        }
    }

    fn write_command(&mut self, tx_reply: Sender<Result<Value>>, value: &mut Value) {
        let channel = uuid::Uuid::new_v4().to_string();
        if let Some(value) = value.as_object_mut() {
            value.insert("channel".to_string(), Value::String(channel.clone()));
            match serde_json::to_vec(value) {
                Ok(bytes) => {
                    self.reply_map.insert(channel, tx_reply);
                    self.write_data(bytes)
                }
                Err(e) => tx_reply.send(Err(e.into())).unwrap(),
            }
        }
    }

    fn read_fixed_buf(&mut self) -> Result<(Vec<u8>, usize)> {
        let mut buf = [0u8; 1024];
        let res = self.conn.read(&mut buf);

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

    fn send_reply(&self, data: &Value) -> bool {
        if let Some(channel) = data.get("channel") {
            let channel = channel.as_str().unwrap();
            let reply = self.reply_map.get(channel);
            if let Some(reply) = reply {
                reply
                    .send(Ok(data.get("data").unwrap_or(&Value::Null).clone()))
                    .unwrap();
                return true;
            }
        }

        false
    }

    pub fn write_data(&mut self, mut data: Vec<u8>) {
        data.push(b'\n');
        self.conn.write_all(&data).unwrap();
        self.conn.flush().unwrap();
    }

    pub fn handle_connection(&mut self) {
        let mut old_buf = vec![];
        loop {
            if let Ok((tx_reply, mut value)) = self.rx_command.try_recv() {
                self.write_command(tx_reply, &mut value);
            }

            let res = self.read_fixed_buf();
            if res.is_err() {
                break;
            }

            let (buf, n) = res.unwrap();
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
                        if !self.send_reply(&data) {
                            let res = self.request_handler.handle_request(&data);
                            match res {
                                Err(e) => {
                                    println!("{:}", e);
                                }
                                Ok(res) => {
                                    self.write_data(res);
                                }
                            }
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
