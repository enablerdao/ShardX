#!/bin/bash

# ShardXの非インタラクティブインストールスクリプト
# 開発モードを選択して実行します

# 元のインストールスクリプトをダウンロード
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install.sh > /tmp/original_install.sh
chmod +x /tmp/original_install.sh

# 非インタラクティブモードで実行するための一時スクリプトを作成
cat > /tmp/auto_install.sh << 'INNEREOF'
#!/bin/bash

# 元のスクリプトを実行し、必要な入力を自動的に提供
export INSTALL_MODE=1  # 開発モードを選択
export AUTO_YES=y      # すべての質問に「y」と回答

# 元のスクリプトを実行
/tmp/original_install.sh << INPUTS
$INSTALL_MODE
$AUTO_YES
$AUTO_YES
$AUTO_YES
$AUTO_YES
INPUTS
INNEREOF

chmod +x /tmp/auto_install.sh
/tmp/auto_install.sh
