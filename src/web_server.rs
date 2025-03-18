use log::{error, info};
use std::path::Path;
use warp::Filter;

/// ウェブサーバー
pub struct WebServer {
    /// ウェブディレクトリのパス
    web_dir: String,
    /// サーバーのポート
    port: u16,
}

impl WebServer {
    /// 新しいウェブサーバーを作成
    pub fn new(web_dir: String, port: u16) -> Self {
        Self { web_dir, port }
    }

    /// サーバーを起動
    pub async fn start(&self) -> Result<(), String> {
        info!("Starting Web server on port {}", self.port);

        // ウェブディレクトリが存在するか確認
        let web_dir_path = Path::new(&self.web_dir);
        if !web_dir_path.exists() || !web_dir_path.is_dir() {
            error!("Web directory does not exist: {}", self.web_dir);
            return Err(format!("Web directory does not exist: {}", self.web_dir));
        }

        // 静的ファイルを提供するルート
        let static_files = warp::fs::dir(self.web_dir.clone());

        // ルートパスへのアクセスをindex.htmlにリダイレクト
        let root = warp::path::end().and(warp::fs::file(format!("{}/index.html", self.web_dir)));

        // CORSを設定
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_headers(vec!["Content-Type"]);

        // ルートを結合
        let routes = root.or(static_files).with(cors).with(warp::log("web"));

        // サーバーを起動
        warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;

        Ok(())
    }
}