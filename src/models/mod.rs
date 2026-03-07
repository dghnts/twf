// データモデル定義

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// RGB色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Lab色空間（知覚的に均一な色空間）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f64,  // 明度 (0-100)
    pub a: f64,  // 緑-赤軸 (-128 to 127)
    pub b: f64,  // 青-黄軸 (-128 to 127)
}

/// XYZ色空間
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyz {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 色情報
#[derive(Debug, Clone)]
pub struct ColorInfo {
    /// 主要な色（Lab色空間）
    pub dominant_colors: Vec<Lab>,
    
    /// 平均明度 (0-100)
    pub average_lightness: f64,
    
    /// 彩度 (0-100)
    pub saturation: f64,
    
    /// 色相 (0-360)
    pub hue: f64,
    
    /// 暗い背景かどうか
    pub is_dark: bool,
}

/// ANSI 16色
#[derive(Debug, Clone)]
pub struct AnsiColors {
    pub black: Rgb,
    pub red: Rgb,
    pub green: Rgb,
    pub yellow: Rgb,
    pub blue: Rgb,
    pub magenta: Rgb,
    pub cyan: Rgb,
    pub white: Rgb,
    pub bright_black: Rgb,
    pub bright_red: Rgb,
    pub bright_green: Rgb,
    pub bright_yellow: Rgb,
    pub bright_blue: Rgb,
    pub bright_magenta: Rgb,
    pub bright_cyan: Rgb,
    pub bright_white: Rgb,
}

/// カラースキーム
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub foreground: Rgb,
    pub background: Lab,
    pub ansi_colors: AnsiColors,
    pub palette_256: Option<Vec<Rgb>>,
    pub supports_true_color: bool,
}

/// フォント設定
#[derive(Debug, Clone)]
pub struct FontConfig {
    pub weight: FontWeight,
    pub recommended_fonts: Vec<String>,
}

/// フォントウェイト
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Light,
    Normal,
    Medium,
    Bold,
}

impl FontWeight {
    pub fn increase(self) -> Self {
        match self {
            FontWeight::Light => FontWeight::Normal,
            FontWeight::Normal => FontWeight::Medium,
            FontWeight::Medium => FontWeight::Bold,
            FontWeight::Bold => FontWeight::Bold,
        }
    }
}

/// ターミナルタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalType {
    ITerm2,
    Alacritty,
    WindowsTerminal,
    GnomeTerminal,
    Kitty,
    WezTerm,
    Unknown,
}

/// シェルタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

/// 入力ソース
#[derive(Debug, Clone)]
pub enum InputSource {
    /// 画像パスが指定された
    ImagePath(PathBuf),
    
    /// 自動検出
    AutoDetect,
    
    /// 背景色が指定された
    Color(String),
}

/// バックアップ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub timestamp: DateTime<Utc>,
    pub checksum: String,
}

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// バックアップディレクトリ
    pub backup_dir: PathBuf,
    
    /// 最小コントラスト比
    pub min_contrast_ratio: f64,
    
    /// サンプリングサイズ
    pub sample_size: usize,
    
    /// タイムアウト（背景色検出）
    pub detection_timeout_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backup_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("twf")
                .join("backups"),
            min_contrast_ratio: 4.5,  // WCAG AA基準
            sample_size: 10000,
            detection_timeout_ms: 1000,
        }
    }
}

/// TWFのエラー型
#[derive(Debug, thiserror::Error)]
pub enum TwfError {
    /// 画像読み込みエラー
    #[error("画像の読み込みに失敗しました: {0}\n\n原因: 指定された画像ファイルが存在しないか、サポートされていない形式です\n\n解決方法:\n  - 画像ファイルのパスが正しいか確認してください\n  - サポートされている形式（PNG、JPEG、GIF、BMP、WebP）を使用してください\n  - ファイルの読み取り権限があるか確認してください")]
    ImageLoadError(String),
    
    /// 画像解析エラー
    #[error("画像の解析に失敗しました: {0}\n\n原因: 画像の処理中にエラーが発生しました\n\n解決方法:\n  - 画像ファイルが破損していないか確認してください\n  - 別の画像ファイルを試してください\n  - --verbose オプションで詳細なエラー情報を確認してください")]
    ImageAnalysisError(String),
    
