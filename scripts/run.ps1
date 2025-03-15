# ShardX 起動スクリプト

Write-Host "=== ShardX 起動スクリプト ===" -ForegroundColor Cyan
Write-Host "ShardXを起動しています..." -ForegroundColor Cyan
Write-Host ""

# ビルドされたバイナリが存在するか確認
if (-not (Test-Path ".\target\release\shardx.exe" -PathType Leaf)) {
    Write-Host "ShardXがビルドされていません。インストールスクリプトを実行してください:" -ForegroundColor Red
    Write-Host "  .\scripts\windows_install.ps1" -ForegroundColor White
    exit 1
}

# 設定ファイルが存在するか確認
if (-not (Test-Path ".\config\default.toml" -PathType Leaf)) {
    Write-Host "設定ファイルが見つかりません。デフォルト設定を作成します..." -ForegroundColor Yellow
    
    # configディレクトリが存在しない場合は作成
    if (-not (Test-Path ".\config" -PathType Container)) {
        New-Item -Path ".\config" -ItemType Directory | Out-Null
    }
    
    # デフォルト設定ファイルを作成
    @"
[server]
host = "0.0.0.0"
port = 54868
web_port = 54867

[node]
id = "local_node"
initial_shards = 10

[storage]
data_dir = "./data"
"@ | Out-File -FilePath ".\config\default.toml" -Encoding utf8
}

# データディレクトリを作成
if (-not (Test-Path ".\data" -PathType Container)) {
    New-Item -Path ".\data" -ItemType Directory | Out-Null
}

# ShardXを起動
Write-Host "ShardXを起動しています..." -ForegroundColor Green
Write-Host "ログはターミナルに表示されます。Ctrl+Cで終了できます。" -ForegroundColor Yellow
Write-Host ""
Write-Host "ブラウザで以下のURLにアクセスできます:" -ForegroundColor Green
Write-Host "- ウェブインターフェース: http://localhost:54867" -ForegroundColor White
Write-Host "- API: http://localhost:54868/api/v1/info" -ForegroundColor White
Write-Host ""

# バイナリを実行
.\target\release\shardx.exe