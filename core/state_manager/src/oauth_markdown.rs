use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq)]
pub enum OauthContentPart {
    Text(String),
    Image(PathBuf),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcessedOauthMarkdown {
    pub title: Option<String>,
    pub parts: Vec<OauthContentPart>,
}

#[tracing::instrument(level = "debug", skip_all)]
fn resolve_extension_image(dest_url: &str, extension_path: Option<&Path>) -> Option<PathBuf> {
    let ext_dir = extension_path?;
    if dest_url.contains("://") || dest_url.starts_with('/') || dest_url.starts_with('\\') {
        tracing::warn!("Ignoring non-relative or external image URL: {}", dest_url);
        return None;
    }

    let candidate = ext_dir.join(dest_url);
    let canonical_ext = match ext_dir.canonicalize() {
        Ok(path) => path,
        Err(err) => {
            tracing::error!(
                "Failed to canonicalize extension path {:?}: {:?}",
                ext_dir,
                err
            );
            return None;
        }
    };

    let canonical_img = match candidate.canonicalize() {
        Ok(path) => path,
        Err(err) => {
            tracing::warn!("Failed to resolve image asset {:?}: {:?}", candidate, err);
            return None;
        }
    };

    if canonical_img.starts_with(&canonical_ext) && canonical_img.is_file() {
        Some(canonical_img)
    } else {
        tracing::warn!(
            "Rejected image path outside extension directory: {:?}",
            canonical_img
        );
        None
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn flush_text_buffer(parts: &mut Vec<OauthContentPart>, buffer: &mut String) {
    let trimmed = buffer.trim();
    if !trimmed.is_empty() {
        parts.push(OauthContentPart::Text(trimmed.to_string()));
    }
    buffer.clear();
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_and_split_oauth_markdown(
    raw_markdown: &str,
    extension_path: Option<&Path>,
) -> ProcessedOauthMarkdown {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(raw_markdown, options);

    let mut title: Option<String> = None;
    let mut in_first_h1 = false;
    let mut title_buf = String::new();
    let mut in_image = false;
    let mut active_link_url: Option<String> = None;
    let mut list_stack: Vec<Option<u64>> = Vec::new();

    let mut parts: Vec<OauthContentPart> = Vec::new();
    let mut text_buf = String::new();

    for event in parser {
        if in_first_h1 {
            match event {
                Event::End(TagEnd::Heading(HeadingLevel::H1)) => {
                    in_first_h1 = false;
                    let trimmed = title_buf.trim();
                    if !trimmed.is_empty() {
                        title = Some(trimmed.to_string());
                    }
                    title_buf.clear();
                }
                Event::Text(text) | Event::Code(text) => {
                    title_buf.push_str(&text);
                }
                _ => {}
            }
        } else if in_image {
            if matches!(event, Event::End(TagEnd::Image)) {
                in_image = false;
            }
        } else {
            match event {
                Event::Start(Tag::Heading {
                    level: HeadingLevel::H1,
                    ..
                }) if title.is_none() => {
                    in_first_h1 = true;
                    title_buf.clear();
                }
                Event::Start(Tag::Heading { .. }) => {
                    text_buf.push_str("**");
                }
                Event::End(TagEnd::Heading(_)) => {
                    text_buf.push_str("**\n\n");
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    flush_text_buffer(&mut parts, &mut text_buf);
                    if let Some(resolved) = resolve_extension_image(&dest_url, extension_path) {
                        parts.push(OauthContentPart::Image(resolved));
                    }
                    in_image = true;
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    if dest_url.starts_with("https://") {
                        active_link_url = Some(dest_url.to_string());
                        text_buf.push('[');
                    } else {
                        tracing::warn!("Stripped non-https link URL: {}", dest_url);
                        active_link_url = None;
                    }
                }
                Event::End(TagEnd::Link) => {
                    if let Some(url) = active_link_url.take() {
                        text_buf.push_str("](");
                        text_buf.push_str(&url);
                        text_buf.push(')');
                    }
                }
                Event::Start(Tag::Paragraph) => {}
                Event::End(TagEnd::Paragraph) => {
                    text_buf.push_str("\n\n");
                }
                Event::Start(Tag::Strong) | Event::End(TagEnd::Strong) => {
                    text_buf.push_str("**");
                }
                Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {
                    text_buf.push('*');
                }
                Event::Start(Tag::Strikethrough) | Event::End(TagEnd::Strikethrough) => {
                    text_buf.push_str("~~");
                }
                Event::Start(Tag::List(start)) => {
                    list_stack.push(start);
                }
                Event::End(TagEnd::List(_)) => {
                    list_stack.pop();
                    text_buf.push('\n');
                }
                Event::Start(Tag::Item) => {
                    let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                    text_buf.push_str(&indent);
                    match list_stack.last_mut() {
                        Some(Some(num)) => {
                            text_buf.push_str(&format!("{}. ", num));
                            *num += 1;
                        }
                        _ => {
                            text_buf.push_str("- ");
                        }
                    }
                }
                Event::End(TagEnd::Item) => {
                    text_buf.push('\n');
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    text_buf.push_str("```\n");
                }
                Event::End(TagEnd::CodeBlock) => {
                    text_buf.push_str("```\n\n");
                }
                Event::Code(code) => {
                    text_buf.push('`');
                    text_buf.push_str(&code);
                    text_buf.push('`');
                }
                Event::Text(text) => {
                    text_buf.push_str(&text);
                }
                Event::SoftBreak => {
                    text_buf.push('\n');
                }
                Event::HardBreak => {
                    text_buf.push_str("  \n");
                }
                Event::Html(_) | Event::InlineHtml(_) => {}
                _ => {}
            }
        }
    }

    flush_text_buffer(&mut parts, &mut text_buf);

    ProcessedOauthMarkdown { title, parts }
}
