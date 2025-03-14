#!/bin/bash
set -e

# Nginxを起動
nginx

# ShardXノードを起動
exec /app/bin/shardx