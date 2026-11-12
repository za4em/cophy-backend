use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{
    config::OpenRouterConfig,
    errors::{AppError, ReaderFault},
};
use crate::features::beans::model::BeanDraft;

const PROMPT: &str = include_str!("prompt.txt");

/// Reads a coffee bag from an image URL. The image itself stays in R2.
#[derive(Clone)]
pub(crate) struct BagReader {
    http: reqwest::Client,
    api_key: String,
    model: String,
    url: String,
    title: String,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: Vec<ContentPart>,
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatReply,
}

#[derive(Deserialize)]
struct ChatReply {
    content: String,
}

impl BagReader {
    pub(crate) fn new(config: OpenRouterConfig) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self {
            http,
            api_key: config.api_key,
            model: config.model,
            url: config.url,
            title: config.title,
            max_tokens: config.max_tokens,
        })
    }

    pub(crate) async fn read(&self, image_url: &str) -> Result<BeanDraft, AppError> {
        let request = ChatRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages: vec![ChatMessage {
                role: "user",
                content: vec![
                    ContentPart::Text {
                        text: PROMPT.to_owned(),
                    },
                    ContentPart::ImageUrl {
                        image_url: ImageUrl {
                            url: image_url.to_owned(),
                        },
                    },
                ],
            }],
        };

        let response = self
            .http
            .post(&self.url)
            .bearer_auth(&self.api_key)
            .header("X-Title", &self.title)
            .json(&request)
            .send()
            .await
            .map_err(|err| {
                tracing::error!(error = %err, "openrouter request failed");
                read_error(ReaderFault::RequestRejected)
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|err| {
            tracing::error!(error = %err, "openrouter response could not be read");
            read_error(ReaderFault::BodyUnreadable)
        })?;

        if !status.is_success() {
            log_snippet(&body);
            return Err(read_error(ReaderFault::Upstream));
        }

        let text = message_text(&body)?;
        parse_extraction(&text)
    }
}

fn message_text(body: &str) -> Result<String, AppError> {
    let response: ChatResponse = serde_json::from_str(body).map_err(|err| {
        tracing::error!(error = %err, "openrouter response had no message text");
        log_snippet(body);
        read_error(ReaderFault::NoMessage)
    })?;
    let text = response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .unwrap_or_default();
    if text.trim().is_empty() {
        log_snippet(body);
        return Err(read_error(ReaderFault::Empty));
    }
    Ok(text)
}

fn parse_extraction(raw: &str) -> Result<BeanDraft, AppError> {
    let json = json_object(raw).ok_or_else(|| {
        tracing::error!("bean extraction did not contain a json object");
        read_error(ReaderFault::NoJson)
    })?;
    let value: Value = serde_json::from_str(json).map_err(|err| {
        tracing::error!(error = %err, "bean extraction was not valid json");
        read_error(ReaderFault::InvalidJson)
    })?;
    let draft: BeanDraft = serde_json::from_value(value).map_err(|err| {
        tracing::error!(error = %err, "bean extraction did not match a bean");
        read_error(ReaderFault::NotACoffee)
    })?;
    let draft = draft.normalized();
    if draft.name.is_empty() {
        tracing::error!("bean extraction did not include a coffee name");
        return Err(read_error(ReaderFault::NameMissing));
    }
    Ok(draft)
}

fn json_object(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    let trimmed = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim();
    let trimmed = trimmed.strip_suffix("```").unwrap_or(trimmed).trim();
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    trimmed.get(start..=end)
}

fn read_error(fault: ReaderFault) -> AppError {
    AppError::Reader(fault)
}

fn log_snippet(body: &str) {
    let snippet: String = body.chars().take(300).collect();
    tracing::error!(body = %snippet, "openrouter response was not a bean");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fenced_json_and_joins_flavor_arrays() {
        let raw = r#"
            Here is the bag:
            ```json
            {"name":" Kieni AA ","origin":"Nyeri, Kenya","process":"Washed","elevation":"1,750 m","tasting_notes":["blackcurrant","grapefruit"]}
            ```
        "#;
        let draft = parse_extraction(raw).unwrap();
        assert_eq!(draft.name, "Kieni AA");
        assert_eq!(draft.origin, "Nyeri, Kenya");
        assert_eq!(draft.roaster, "");
        assert_eq!(draft.elevation, "1,750 m");
        assert_eq!(
            draft.tasting_notes,
            vec!["blackcurrant".to_owned(), "grapefruit".to_owned()]
        );
    }

    #[test]
    fn rejects_prose_without_json() {
        assert!(parse_extraction("I cannot see a coffee bag.").is_err());
    }

    #[test]
    fn empty_name_is_a_reader_failure() {
        assert_eq!(
            parse_extraction(r#"{"name":"   "}"#).unwrap_err(),
            AppError::Reader(ReaderFault::NameMissing)
        );
    }

    #[test]
    fn empty_model_message_is_empty() {
        assert_eq!(
            message_text(r#"{"choices":[]}"#).unwrap_err(),
            AppError::Reader(ReaderFault::Empty)
        );
    }

    #[test]
    fn body_without_a_chat_message_has_no_message() {
        assert_eq!(
            message_text("nope").unwrap_err(),
            AppError::Reader(ReaderFault::NoMessage)
        );
    }

    #[test]
    fn broken_json_object_is_invalid() {
        assert_eq!(
            parse_extraction("{not json}").unwrap_err(),
            AppError::Reader(ReaderFault::InvalidJson)
        );
    }

    #[test]
    fn note_string_is_not_a_coffee() {
        assert_eq!(
            parse_extraction(r#"{"name":"A","tasting_notes":"honey"}"#).unwrap_err(),
            AppError::Reader(ReaderFault::NotACoffee)
        );
    }

    #[test]
    fn prompt_names_the_draft_fields() {
        const SCHEMA: &str = r#"{"name":"","origin":"","roaster":"","process":"","elevation":"","tasting_notes":[]}"#;
        assert!(PROMPT.contains(SCHEMA));
        serde_json::from_str::<BeanDraft>(SCHEMA).unwrap();
    }
}
