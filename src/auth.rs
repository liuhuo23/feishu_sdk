use log::debug;
use reqwest::header::{self, HeaderMap};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

const FEISHU_BASE_URL: &str = "https://open.feishu.cn/open-apis/";

#[derive(Debug)]
pub struct Auth {
    pub app_id: String,
    pub app_key: String,
    access_token: String,
}

impl Auth {
    /// 获取访问飞书API的权限
    /// ```rust
    /// let auth = Auth::new(&app_id, &app_key);
    /// ```
    pub async fn new(app_id: &str, app_key: &str) -> Self {
        let access_token = get_tenant_access_token(app_id, app_key).await;
        debug!("{}", access_token);
        Auth {
            app_id: app_id.to_string(),
            app_key: app_key.to_string(),
            access_token: access_token,
        }
    }

    pub fn get_header(&self) -> HeaderMap {
        let mut headermap = HeaderMap::new();
        headermap.insert(
            header::AUTHORIZATION,
            format!("Bearer {}", self.access_token).parse().unwrap(),
        );
        headermap.insert(
            header::CONTENT_TYPE,
            "application/json; charset=utf-8".parse().unwrap(),
        );
        return headermap;
    }
}

async fn get_tenant_access_token(app_id: &str, app_key: &str) -> String {
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
        .await
        .unwrap();
    let res: Value = resp.json().await.unwrap();
    if !res.get("tenant_access_token").unwrap().is_string() {
        panic!("没有获取token");
    }
    let res = res.get("tenant_access_token").unwrap().as_str();
    match res {
        Some(v) => v.to_string(),
        _ => panic!("获取为空"),
    }
}
