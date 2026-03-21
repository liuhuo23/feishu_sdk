use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum FeishuError {
    Http(reqwest::Error),
    Json(serde_json::Error),
    Header(String),
    InvalidSheetUrl(String),
    MissingField(&'static str),
    Api { code: i64, msg: String },
}

impl Display for FeishuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeishuError::Http(err) => write!(f, "http error: {err}"),
            FeishuError::Json(err) => write!(f, "json parse error: {err}"),
            FeishuError::Header(err) => write!(f, "invalid header value: {err}"),
            FeishuError::InvalidSheetUrl(url) => write!(f, "invalid feishu sheet url: {url}"),
            FeishuError::MissingField(field) => write!(f, "missing field: {field}"),
            FeishuError::Api { code, msg } => write!(f, "feishu api error, code={code}, msg={msg}"),
        }
    }
}

impl std::error::Error for FeishuError {}

impl From<reqwest::Error> for FeishuError {
    fn from(value: reqwest::Error) -> Self {
        FeishuError::Http(value)
    }
}

impl From<serde_json::Error> for FeishuError {
    fn from(value: serde_json::Error) -> Self {
        FeishuError::Json(value)
    }
}
