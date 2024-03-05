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

struct YoutubePage {}

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

fn json_after(haystack: &str, left: &str) -> Option<String> {
    if let Some(pos) = haystack.find(left) {
        let haystack = &haystack[pos + left.len()..];
        cut_after_json(haystack)
    } else {
        None
    }
}

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

fn between(haystack: &str, left: &str, right: &str) -> Option<String> {
    if let Some(pos_left) = haystack.find(left) {
        let haystack = &haystack[pos_left + left.len()..];
        haystack.find(right).map(|pos_right| haystack[..pos_right].to_string())
    } else {
        None
    }
}
