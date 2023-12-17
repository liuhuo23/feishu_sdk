use dotenv::dotenv;
use feishu_sdk::blocking::{auth::Auth, sheet::FeishuSheetBuild};
use std::env;
fn main() {
    dotenv().ok();
    env_logger::init();
    let app_id = env::var("app_id").unwrap();
    let app_key = env::var("app_key").unwrap();
    let auth = Auth::new(&app_id, &app_key);
    let feishubuild = FeishuSheetBuild::new(&auth);
    let sheet = feishubuild
        .get_sheet("https://isw1t6yp68.feishu.cn/sheets/Yyk2sbhVAh4W7etDRqDcwV2WnRi?sheet=Owgp3G");
    let value = sheet.read("A1:C3");
    println!("{}", value);
}
