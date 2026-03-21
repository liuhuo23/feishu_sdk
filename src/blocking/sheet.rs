use log::{debug, error};
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use reqwest::Url;
use serde_json::{json, Value};
use std::fmt::Display;
use std::rc::Rc;

use super::auth::Auth;
use crate::error::FeishuError;

const SHEET_URL: &str = "https://open.feishu.cn/open-apis/sheets/v2/spreadsheets/";

pub struct FeishuSheet {
    sheet_id: String,
    sheet_token: String,
    client: Rc<Client>,
    header: HeaderMap,
}

impl FeishuSheet {
    fn new(url: &str, auth: &Auth) -> Result<Self, FeishuError> {
        let (sheet_token, sheet_id) = parse_sheet_url(url)?;
        let client = Client::new();
        Ok(FeishuSheet {
            sheet_token,
            sheet_id,
            client: Rc::new(client),
            header: auth.get_header()?,
        })
    }
    pub fn write_line(&self, range: &str, value: Vec<Value>) -> Result<(), FeishuError> {
        let values: Vec<Vec<Value>> = vec![value];
        self.write(range, values)
    }
    pub fn write(&self, range: &str, value: Vec<Vec<Value>>) -> Result<(), FeishuError> {
        let body = json!({
            "valueRange": {
                "range": format!("{}!{}", self.sheet_id, range),
                "values": value
            }
        });
        debug!("json: {}", body);
        let value = self.send_json(&body)?;
        ensure_api_success(&value)?;
        Ok(())
    }
    pub fn read(&self, range: &str) -> Result<Value, FeishuError> {
        let url = format!(
            "https://open.feishu.cn/open-apis/sheets/v2/spreadsheets/{spreadsheetToken}\
            /values/{range}?valueRenderOption=ToString&dateTimeRenderOption=FormattedString",
            spreadsheetToken = self.sheet_token,
            range = format!("{}!{}", self.sheet_id, range)
        );
        let resp = self
            .client
            .get(url)
            .headers(self.header.clone())
            .send()
            .map_err(FeishuError::from)?;
        let value: Value = resp.json().map_err(FeishuError::from)?;
        if let Err(err) = ensure_api_success(&value) {
            error!("read sheet failed: {}", err);
            return Err(err);
        }
        let values = value["data"]["valueRange"]["values"].clone();
        if values.is_null() {
            return Err(FeishuError::MissingField("data.valueRange.values"));
        }
        Ok(values)
    }
    fn send_json(&self, body: &Value) -> Result<Value, FeishuError> {
        let sheet_url = format!("{}{}/values_prepend", SHEET_URL, self.sheet_token);
        debug!("sheet_url: {}", sheet_url);
        debug!("valuerange->range{}", body);
        let resp = self
            .client
            .post(sheet_url)
            .headers(self.header.clone())
            .json(body)
            .send()
            .map_err(FeishuError::from)?;
        let value = resp.json().map_err(FeishuError::from)?;
        Ok(value)
    }

    pub fn add_sheet(&self, sheet_title: &str) -> Result<FeishuSheet, FeishuError> {
        let url = format!(
            "https://open.feishu.cn/open-apis/sheets/v2/spreadsheets/{spreadsheetToken}/sheets_batch_update",
            spreadsheetToken = self.sheet_token
        );
        let value = json!({
          "requests": [
            {
              "addSheet": {
                "properties": {
                  "title": sheet_title,
                  "index": 0
                }
              }
            }
          ]
        });
        let resp = self
            .client
            .post(url)
            .headers(self.header.clone())
            .json(&value)
            .send()
            .map_err(FeishuError::from)?;
        let value: Value = resp.json().map_err(FeishuError::from)?;
        if let Err(err) = ensure_api_success(&value) {
            error!("add sheet failed: {}", err);
            return Err(err);
        }
        debug!("{}", value);
        let sheet_id = value["data"]["replies"][0]["addSheet"]["properties"]["sheetId"]
            .as_str()
            .ok_or(FeishuError::MissingField(
                "data.replies[0].addSheet.properties.sheetId",
            ))?
            .to_string();
        Ok(FeishuSheet {
            sheet_token: self.sheet_token.clone(),
            client: self.client.clone(),
            sheet_id,
            header: self.header.clone(),
        })
    }
}

impl Display for FeishuSheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = format!(
            r#"{{sheet_token: {}, sheet_id: {}}}"#,
            self.sheet_token, self.sheet_id
        );
        write!(f, "Sheet {:?}", output)
    }
}

pub struct FeishuSheetBuild<'a> {
    auth: &'a Auth,
}

impl<'a> FeishuSheetBuild<'a> {
    pub fn new(auth: &'a Auth) -> Self {
        FeishuSheetBuild { auth: auth }
    }
    pub fn get_sheet(&self, url: &str) -> Result<FeishuSheet, FeishuError> {
        FeishuSheet::new(url, self.auth)
    }
}

fn parse_sheet_url(url: &str) -> Result<(String, String), FeishuError> {
    let parsed = Url::parse(url).map_err(|_| FeishuError::InvalidSheetUrl(url.to_string()))?;
    let mut segments = parsed
        .path_segments()
        .ok_or_else(|| FeishuError::InvalidSheetUrl(url.to_string()))?;
    let first = segments.next();
    let second = segments.next();
    let sheet_token = match (first, second) {
        (Some("sheets"), Some(token)) if !token.is_empty() => token.to_string(),
        _ => return Err(FeishuError::InvalidSheetUrl(url.to_string())),
    };
    let sheet_id = parsed
        .query_pairs()
        .find_map(|(key, value)| {
            if key == "sheet" && !value.is_empty() {
                Some(value.into_owned())
            } else {
                None
            }
        })
        .ok_or_else(|| FeishuError::InvalidSheetUrl(url.to_string()))?;
    debug!("sheet_token: {}", sheet_token);
    Ok((sheet_token, sheet_id))
}

fn ensure_api_success(value: &Value) -> Result<(), FeishuError> {
    let code = value
        .get("code")
        .and_then(Value::as_i64)
        .ok_or(FeishuError::MissingField("code"))?;
    if code == 0 {
        return Ok(());
    }
    let msg = value
        .get("msg")
        .and_then(Value::as_str)
        .unwrap_or("unknown error")
        .to_string();
    Err(FeishuError::Api { code, msg })
}
