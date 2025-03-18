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
        info!("Web directory path: {}", self.web_dir);

        // ウェブディレクトリが存在するか確認
        let web_dir_path = Path::new(&self.web_dir);
        if !web_dir_path.exists() {
            error!("Web directory does not exist: {}", self.web_dir);
            return Err(format!("Web directory does not exist: {}", self.web_dir));
        }
        
        if !web_dir_path.is_dir() {
            error!("Web path is not a directory: {}", self.web_dir);
            return Err(format!("Web path is not a directory: {}", self.web_dir));
        }
        
        info!("Web directory exists and is valid");
        
        // index.htmlが存在するか確認
        let index_path = format!("{}/index.html", self.web_dir);
        let index_file_path = Path::new(&index_path);
        if !index_file_path.exists() {
            error!("index.html does not exist: {}", index_path);
            return Err(format!("index.html does not exist: {}", index_path));
        }
        
        info!("index.html exists at: {}", index_path);

        // 静的ファイルを提供するルート
        let static_files = warp::fs::dir(self.web_dir.clone());
        info!("Created static files route");

        // ルートパスへのアクセスをindex.htmlにリダイレクト
        let root = warp::path::end().and(warp::fs::file(index_path));
        info!("Created root path route");

        // CORSを設定
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_headers(vec!["Content-Type"]);
        info!("Configured CORS");

        // ルートを結合
        let routes = root.or(static_files).with(cors).with(warp::log("web"));
        info!("Combined routes");

        // サーバーを起動
        info!("Starting Web server at http://0.0.0.0:{}", self.port);
        warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;
        info!("Web server stopped");

        Ok(())
    }
}