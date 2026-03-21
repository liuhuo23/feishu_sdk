use dotenv::dotenv;
use feishu_sdk::auth::Auth;
use feishu_sdk::sheet::FeishuSheetBuild;
use log::debug;
use std::env;

#[tokio::test]
#[ignore = "requires Feishu credentials and network access"]
async fn sheet() {
    dotenv().ok();
    env_logger::init();
    // 从环境变量中获取key
    let app_id = env::var("app_id").unwrap();
    let app_key = env::var("app_key").unwrap();
    // 登陆
    let auth = Auth::new(&app_id, &app_key).await.unwrap();
    // 创建builder
    let feishubuild = FeishuSheetBuild::new(&auth);
    // 根据飞书表格的url创建sheet
    let sheet = feishubuild
        .get_sheet("https://isw1t6yp68.feishu.cn/sheets/Yyk2sbhVAh4W7etDRqDcwV2WnRi?sheet=Owgp3G")
        .unwrap();
    // 读取
    let value = sheet.read("A1:C3").await.unwrap();
    debug!("{}", value);
}