    /// ファイルI/Oエラー
    #[error("ファイル操作に失敗しました: {0}\n\n原因: ファイルの読み書き中にエラーが発生しました\n\n解決方法:\n  - ファイルパスが正しいか確認してください\n  - ファイルの読み取り/書き込み権限があるか確認してください\n  - ディスク容量が十分にあるか確認してください")]
    IoError(#[from] std::io::Error),
    
    /// 設定ファイルパースエラー
    #[error("設定ファイルのパースに失敗しました: {0}\n\n原因: 設定ファイルの形式が不正です\n\n解決方法:\n  - 設定ファイルの構文が正しいか確認してください\n  - 設定ファイルが破損していないか確認してください\n  - デフォルト設定を使用する場合は、設定ファイルを削除してください")]
    ConfigParseError(String),
    
    /// ターミナル検出エラー
    #[error("ターミナルタイプを判定できませんでした\n\n原因: 現在のターミナルエミュレータを特定できませんでした\n\n解決方法:\n  - --image オプションで画像パスを明示的に指定してください\n    例: twf --image ~/Pictures/wallpaper.png\n  - --color オプションで背景色を直接指定してください\n    例: twf --color \"#1e1e1e\"\n  - サポートされているターミナル（iTerm2、Alacritty、Windows Terminal、GNOME Terminal、Kitty、WezTerm）を使用してください")]
    TerminalDetectionError,
    
    /// 背景色検出エラー
    #[error("背景色が検出できませんでした\n\n原因: ターミナルから背景色を取得できませんでした\n\n解決方法:\n  - --image オプションで画像パスを明示的に指定してください\n    例: twf --image ~/Pictures/wallpaper.png\n  - --color オプションで背景色を直接指定してください\n    例: twf --color \"#1e1e1e\"\n  - ターミナルがOSC 11エスケープシーケンスに対応しているか確認してください")]
    BackgroundColorDetectionError,
    
    /// コントラスト比計算エラー
    #[error("コントラスト比の計算に失敗しました: {0}\n\n原因: 色の変換またはコントラスト比の計算中にエラーが発生しました\n\n解決方法:\n  - 別の背景色または画像を試してください\n  - --verbose オプションで詳細なエラー情報を確認してください")]
    ContrastCalculationError(String),
    
    /// 設定適用エラー
    #[error("設定の適用に失敗しました: {0}\n\n原因: シェル設定ファイルへの書き込み中にエラーが発生しました\n\n解決方法:\n  - シェル設定ファイルへの書き込み権限があるか確認してください\n  - ディスク容量が十分にあるか確認してください\n  - --rollback オプションで設定を元に戻すことができます")]
    ConfigApplyError(String),
    
    /// バックアップエラー
    #[error("バックアップの作成に失敗しました: {0}\n\n原因: 設定ファイルのバックアップ中にエラーが発生しました\n\n解決方法:\n  - バックアップディレクトリへの書き込み権限があるか確認してください\n  - ディスク容量が十分にあるか確認してください\n  - バックアップディレクトリのパスが正しいか確認してください")]
    BackupError(String),
    
    /// ロールバックエラー
    #[error("ロールバックに失敗しました: {0}\n\n原因: バックアップからの復元中にエラーが発生しました\n\n解決方法:\n  - バックアップファイルが存在するか確認してください\n  - バックアップファイルが破損していないか確認してください\n  - 手動でバックアップファイルから設定を復元してください")]
    RollbackError(String),
    
    /// 色変換エラー
    #[error("色の変換に失敗しました: {0}\n\n原因: 色空間の変換中にエラーが発生しました\n\n解決方法:\n  - 入力された色の値が有効な範囲内にあるか確認してください\n  - 別の色または画像を試してください")]
    ColorConversionError(String),
    
    /// コントラスト比不足エラー
    #[error("コントラスト比が不足しています: {actual:.2} (最小要件: {required:.2})\n\n原因: 生成されたカラースキームがWCAG 2.1 AA基準を満たしていません\n\n解決方法:\n  - 別の背景色または画像を試してください\n  - より明るいまたは暗い背景を使用してください\n  - --verbose オプションで詳細な色情報を確認してください")]
    InsufficientContrast { actual: f64, required: f64 },
    
    /// パースエラー
    #[error("パースエラー: {0}\n\n原因: 入力データの解析中にエラーが発生しました\n\n解決方法:\n  - 入力形式が正しいか確認してください\n  - 色の指定は \"#RRGGBB\" または \"rgb(R,G,B)\" 形式を使用してください")]
    ParseError(String),
    
    /// シェル検出エラー
    #[error("シェルタイプを判定できませんでした\n\n原因: 現在のシェル環境を特定できませんでした\n\n解決方法:\n  - SHELL環境変数が正しく設定されているか確認してください\n  - サポートされているシェル（bash、zsh、fish、PowerShell）を使用してください\n  - 手動でシェル設定ファイルに設定を追加してください")]
    ShellDetectionError,
    
    /// 設定ファイル未検出エラー
    #[error("設定ファイルが見つかりません: {0}\n\n原因: シェル設定ファイルが存在しません\n\n解決方法:\n  - シェル設定ファイルを作成してください（例: touch ~/.bashrc）\n  - 正しいシェルを使用しているか確認してください\n  - 手動で設定ファイルのパスを指定してください")]
    ConfigFileNotFound(PathBuf),
    
    /// ファイル操作エラー
    #[error("ファイル操作に失敗しました: {0}\n\n原因: ファイルの読み書き中にエラーが発生しました\n\n解決方法:\n  - ファイルパスが正しいか確認してください\n  - ファイルの読み取り/書き込み権限があるか確認してください\n  - ディスク容量が十分にあるか確認してください")]
    FileOperationError(String),
}

/// Result型のエイリアス
pub type Result<T> = std::result::Result<T, TwfError>;
