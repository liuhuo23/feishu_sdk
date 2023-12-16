mod auth;
mod sheet;
#[allow(unused_imports)]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{auth::Auth, sheet::FeishuSheetBuild};
    use dotenv::dotenv;
    use log::{debug, info};
    use serde_json::json;
    use std::{env, fmt::format};
    fn init() {
        dotenv().ok();
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn login() {
        init();
        let app_id = env::var("app_id").unwrap_or("12312".to_string());
        let app_key = env::var("app_key").unwrap_or("12312".to_string());
        let auth = Auth::new(app_id.as_str(), app_key.as_str());
        println!("{:?}", auth);
    }

    #[test]
    fn get_sheet() {
        init();
        let app_id = env::var("app_id").unwrap_or("12312".to_string());
        let app_key = env::var("app_key").unwrap_or("12312".to_string());
        debug!("{}", app_id);
        let auth = Auth::new(app_id.as_str(), app_key.as_str());
        let feishubuild = FeishuSheetBuild::new(&auth);
        let sheet = feishubuild.get_sheet(
            "https://isw1t6yp68.feishu.cn/sheets/Yyk2sbhVAh4W7etDRqDcwV2WnRi?sheet=Owgp3G",
        );
        let value = sheet.read("A1:A3");
        debug!("{}", value);
        for i in 0..10 {
            sheet.write_line(
                &format!("A{i}:C{i}", i = i),
                vec![
                    json!(format!("测试1{i}", i = i)),
                    json!(format!("测试2{i}", i = i)),
                    json!(format!("测试3{i}", i = i)),
                ],
            );
        }
    }
}
