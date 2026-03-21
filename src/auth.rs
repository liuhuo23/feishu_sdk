use log::debug;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

use crate::error::FeishuError;

const FEISHU_BASE_URL: &str = "https://open.feishu.cn/open-apis/";

#[derive(Debug)]
pub struct Auth {
    pub app_id: String,
    pub app_key: String,
    access_token: String,
}

impl Auth {
    /// 获取访问飞书API的权限
    /// ```ignore
    /// let auth = Auth::new(&app_id, &app_key);
    /// ```
    pub async fn new(app_id: &str, app_key: &str) -> Result<Self, FeishuError> {
        let access_token = get_tenant_access_token(app_id, app_key).await?;
        debug!("tenant access token acquired");
        Ok(Auth {
            app_id: app_id.to_string(),
            app_key: app_key.to_string(),
            access_token: access_token,
        })
    }

    pub fn get_header(&self) -> Result<HeaderMap, FeishuError> {
        let mut headermap = HeaderMap::new();
        let authorization = HeaderValue::from_str(&format!("Bearer {}", self.access_token))
            .map_err(|err| FeishuError::Header(err.to_string()))?;
        headermap.insert(header::AUTHORIZATION, authorization);
        headermap.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
        Ok(headermap)
    }
}

async fn get_tenant_access_token(app_id: &str, app_key: &str) -> Result<String, FeishuError> {
    let client = Client::new();
    let mut map = HashMap::new();
    map.insert("app_id", app_id);
    map.insert("app_secret", app_key);
    let resp = client
        .post(format!(
            "{}auth/v3/tenant_access_token/internal/",
            FEISHU_BASE_URL
        ))
        .json(&map)
        .send()
        .await?;
    let res: Value = resp.json().await?;
    let code = res
        .get("code")
        .and_then(Value::as_i64)
        .ok_or(FeishuError::MissingField("code"))?;
    if code != 0 {
        let msg = res
            .get("msg")
            .and_then(Value::as_str)
            .unwrap_or("unknown error")
            .to_string();
        return Err(FeishuError::Api { code, msg });
    }
    let token = res
        .get("tenant_access_token")
        .and_then(Value::as_str)
        .ok_or(FeishuError::MissingField("tenant_access_token"))?;
    Ok(token.to_string())
}
