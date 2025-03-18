#!/usr/bin/env python3
import http.server
import socketserver
import os
import sys

# ポート番号を取得（デフォルトは51084）
PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 51084

# Webディレクトリを取得（デフォルトは./web）
WEB_DIR = sys.argv[2] if len(sys.argv) > 2 else "./web"

# 現在のディレクトリを保存
original_dir = os.getcwd()

# Webディレクトリに移動
os.chdir(WEB_DIR)

# サーバーの設定
Handler = http.server.SimpleHTTPRequestHandler

# サーバーを起動
with socketserver.TCPServer(("", PORT), Handler) as httpd:
    print(f"ShardX Web Server running at http://localhost:{PORT}")
    print(f"Serving files from: {os.getcwd()}")
    print("Press Ctrl+C to stop")
    httpd.serve_forever()