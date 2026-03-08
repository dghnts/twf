#!/bin/bash
# TWFのデモを録画するスクリプト

set -e

echo "=== TWF Demo Recording Script ==="
echo ""

# asciinemaがインストールされているか確認
if ! command -v asciinema &> /dev/null; then
    echo "❌ asciinemaがインストールされていません"
    echo ""
    echo "インストール方法:"
    echo "  Ubuntu/Debian: sudo apt install asciinema"
    echo "  macOS: brew install asciinema"
    echo "  その他: pip install asciinema"
    echo ""
    exit 1
fi

echo "✓ asciinemaが見つかりました"
echo ""

# デモディレクトリを作成
mkdir -p demos

# デモ1: 背景色を指定してプレビュー
echo "📹 デモ1を録画中: 背景色指定 (ダークテーマ)"
asciinema rec demos/demo_dark.cast -c "bash -c 'echo \"$ twf --color \\\"#282c34\\\" --preview\"; sleep 1; twf --color \"#282c34\" --preview; sleep 3'" --overwrite

echo "✓ デモ1完了"
echo ""

# デモ2: 背景色を指定してプレビュー (ライトテーマ)
echo "📹 デモ2を録画中: 背景色指定 (ライトテーマ)"
asciinema rec demos/demo_light.cast -c "bash -c 'echo \"$ twf --color \\\"#f0f0f0\\\" --preview\"; sleep 1; twf --color \"#f0f0f0\" --preview; sleep 3'" --overwrite

echo "✓ デモ2完了"
echo ""

echo "=== 録画完了 ==="
echo ""
echo "生成されたファイル:"
echo "  - demos/demo_dark.cast"
echo "  - demos/demo_light.cast"
echo ""
echo "次のステップ:"
echo "1. GIFに変換する場合:"
echo "   - agg (https://github.com/asciinema/agg) をインストール"
echo "   - agg demos/demo_dark.cast demos/demo_dark.gif"
echo ""
echo "2. README.mdに埋め込む場合:"
echo "   - GitHubにアップロード後、以下のように記述:"
echo "   - [![asciicast](https://asciinema.org/a/XXXXX.svg)](https://asciinema.org/a/XXXXX)"
echo ""
