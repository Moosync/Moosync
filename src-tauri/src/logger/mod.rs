use core::str;
use std::{fs, io::Write, sync::Arc, sync::Mutex};

use tauri::{AppHandle, Manager, State};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use types::errors::Result;

use regex::Regex;
use std::sync::Once;

static mut REGEX: Option<Regex> = None;
static INIT: Once = Once::new();

fn get_regex() -> &'static Regex {
    unsafe {
        INIT.call_once(|| {
            REGEX = Some(Regex::new(r"(?:\x1B[@-_]|[\x80-\x9F])[0-?]*[ -/]*[@-~]").unwrap());
        });
        REGEX.as_ref().unwrap()
    }
}

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

        let parsed = str::from_utf8(&data)?;
        let re = get_regex();
        let parsed_stripped = re.replace_all(parsed, "");

        file_appender.write_all(parsed_stripped.as_bytes())?;
        drop(file_appender);

        println!("{}", parsed);

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
