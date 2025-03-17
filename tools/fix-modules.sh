#!/bin/bash
# モジュールの重複を自動修正するスクリプト

set -e

echo "ShardX モジュール自動修正ツール"
echo "================================"

# 重複するモジュールファイルを検出して修正
echo "1. 重複するモジュールファイルを検出しています..."
DUPLICATES_FOUND=0

find src -name "*.rs" | grep -v "mod.rs" | while read file; do
  base_name=$(basename "$file" .rs)
  dir_name=$(dirname "$file")
  
  if [ -d "$dir_name/$base_name" ] && [ -f "$dir_name/$base_name/mod.rs" ]; then
    echo "  修正: $file と $dir_name/$base_name/mod.rs の重複"
    DUPLICATES_FOUND=1
    
    # mod.rsの内容を確認
    if grep -q "pub mod" "$dir_name/$base_name/mod.rs"; then
      # mod.rsにサブモジュールがある場合はファイルを削除
      echo "    - $file を削除します（サブモジュールは $dir_name/$base_name/mod.rs に保持）"
      rm "$file"
    else
      # mod.rsにサブモジュールがない場合はディレクトリを削除
      echo "    - $dir_name/$base_name ディレクトリを削除します（内容は $file に保持）"
      rm -rf "$dir_name/$base_name"
    fi
  fi
done

if [ $DUPLICATES_FOUND -eq 0 ]; then
  echo "  ✅ 重複するモジュールファイルは見つかりませんでした"
fi

# 予約語をモジュール名として使用している場合を修正
echo "2. 予約語をモジュール名として使用している場合を修正しています..."
RESERVED_FOUND=0

RESERVED_KEYWORDS=("as" "break" "const" "continue" "crate" "else" "enum" "extern" "false" "fn" "for" "if" "impl" "in" "let" "loop" "match" "mod" "move" "mut" "pub" "ref" "return" "self" "Self" "static" "struct" "super" "trait" "true" "type" "unsafe" "use" "where" "while" "async" "await" "dyn" "abstract" "become" "box" "do" "final" "macro" "override" "priv" "typeof" "unsized" "virtual" "yield")

for keyword in "${RESERVED_KEYWORDS[@]}"; do
  # ディレクトリの場合
  if [ -d "src/$keyword" ]; then
    echo "  修正: 予約語 '$keyword' がディレクトリ名として使用されています"
    RESERVED_FOUND=1
    
    # 新しい名前を作成
    new_name="${keyword}_module"
    echo "    - src/$keyword を src/$new_name にリネームします"
    
    # ディレクトリをリネーム
    mv "src/$keyword" "src/$new_name"
    
    # 参照を更新
    find src -name "*.rs" -type f -exec sed -i "s/use crate::$keyword::/use crate::${new_name}::/g" {} \;
    find src -name "*.rs" -type f -exec sed -i "s/use crate::$keyword;/use crate::$new_name;/g" {} \;
    find src -name "*.rs" -type f -exec sed -i "s/pub mod $keyword;/pub mod $new_name;/g" {} \;
  fi
  
  # ファイルの場合
  if [ -f "src/$keyword.rs" ]; then
    echo "  修正: 予約語 '$keyword' がファイル名として使用されています"
    RESERVED_FOUND=1
    
    # 新しい名前を作成
    new_name="${keyword}_module"
    echo "    - src/$keyword.rs を src/$new_name.rs にリネームします"
    
    # ファイルをリネーム
    mv "src/$keyword.rs" "src/$new_name.rs"
    
    # 参照を更新
    find src -name "*.rs" -type f -exec sed -i "s/use crate::$keyword::/use crate::${new_name}::/g" {} \;
    find src -name "*.rs" -type f -exec sed -i "s/use crate::$keyword;/use crate::$new_name;/g" {} \;
    find src -name "*.rs" -type f -exec sed -i "s/pub mod $keyword;/pub mod $new_name;/g" {} \;
  fi
done

if [ $RESERVED_FOUND -eq 0 ]; then
  echo "  ✅ 予約語をモジュール名として使用している例は見つかりませんでした"
fi

# 特定のモジュール参照を修正
echo "3. 特定のモジュール参照を修正しています..."

# asyncモジュールの参照を修正
if find src -name "*.rs" -type f -exec grep -l "use crate::async::" {} \; | grep -q .; then
  echo "  修正: 'use crate::async::' を 'use crate::async_utils::' に変更します"
  find src -name "*.rs" -type f -exec sed -i 's/use crate::async::/use crate::async_utils::/g' {} \;
fi

if find src -name "*.rs" -type f -exec grep -l "use crate::async;" {} \; | grep -q .; then
  echo "  修正: 'use crate::async;' を 'use crate::async_utils;' に変更します"
  find src -name "*.rs" -type f -exec sed -i 's/use crate::async;/use crate::async_utils;/g' {} \;
fi

# mod.rsファイルで参照されているが存在しないモジュールを修正
echo "4. 存在しないモジュールへの参照を修正しています..."
MISSING_FOUND=0

find src -name "mod.rs" | while read mod_file; do
  dir=$(dirname "$mod_file")
  
  # モジュール宣言を抽出
  grep -E "^(pub )?mod [a-zA-Z0-9_]+;" "$mod_file" | sed -E 's/^(pub )?mod ([a-zA-Z0-9_]+);/\2/' | while read module; do
    # モジュールがファイルまたはディレクトリとして存在するか確認
    if [ ! -f "$dir/$module.rs" ] && [ ! -d "$dir/$module" ]; then
      echo "  修正: $mod_file で参照されているモジュール '$module' が存在しません"
      MISSING_FOUND=1
      
      # モジュール宣言をコメントアウト
      sed -i "s/^\(pub \)\?mod $module;/\/\/ \0 \/\/ TODO: このモジュールが見つかりません/g" "$mod_file"
      echo "    - モジュール宣言をコメントアウトしました"
    fi
  done
done

if [ $MISSING_FOUND -eq 0 ]; then
  echo "  ✅ 存在しないモジュールへの参照は見つかりませんでした"
fi

echo "5. コードをフォーマットしています..."
cargo fmt --all

echo "================================"
echo "モジュール修正が完了しました！"