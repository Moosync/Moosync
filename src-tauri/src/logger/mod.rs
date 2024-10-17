use core::str;
use std::{fs, io::Write, sync::Arc, sync::Mutex};

use tauri::{AppHandle, Manager, State};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use types::errors::Result;

pub struct Logger {
    file_appender: Arc<Mutex<RollingFileAppender>>,
}

impl Logger {
    pub fn new(app_handle: AppHandle) -> Self {
        let log_path = app_handle.path().app_log_dir().unwrap();
        if !log_path.exists() {
            fs::create_dir_all(log_path.clone()).unwrap();
        }

        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_path, "moosync_renderer");
        Self {
            file_appender: Arc::new(Mutex::new(file_appender)),
        }
    }

    pub fn renderer_write(&self, data: Vec<u8>) -> Result<()> {
        let mut file_appender = self.file_appender.lock().unwrap();
        file_appender.write_all(&data)?;
        drop(file_appender);

        println!("{}", str::from_utf8(&data)?);

        Ok(())
    }
}

pub fn get_logger_state(app: AppHandle) -> Logger {
    Logger::new(app)
}

#[tauri::command(async)]
pub fn renderer_write(logger: State<Logger>, data: Vec<u8>) -> Result<()> {
    logger.renderer_write(data)
}
