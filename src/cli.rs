// CLI引数パースモジュール

use clap::Parser;
use std::path::PathBuf;

/// TWF (Terminal Wallpaper Fit) - ターミナルの背景画像または背景色を解析し、
/// 視認性の高いカラースキームとフォント設定を自動生成するCLIツール
/// 
/// 使用例:
///   twf                                    # 自動検出してプレビュー
///   twf --image ~/Pictures/wallpaper.png   # 画像を指定してプレビュー
///   twf --color "#1e1e1e" --apply          # 背景色を指定して適用
///   twf --detect --apply                   # 自動検出して適用
///   twf --rollback                         # 設定をロールバック
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "ターミナルの背景画像または背景色を解析し、視認性の高いカラースキームを自動生成",
    long_about = "TWF (Terminal Wallpaper Fit) は、ターミナルの背景画像または背景色を解析し、\n\
                  視認性の高いカラースキームとフォント設定を自動生成するCLIツールです。\n\n\
                  対応ターミナル: iTerm2, Alacritty, Windows Terminal, GNOME Terminal, Kitty, WezTerm\n\
                  対応シェル: bash, zsh, fish, PowerShell"
)]
pub struct CliArgs {
    /// 背景画像のパスを指定
    /// 
    /// 自動検出が失敗した場合や、事前に特定の画像でテストしたい場合に使用します。
    /// 
    /// 例: --image ~/Pictures/wallpaper.png
    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "背景画像のパスを指定（自動検出が失敗した場合や事前にテストしたい場合に使用）"
    )]
    pub image: Option<PathBuf>,
    
    /// 自動検出を強制
    /// 
    /// 画像パスや背景色が指定されていても、自動検出を試みます。
    /// ターミナルエミュレータの設定から背景画像パスを検出します。
    #[arg(
        short,
        long,
        help = "自動検出を強制（ターミナル設定から背景画像を検出）"
    )]
    pub detect: bool,
    
    /// 背景色を直接指定
    /// 
    /// 背景画像を使用していない場合や、特定の背景色に合わせたカラースキームを
    /// 生成したい場合に使用します。
    /// 
    /// 対応形式:
    ///   - 16進数: "#1e1e1e", "#282c34"
    ///   - RGB: "rgb(30,30,30)"
    /// 
    /// 例: --color "#1e1e1e"
    #[arg(
        short,
        long,
        value_name = "COLOR",
        help = "背景色を直接指定（背景画像を使用していない場合や特定の背景色に合わせたい場合に使用）"
    )]
    pub color: Option<String>,
    
    /// プレビューのみモード
    /// 
    /// 生成したカラースキームをプレビュー表示のみ行い、設定を適用しません。
    /// 確認プロンプトも表示されません。
    #[arg(
        short,
        long,
        help = "プレビューのみ表示（設定を適用せず、確認プロンプトも表示しない）"
    )]
    pub preview: bool,
    
    /// ロールバック
    /// 
    /// 最後のバックアップから設定を復元します。
    #[arg(
        short,
        long,
        help = "最後のバックアップから設定を復元"
    )]
    pub rollback: bool,
    
    /// 詳細出力
    /// 
    /// 検出プロセス、色解析の詳細、コントラスト比の計算結果など、
    /// 詳細な情報を出力します。
    #[arg(
        short,
        long,
        help = "詳細な出力を表示（検出プロセス、色解析の詳細など）"
    )]
    pub verbose: bool,
}
