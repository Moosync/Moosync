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
use std::io::{self, Write};

use tracing_subscriber::fmt::MakeWriter;
use wasm_bindgen_futures::spawn_local;

use crate::{console_debug, console_error, console_info, console_trace, console_warn};

use super::invoke::renderer_write;

#[derive(Clone, Copy, Debug, Default)]
pub struct MakeConsoleWriter {
    log_file: bool,
}

impl MakeConsoleWriter {
    pub fn new_log_file() -> Self {
        Self { log_file: true }
    }
}

impl<'a> MakeWriter<'a> for MakeConsoleWriter {
    type Writer = ConsoleWriter;

    #[tracing::instrument(level = "trace", skip(self))]
    fn make_writer(&'a self) -> Self::Writer {
        ConsoleWriter {
            level: tracing::Level::DEBUG,
            data: vec![],
            log_file: self.log_file,
        }
    }

    #[tracing::instrument(level = "trace", skip(self, meta))]
    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        ConsoleWriter {
            level: *meta.level(),
            data: vec![],
            log_file: self.log_file,
        }
    }
}

pub struct ConsoleWriter {
    level: tracing::Level,
    data: Vec<u8>,
    log_file: bool,
}

impl Write for ConsoleWriter {
    #[tracing::instrument(level = "trace", skip(self, buf))]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.write(buf)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn flush(&mut self) -> io::Result<()> {
        if !self.log_file {
            let parsed = str::from_utf8(&self.data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            match self.level {
                tracing::Level::DEBUG => {
                    console_debug!("{}", parsed);
                }
                tracing::Level::ERROR => {
                    console_error!("{}", parsed);
                }
                tracing::Level::INFO => {
                    console_info!("{}", parsed);
                }
                tracing::Level::TRACE => {
                    console_trace!("{}", parsed);
                }
                tracing::Level::WARN => {
                    console_warn!("{}", parsed);
                }
            };
        } else {
            let data = self.data.clone();
            spawn_local(async move {
                if let Err(e) = renderer_write(data).await {
                    console_error!("Failed to write log: {:?}", e);
                }
            });
        }

        Ok(())
    }
}

impl Drop for ConsoleWriter {
    #[tracing::instrument(level = "trace", skip(self))]
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
