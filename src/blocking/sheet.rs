use log::{debug, error};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std::fmt::Display;
use std::rc::Rc;

use super::auth::Auth;

const SHEET_URL: &str = "https://open.feishu.cn/open-apis/sheets/v2/spreadsheets/";

pub struct FeishuSheet {
    sheet_id: String,
    sheet_token: String,
    client: Rc<Client>,
    header: HeaderMap,
}

impl FeishuSheet {
    fn new(url: &str, auth: &Auth) -> Self {
        let token_regex = Regex::new(r"sheets/(.*)\?").unwrap();
        let id_regex = Regex::new(r"sheet=(.+)*").unwrap();
        let token_res = &token_regex.captures_at(url, 0).unwrap()[1];
        debug!("sheet_token: {}", token_res);
        let id_res = &id_regex.captures_at(url, 0).unwrap()[1];
        let client = Client::new();
        FeishuSheet {
            sheet_token: token_res.to_string(),
            sheet_id: id_res.to_string(),
            client: Rc::new(client),
            header: auth.get_header(),
        }
    }
    pub fn write_line(&self, range: &str, value: Vec<Value>) {
        let values: Vec<Vec<Value>> = vec![value];
        self.write(range, values);
    }
    pub fn write(&self, range: &str, value: Vec<Vec<Value>>) {
        let json_str = format!(
            r#"{{
            "valueRange": {{
                "range": "{range}",
                "values": {value}
            }}
        }}"#,
            range = format!("{}!{}", self.sheet_id, range),
            value = json!(value),
        );
        debug!("json: {}", json_str);
        let value = self.send_json(&json_str);
        println!("{}", value);
    }
    pub fn read(&self, range: &str) -> Value {
        let url = format!(
            "https://open.feishu.cn/open-apis/sheets/v2/spreadsheets/{spreadsheetToken}\
            /values/{range}?valueRenderOption=ToString&dateTimeRenderOption=FormattedString",
            spreadsheetToken = self.sheet_token,
            range = format!("{}!{}", self.sheet_id, range)
        );
        let res = self.client.get(url).headers(self.header.clone()).send();
        match res {
            Ok(resp) => {
                let value: Value = resp.json().unwrap();
                if value["code"] != 0 {
                    error!("msg{}", value["msg"]);
                    panic!("{}", value);
                }
                value["data"]["valueRange"]["values"].clone()
            }
            Err(_) => panic!("连接失败！"),
        }
    }
    fn send_json(&self, json_str: &str) -> Value {
        let sheet_url = format!("{}{}/values_prepend", SHEET_URL, self.sheet_token);
        debug!("sheet_url: {}", sheet_url);
        let value: Value = serde_json::from_str(json_str).unwrap();
        debug!("valuerange->range{}", value);
        let res = self
            .client
            .post(sheet_url)
            .headers(self.header.clone())
            .json(&value)
            .send();
        match res {
            Ok(resp) => resp.json().unwrap(),
            Err(_) => panic!("发送失败！"),
        }
    }

    pub fn add_sheet(&self, sheet_title: &str) -> FeishuSheet {
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
        let res = self
            .client
            .post(url)
            .headers(self.header.clone())
            .json(&value)
            .send();
        match res {
            Ok(resp) => {
                let value: Value = resp.json().unwrap();
                if value["code"] != 0 {
                    error!("{}", value);
                    panic!("创建表返回失败")
                }
                debug!("{}", value);
                let sheet_id: String = value["data"]["replies"][0]["addSheet"]["properties"]
                    ["sheetId"]
                    .as_str()
                    .unwrap()
                    .to_string();
                FeishuSheet {
                    sheet_token: self.sheet_token.clone(),
                    client: self.client.clone(),
                    sheet_id,
                    header: self.header.clone(),
                }
            }
            Err(_) => panic!("创建表失败"),
        }
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
    pub fn get_sheet(&self, url: &str) -> FeishuSheet {
        let feishusheet = FeishuSheet::new(url, self.auth);
        feishusheet
    }
}
