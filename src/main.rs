// TWF (Terminal Wallpaper Fit) - メインエントリーポイント

mod cli;
mod detector;
mod analyzer;
mod generator;
mod applier;
mod preview;
mod models;
mod utils;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("TWF (Terminal Wallpaper Fit) v{}", env!("CARGO_PKG_VERSION"));
    
    // TODO: CLI引数のパースと処理フローの実装
    
    Ok(())
}
