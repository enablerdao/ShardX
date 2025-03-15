# ShardX Windows インストールスクリプト

Write-Host "=== ShardX Windows インストールスクリプト ===" -ForegroundColor Cyan
Write-Host "このスクリプトはShardXをWindows環境にインストールします" -ForegroundColor Cyan
Write-Host ""

# 必要な依存関係をチェック
Write-Host "依存関係をチェックしています..." -ForegroundColor Yellow

# Rustがインストールされているか確認
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Rustがインストールされていません。インストールします..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
    .\rustup-init.exe -y
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}

# Gitがインストールされているか確認
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "Gitがインストールされていません。インストールしてください:" -ForegroundColor Red
    Write-Host "https://git-scm.com/download/win" -ForegroundColor Red
    exit 1
}

# Visual C++ Build Toolsがインストールされているか確認
if (-not (Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools" -PathType Container) -and
    -not (Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" -PathType Container)) {
    Write-Host "Visual C++ Build Toolsがインストールされていない可能性があります。" -ForegroundColor Yellow
    Write-Host "インストールするには以下のリンクからダウンロードしてください:" -ForegroundColor Yellow
    Write-Host "https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Yellow
    $continue = Read-Host "続行しますか？ (y/n)"
    if ($continue -ne "y") {
        exit 1
    }
}

# プロジェクトをビルド
Write-Host "ShardXをビルドしています..." -ForegroundColor Yellow
cargo build --release

Write-Host ""
Write-Host "=== インストールが完了しました！ ===" -ForegroundColor Green
Write-Host "ShardXを起動するには次のコマンドを実行してください:" -ForegroundColor Green
Write-Host ".\scripts\run.ps1" -ForegroundColor White
Write-Host ""
Write-Host "ブラウザで以下のURLにアクセスできます:" -ForegroundColor Green
Write-Host "- ウェブインターフェース: http://localhost:54867" -ForegroundColor White
Write-Host "- API: http://localhost:54868/api/v1/info" -ForegroundColor White