// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use core::str;
use std::{fs, io::Write, sync::Arc, sync::Mutex};

use tauri::{AppHandle, Manager, State};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use types::errors::Result;
use types::errors::error_helpers;

use regex::Regex;
use std::sync::OnceLock;

static REGEX: OnceLock<Regex> = OnceLock::new();

fn get_regex() -> &'static Regex {
    REGEX.get_or_init(|| Regex::new(r"(?:\x1B[@-_]|[\x80-\x9F])[0-?]*[ -/]*[@-~]").unwrap())
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

        let parsed = str::from_utf8(&data).map_err(error_helpers::to_parse_error)?;
        let re = get_regex();
        let parsed_stripped = re.replace_all(parsed, "");

        file_appender
            .write_all(parsed_stripped.as_bytes())
            .map_err(error_helpers::to_file_system_error)?;
        drop(file_appender);

        println!("{parsed}");

        Ok(())
    }
}

pub fn get_logger_state(app: AppHandle) -> Logger {
    Logger::new(app)
}

#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn renderer_write(logger: State<Logger>, data: Vec<u8>) -> Result<()> {
    logger.renderer_write(data)
}
