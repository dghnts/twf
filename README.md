# TWF (Terminal Wallpaper Fit)

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/dghnts/twf/rust.yml?branch=main)](https://github.com/dghnts/twf/actions)
[![Crates.io](https://img.shields.io/crates/v/twf.svg)](https://crates.io/crates/twf)

[English](#english) | [日本語](#japanese)

---

<a name="japanese"></a>
## 日本語

> 🎨 ターミナルの背景画像または背景色を解析し、視認性の高いカラースキームとフォント設定を自動生成するRust製CLIツールです。

### デモ

```
$ twf --color "#282c34" --preview

TWF (Terminal Wallpaper Fit) v0.1.0
🎨 背景色を解析中...
✓ 解析完了

=== TWF カラースキーム プレビュー ===

ANSI 16色:
基本色:
■ Black           RGB( 40,  40,  40)
■ Red             RGB(  0, 158, 255)
■ Green           RGB(255,   0,  23)
■ Yellow          RGB(255,   0, 255)
■ Blue            RGB(  0, 180,  21)
■ Magenta         RGB(  0, 186, 255)
■ Cyan            RGB(195, 172,   0)
■ White           RGB(220, 220, 220)

コントラスト比: 14.05:1 (AAA評価)
```

### 機能

- **背景画像の自動検出**: iTerm2、Alacritty、Windows Terminal、GNOME Terminal、Kitty、WezTerm対応
- **背景色の自動検出**: OSC 11エスケープシーケンスを使用してターミナルの背景色を取得
- **カラースキームの自動生成**: ANSI 16色、256色パレット、True Color対応
- **コントラスト比の保証**: WCAG 2.1 AA基準（4.5:1以上）を満たす視認性の高い配色
- **フォント設定の最適化**: 背景の明度と彩度に基づいて最適なフォントウェイトを推奨
- **クロスプラットフォーム対応**: Linux、macOS、Windows
- **複数シェル対応**: bash、zsh、fish、PowerShell
- **安全なバックアップ**: 設定変更前に自動バックアップを作成し、ロールバック可能

### インストール

#### Cargoを使用

```bash
cargo install twf
```

#### ソースからビルド

```bash
git clone https://github.com/dghnts/twf.git
cd twf
cargo build --release
```

バイナリは `target/release/twf` に生成されます。

### 使用方法

#### 基本的な使い方

```bash
# 自動検出してプレビュー表示
twf

# 画像パスを指定
twf --image ~/Pictures/wallpaper.jpg

# 背景色を指定
twf --color "#1e1e1e"

# 設定を適用
twf --apply

# ロールバック
twf --rollback
```

#### 詳細な使用例

```bash
# 例1: 自動検出してプレビュー
$ twf
🔍 ターミナルを検出中...
✓ iTerm2を検出しました
🔍 背景画像を検出中...
✓ 背景画像を検出: ~/Pictures/wallpaper.png
🎨 画像を解析中...
✓ 解析完了

=== TWF カラースキーム プレビュー ===
[プレビュー表示]

設定を適用しますか？ (y/n): 

# 例2: 画像を指定して即座に適用
$ twf --image ~/Pictures/my-wallpaper.jpg --apply
🎨 画像を解析中...
✓ 解析完了
💾 設定をバックアップ中...
✓ バックアップ完了
✏️  設定を適用中...
✓ 設定を適用しました

# 例3: 背景色を指定してプレビュー
$ twf --color "#282c34" --preview
🎨 背景色を解析中...
✓ 解析完了

=== TWF カラースキーム プレビュー ===
[プレビュー表示]

# 例4: ロールバック
$ twf --rollback
🔄 設定をロールバック中...
✓ ロールバック完了
```

### オプション

| オプション | 短縮形 | 引数 | 説明 |
|-----------|--------|------|------|
| `--image` | `-i` | `<PATH>` | 背景画像のパスを指定。自動検出が失敗した場合や事前にテストしたい場合に使用 |
| `--detect` | `-d` | なし | 自動検出を強制。画像パスや背景色が指定されていても自動検出を試みる |
| `--color` | `-c` | `<COLOR>` | 背景色を直接指定（例: "#1e1e1e", "rgb(30,30,30)"）。背景画像を使用していない場合や特定の背景色に合わせたい場合に使用 |
| `--preview` | `-p` | なし | プレビューのみ表示し、設定を適用しない（デフォルト） |
| `--apply` | `-a` | なし | 設定をシェル設定ファイルに書き込み、現在のセッションに適用 |
| `--rollback` | `-r` | なし | 最後のバックアップから設定を復元 |
| `--verbose` | `-v` | なし | 詳細な出力を表示（検出プロセス、色解析の詳細など） |
| `--help` | `-h` | なし | ヘルプメッセージを表示 |
| `--version` | `-V` | なし | バージョン情報を表示 |

### 対応ターミナル

- **iTerm2** (macOS)
- **Alacritty** (Linux, macOS, Windows)
- **Windows Terminal** (Windows)
- **GNOME Terminal** (Linux)
- **Kitty** (Linux, macOS)
- **WezTerm** (Linux, macOS, Windows)

その他のターミナルでも、OSC 11エスケープシーケンスに対応していれば背景色検出が可能です。

### 対応シェル

- **bash**
- **zsh**
- **fish**
- **PowerShell**

### 対応画像形式

- PNG
- JPEG
- GIF
- BMP
- WebP

### 動作の仕組み

TWFは以下の順序で処理を実行します：

1. **入力ソースの決定**
   - `--image` オプションが指定されている場合: 画像パスを使用
   - `--color` オプションが指定されている場合: 背景色を使用
   - それ以外: 自動検出を試行

2. **自動検出フロー**（`--detect` または引数なしの場合）
   - ターミナルエミュレータの種類を判定
   - 設定ファイルから背景画像パスを検出
   - 検出失敗時: OSC 11で背景色を取得
   - それも失敗時: デフォルト背景色（黒）を使用

3. **色情報の解析**
   - 画像の場合: K-meansクラスタリングで主要色を抽出
   - 背景色の場合: 色空間変換（RGB → Lab）で色情報を取得

4. **カラースキームの生成**
   - フォアグラウンドカラーの計算（コントラスト比4.5:1以上を保証）
   - ANSI 16色の生成（背景の色相に調和する配色）
   - 256色パレットの生成（オプション）

5. **フォント設定の最適化**
   - 背景の明度に基づいてフォントウェイトを決定
   - 彩度が高い場合は太めのフォントを推奨

6. **設定の適用**
   - プレビュー表示（デフォルト）
   - ユーザー確認後、シェル設定ファイルに書き込み
   - 現在のターミナルセッションに即座に適用

### 設定ファイル

TWFは `~/.config/twf/config.toml` に設定を保存します。

```toml
[general]
backup_dir = "~/.config/twf/backups"
min_contrast_ratio = 4.5
verbose = false

[analyzer]
sample_size = 10000
kmeans_clusters = 5
kmeans_max_iterations = 100

[detector]
detection_timeout_ms = 1000

[generator]
prefer_true_color = true
generate_256_palette = true
```

### トラブルシューティング

#### 背景画像が検出できない

```bash
# 画像パスを明示的に指定
twf --image ~/Pictures/wallpaper.png

# または背景色を直接指定
twf --color "#1e1e1e"
```

#### 設定が適用されない

- シェル設定ファイル（`.bashrc`、`.zshrc`等）に書き込み権限があることを確認
- 新しいターミナルセッションを開くか、設定ファイルを再読み込み：
  ```bash
  source ~/.bashrc  # bashの場合
  source ~/.zshrc   # zshの場合
  ```

#### 設定を元に戻したい

```bash
twf --rollback
```

### 開発

#### ビルド

```bash
cargo build --release
```

#### テスト

```bash
# ユニットテスト
cargo test --lib

# プロパティベーステスト
cargo test --test property

# 統合テスト
cargo test --test integration

# すべてのテスト
cargo test
```

#### コードカバレッジ

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### ライセンス

MIT OR Apache-2.0

### 貢献

プルリクエストを歓迎します！バグ報告や機能リクエストは、GitHubのIssueでお願いします。

---

<a name="english"></a>
## English

> 🎨 A Rust CLI tool that analyzes terminal background images or colors and automatically generates high-visibility color schemes and font settings.

### Demo

```
$ twf --color "#282c34" --preview

TWF (Terminal Wallpaper Fit) v0.1.0
🎨 Analyzing background color...
✓ Analysis complete

=== TWF Color Scheme Preview ===

ANSI 16 colors:
Basic colors:
■ Black           RGB( 40,  40,  40)
■ Red             RGB(  0, 158, 255)
■ Green           RGB(255,   0,  23)
■ Yellow          RGB(255,   0, 255)
■ Blue            RGB(  0, 180,  21)
■ Magenta         RGB(  0, 186, 255)
■ Cyan            RGB(195, 172,   0)
■ White           RGB(220, 220, 220)

Contrast ratio: 14.05:1 (AAA rating)
```

### Features

- **Automatic Background Image Detection**: Supports iTerm2, Alacritty, Windows Terminal, GNOME Terminal, Kitty, WezTerm
- **Automatic Background Color Detection**: Uses OSC 11 escape sequences to retrieve terminal background color
- **Automatic Color Scheme Generation**: ANSI 16 colors, 256-color palette, True Color support
- **Contrast Ratio Guarantee**: Ensures high visibility with WCAG 2.1 AA standard (4.5:1 or higher)
- **Font Settings Optimization**: Recommends optimal font weight based on background brightness and saturation
- **Cross-Platform Support**: Linux, macOS, Windows
- **Multiple Shell Support**: bash, zsh, fish, PowerShell
- **Safe Backup**: Automatically creates backups before configuration changes, with rollback capability

### Installation

#### Using Cargo

```bash
cargo install twf
```

#### Build from Source

```bash
git clone https://github.com/dghnts/twf.git
cd twf
cargo build --release
```

The binary will be generated at `target/release/twf`.

### Usage

#### Basic Usage

```bash
# Auto-detect and preview
twf

# Specify image path
twf --image ~/Pictures/wallpaper.jpg

# Specify background color
twf --color "#1e1e1e"

# Apply settings
twf --apply

# Rollback
twf --rollback
```

#### Detailed Examples

```bash
# Example 1: Auto-detect and preview
$ twf
🔍 Detecting terminal...
✓ Detected iTerm2
🔍 Detecting background image...
✓ Background image detected: ~/Pictures/wallpaper.png
🎨 Analyzing image...
✓ Analysis complete

=== TWF Color Scheme Preview ===
[Preview display]

Apply settings? (y/n): 

# Example 2: Specify image and apply immediately
$ twf --image ~/Pictures/my-wallpaper.jpg --apply
🎨 Analyzing image...
✓ Analysis complete
💾 Creating backup...
✓ Backup complete
✏️  Applying settings...
✓ Settings applied

# Example 3: Specify background color and preview
$ twf --color "#282c34" --preview
🎨 Analyzing background color...
✓ Analysis complete

=== TWF Color Scheme Preview ===
[Preview display]

# Example 4: Rollback
$ twf --rollback
🔄 Rolling back settings...
✓ Rollback complete
```

### Options

| Option | Short | Argument | Description |
|--------|-------|----------|-------------|
| `--image` | `-i` | `<PATH>` | Specify background image path. Use when auto-detection fails or for testing |
| `--detect` | `-d` | None | Force auto-detection. Attempts auto-detection even if image path or color is specified |
| `--color` | `-c` | `<COLOR>` | Directly specify background color (e.g., "#1e1e1e", "rgb(30,30,30)"). Use when not using background image or to match specific color |
| `--preview` | `-p` | None | Display preview only without applying settings (default) |
| `--apply` | `-a` | None | Write settings to shell configuration file and apply to current session |
| `--rollback` | `-r` | None | Restore settings from last backup |
| `--verbose` | `-v` | None | Display detailed output (detection process, color analysis details, etc.) |
| `--help` | `-h` | None | Display help message |
| `--version` | `-V` | None | Display version information |

### Supported Terminals

- **iTerm2** (macOS)
- **Alacritty** (Linux, macOS, Windows)
- **Windows Terminal** (Windows)
- **GNOME Terminal** (Linux)
- **Kitty** (Linux, macOS)
- **WezTerm** (Linux, macOS, Windows)

Other terminals may work if they support OSC 11 escape sequences for background color detection.

### Supported Shells

- **bash**
- **zsh**
- **fish**
- **PowerShell**

### Supported Image Formats

- PNG
- JPEG
- GIF
- BMP
- WebP

### How It Works

TWF executes the following process:

1. **Determine Input Source**
   - If `--image` option is specified: Use image path
   - If `--color` option is specified: Use background color
   - Otherwise: Attempt auto-detection

2. **Auto-Detection Flow** (with `--detect` or no arguments)
   - Detect terminal emulator type
   - Detect background image path from configuration files
   - On detection failure: Retrieve background color using OSC 11
   - If that also fails: Use default background color (black)

3. **Analyze Color Information**
   - For images: Extract dominant colors using K-means clustering
   - For background color: Obtain color information via color space conversion (RGB → Lab)

4. **Generate Color Scheme**
   - Calculate foreground color (guarantees contrast ratio of 4.5:1 or higher)
   - Generate ANSI 16 colors (harmonious with background hue)
   - Generate 256-color palette (optional)

5. **Optimize Font Settings**
   - Determine font weight based on background brightness
   - Recommend bolder fonts for high saturation backgrounds

6. **Apply Settings**
   - Display preview (default)
   - After user confirmation, write to shell configuration file
   - Apply immediately to current terminal session

### Configuration File

TWF stores configuration in `~/.config/twf/config.toml`.

```toml
[general]
backup_dir = "~/.config/twf/backups"
min_contrast_ratio = 4.5
verbose = false

[analyzer]
sample_size = 10000
kmeans_clusters = 5
kmeans_max_iterations = 100

[detector]
detection_timeout_ms = 1000

[generator]
prefer_true_color = true
generate_256_palette = true
```

### Troubleshooting

#### Background Image Not Detected

```bash
# Explicitly specify image path
twf --image ~/Pictures/wallpaper.png

# Or directly specify background color
twf --color "#1e1e1e"
```

#### Settings Not Applied

- Verify write permissions for shell configuration file (`.bashrc`, `.zshrc`, etc.)
- Open a new terminal session or reload configuration file:
  ```bash
  source ~/.bashrc  # for bash
  source ~/.zshrc   # for zsh
  ```

#### Want to Revert Settings

```bash
twf --rollback
```

### Development

#### Build

```bash
cargo build --release
```

#### Testing

```bash
# Unit tests
cargo test --lib

# Property-based tests
cargo test --test property

# Integration tests
cargo test --test integration

# All tests
cargo test
```

#### Code Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### License

MIT OR Apache-2.0

### Contributing

Pull requests are welcome! For bug reports and feature requests, please use GitHub Issues.

---

## リポジトリ

GitHub: [https://github.com/dghnts/twf](https://github.com/dghnts/twf)

## 作者

[@dghnts](https://github.com/dghnts)

## 謝辞

- [image](https://github.com/image-rs/image) - Image processing library
- [clap](https://github.com/clap-rs/clap) - Command line argument parser
- [proptest](https://github.com/proptest-rs/proptest) - Property-based testing framework
