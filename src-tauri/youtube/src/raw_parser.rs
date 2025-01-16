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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use serde::{Deserialize, Serialize};

use crate::types::Root;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub client: Client,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Client {
    pub client_version: String,
    pub gl: String,
    pub hl: String,
    pub client_name: String,
    pub utc_offset_minutes: i32,
}

#[tracing::instrument(level = "trace", skip(body))]
pub fn parse_body(body: &str) -> (Option<Root>, Option<String>, Context) {
    let json = json_after(body, "window[\"ytInitialData\"] = ")
        .or(json_after(body, "var ytInitialData = "));
    let api_key = between(body, "INNERTUBE_API_KEY\":\"", "\"").or(between(
        body,
        "innertubeApiKey\":\"",
        "\"",
    ));
    let client_version = between(body, "INNERTUBE_CONTEXT_CLIENT_VERSION\":\"", "\"").or(between(
        body,
        "innertube_context_client_version\":\"",
        "\"",
    ));

    let context = Context {
        client: Client {
            client_version: client_version.unwrap_or_default(),
            gl: "US".to_string(),
            hl: "en".to_string(),
            client_name: "WEB".to_string(),
            utc_offset_minutes: 0,
        },
    };

    // if let Some(gl) = options.gl {
    //     context.client.gl = Some(gl);
    // }
    // if let Some(hl) = options.hl {
    //     context.client.hl = Some(hl);
    // }
    // if let Some(utc_offset_minutes) = options.utc_offset_minutes {
    //     context.client.utc_offset_minutes = Some(utc_offset_minutes);
    // }

    (
        json.map(|v| serde_json::from_str(v.as_str()).unwrap()),
        api_key,
        context,
    )
}

#[tracing::instrument(level = "trace", skip(haystack, left))]
fn json_after(haystack: &str, left: &str) -> Option<String> {
    if let Some(pos) = haystack.find(left) {
        let haystack = &haystack[pos + left.len()..];
        cut_after_json(haystack)
    } else {
        None
    }
}

#[tracing::instrument(level = "trace", skip(haystack))]
fn cut_after_json(haystack: &str) -> Option<String> {
    let mut brace_count = 0;
    let mut end_index = 0;

    for (idx, ch) in haystack.char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    end_index = idx + 1;
                    break;
                }
            }
            _ => continue,
        }
    }

    if end_index > 0 {
        Some(haystack[..end_index].to_string())
    } else {
        None
    }
}

#[tracing::instrument(level = "trace", skip(haystack, left, right))]
fn between(haystack: &str, left: &str, right: &str) -> Option<String> {
    if let Some(pos_left) = haystack.find(left) {
        let haystack = &haystack[pos_left + left.len()..];
        haystack
            .find(right)
            .map(|pos_right| haystack[..pos_right].to_string())
    } else {
        None
    }
}
