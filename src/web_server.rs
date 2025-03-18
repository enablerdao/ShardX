use log::{error, info, warn};
use std::path::Path;
use warp::Filter;
use std::time::Duration;
use tokio::time::sleep;

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
        info!("Webサーバーを起動中 (ポート: {})...", self.port);
        info!("Webディレクトリパス: {}", self.web_dir);

        // ウェブディレクトリが存在するか確認
        let web_dir_path = Path::new(&self.web_dir);
        if !web_dir_path.exists() {
            error!("Webディレクトリが存在しません: {}", self.web_dir);
            return Err(format!("Webディレクトリが存在しません: {}", self.web_dir));
        }
        
        if !web_dir_path.is_dir() {
            error!("Webディレクトリがディレクトリではありません: {}", self.web_dir);
            return Err(format!("Webディレクトリがディレクトリではありません: {}", self.web_dir));
        }
        
        info!("Webディレクトリの検証が完了しました");
        
        // index.htmlが存在するか確認
        let index_path = web_dir_path.join("index.html");
        if !index_path.exists() {
            error!("index.htmlが存在しません: {}", index_path.display());
            return Err(format!("index.htmlが存在しません: {}", index_path.display()));
        }
        
        info!("index.htmlの検証が完了しました: {}", index_path.display());

        // 静的ファイルを提供するルート
        let static_files = warp::fs::dir(self.web_dir.clone());
        info!("静的ファイルルートを作成しました");

        // ルートパスへのアクセスをindex.htmlにリダイレクト
        let index_path_str = index_path.to_str().unwrap().to_string();
        let root = warp::path::end().and(warp::fs::file(index_path_str));
        info!("ルートパスルートを作成しました");

        // CORSを設定
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_headers(vec!["Content-Type", "Authorization", "Accept"]);
        info!("CORSを設定しました");

        // ルートを結合
        let routes = root.or(static_files).with(cors).with(warp::log("web"));
        info!("ルートを結合しました");

        // サーバーを起動
        info!("Webサーバーを起動します: http://0.0.0.0:{}", self.port);
        
        // 最大3回まで再試行
        for attempt in 1..=3 {
            match self.start_server(routes.clone()).await {
                Ok(_) => {
                    info!("Webサーバーが正常に終了しました");
                    return Ok(());
                }
                Err(e) => {
                    if attempt < 3 {
                        warn!("Webサーバーの起動に失敗しました (試行 {}/3): {}", attempt, e);
                        sleep(Duration::from_secs(1)).await;
                    } else {
                        error!("Webサーバーの起動に失敗しました (最終試行): {}", e);
                        return Err(format!("Webサーバーの起動に失敗しました: {}", e));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// サーバーを実際に起動する内部メソッド
    async fn start_server<F>(&self, routes: F) -> Result<(), String>
    where
        F: warp::Filter + Clone + Send + Sync + 'static,
        F::Extract: warp::Reply,
    {
        info!("Webサーバーを起動しています: http://0.0.0.0:{}", self.port);
        
        // エラーハンドリングを追加
        let result = tokio::task::spawn(async move {
            warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;
        }).await;
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Webサーバータスクの実行中にエラーが発生しました: {}", e)),
        }
    }
}