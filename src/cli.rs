// CLI引数パースモジュール

use clap::Parser;
use std::path::PathBuf;

/// TWF (Terminal Wallpaper Fit) - ターミナルの背景画像に基づいてカラースキームを自動生成
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// 背景画像のパス（オプション）
    #[arg(short, long)]
    pub image: Option<PathBuf>,
    
    /// 自動検出を強制
    #[arg(short, long)]
    pub detect: bool,
    
    /// 背景色を直接指定（例: "#1e1e1e"）
    #[arg(short, long)]
    pub color: Option<String>,
    
    /// プレビューモード
    #[arg(short, long)]
    pub preview: bool,
    
    /// 設定を適用
    #[arg(short, long)]
    pub apply: bool,
    
    /// ロールバック
    #[arg(short, long)]
    pub rollback: bool,
    
    /// 詳細出力
    #[arg(short, long)]
    pub verbose: bool,
}
