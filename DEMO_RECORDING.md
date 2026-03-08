# TWF デモの録画方法

このドキュメントでは、TWFのデモGIFやスクリーンショットを作成する方法を説明します。

## 必要なツール

### 1. asciinema（ターミナル録画）

**インストール:**

```bash
# Ubuntu/Debian
sudo apt install asciinema

# macOS
brew install asciinema

# pip経由
pip install asciinema
```

### 2. agg（asciinemaをGIFに変換）

**インストール:**

```bash
# macOS
brew install agg

# Cargo経由
cargo install --git https://github.com/asciinema/agg

# バイナリダウンロード
# https://github.com/asciinema/agg/releases
```

## デモの録画手順

### 自動録画スクリプトを使用

```bash
# スクリプトを実行
./scripts/record_demo.sh
```

このスクリプトは以下のデモを自動的に録画します：
- ダークテーマのカラースキーム生成
- ライトテーマのカラースキーム生成

### 手動で録画

```bash
# デモディレクトリを作成
mkdir -p demos

# ダークテーマのデモを録画
asciinema rec demos/demo_dark.cast

# 録画中に以下を実行:
twf --color "#282c34" --preview

# Ctrl+D で録画終了
```

## GIFへの変換

```bash
# aggを使用してGIFに変換
agg demos/demo_dark.cast demos/demo_dark.gif

# オプション指定
agg --speed 1.5 --theme monokai demos/demo_dark.cast demos/demo_dark.gif
```

### aggのオプション

- `--speed <FLOAT>`: 再生速度（デフォルト: 1.0）
- `--theme <THEME>`: カラーテーマ（asciinema, monokai, solarized-dark等）
- `--font-size <SIZE>`: フォントサイズ（デフォルト: 14）
- `--cols <COLS>`: 列数（デフォルト: 80）
- `--rows <ROWS>`: 行数（デフォルト: 24）

## README.mdへの埋め込み

### 方法1: ローカルGIFファイル

```markdown
### デモ

![TWF Demo - Dark Theme](demos/demo_dark.gif)
```

### 方法2: asciinema.orgにアップロード

```bash
# asciinema.orgにアップロード
asciinema upload demos/demo_dark.cast
```

アップロード後、URLが表示されます。README.mdに以下のように記述:

```markdown
### デモ

[![asciicast](https://asciinema.org/a/XXXXX.svg)](https://asciinema.org/a/XXXXX)
```

## 推奨設定

### ターミナル設定

- フォント: Fira Code, Hack, または Inconsolata
- フォントサイズ: 14-16pt
- ウィンドウサイズ: 80x24 または 100x30

### 録画のベストプラクティス

1. **短く保つ**: 10-30秒程度
2. **クリアな出力**: 不要な出力を避ける
3. **適切な速度**: `--speed 1.5` で少し速めに
4. **テーマの一貫性**: ダークとライトの両方を用意

## トラブルシューティング

### asciinemaが見つからない

```bash
# パスを確認
which asciinema

# 再インストール
pip install --upgrade asciinema
```

### aggでエラーが発生

```bash
# 最新版を確認
agg --version

# 再インストール
cargo install --git https://github.com/asciinema/agg --force
```

### GIFサイズが大きすぎる

```bash
# 解像度を下げる
agg --cols 80 --rows 24 demos/demo_dark.cast demos/demo_dark.gif

# gifsicleで最適化
gifsicle -O3 demos/demo_dark.gif -o demos/demo_dark_optimized.gif
```

## 参考リンク

- [asciinema](https://asciinema.org/)
- [agg](https://github.com/asciinema/agg)
- [gifsicle](https://www.lcdf.org/gifsicle/)
