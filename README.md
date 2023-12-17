# feishu_sdk

一个用来学习 rust 的小项目

# 使用方式

```rust
use dotenv::dotenv;
use feishu_sdk::blocking::{auth::Auth, sheet::FeishuSheetBuild};
use std::env;
fn main() {
    dotenv().ok();
    env_logger::init();
    // 从环境变量中获取key
    let app_id = env::var("app_id").unwrap();
    let app_key = env::var("app_key").unwrap();
    // 登陆
    let auth = Auth::new(&app_id, &app_key);
    // 创建builder
    let feishubuild = FeishuSheetBuild::new(&auth);
    // 根据飞书表格的url创建sheet
    let sheet = feishubuild
        .get_sheet("");
    // 读取
    let value = sheet.read("A1:C3");
    println!("{}", value);
}


```
