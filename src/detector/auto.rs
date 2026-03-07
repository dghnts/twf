// 自動検出ロジック

use crate::models::{Result, TerminalType};
use std::path::PathBuf;

use super::terminal::detect_terminal;
use super::iterm2::detect_iterm2_background;
use super::alacritty::detect_alacritty_background;
use super::windows_terminal::detect_windows_terminal_background;
use super::gnome_terminal::detect_gnome_terminal_background;
use super::kitty::detect_kitty_background;
use super::wezterm::detect_wezterm_background;

/// 自動検出器
///
/// ターミナルエミュレータの種類を判定し、適切な検出関数を呼び出して
/// 背景画像のパスを自動検出します。
///
/// # Requirements
/// - 2.8.1: ターミナルエミュレータの設定ファイルから背景画像パスを自動検出できること
/// - 2.8.4: 現在実行中のターミナルエミュレータを自動判定できること
pub struct AutoDetector {
    /// ターミナルタイプ
    terminal_type: TerminalType,
}

impl AutoDetector {
    /// 新しいAutoDetectorインスタンスを作成
    ///
    /// 現在のターミナルタイプを自動判定してインスタンスを初期化します。
    ///
    /// # Returns
    /// 初期化されたAutoDetectorインスタンス
    ///
    /// # 例
    /// ```
    /// use twf::detector::auto::AutoDetector;
    ///
    /// let detector = AutoDetector::new();
    /// println!("検出されたターミナル: {:?}", detector.terminal_type());
    /// ```
    pub fn new() -> Self {
        let terminal_type = detect_terminal();
        Self { terminal_type }
    }
    
    /// ターミナルタイプを取得
    ///
    /// # Returns
    /// 検出されたターミナルタイプ
    pub fn terminal_type(&self) -> TerminalType {
        self.terminal_type
    }
    
    /// 背景画像パスを自動検出
    ///
    /// ターミナルタイプに応じて適切な検出関数を呼び出し、
    /// 背景画像のパスを取得します。
    ///
    /// # 処理フロー
    /// 1. ターミナルタイプを確認
    /// 2. ターミナルタイプに応じて適切な検出関数を呼び出し
    ///    - iTerm2 → detect_iterm2_background
    ///    - Alacritty → detect_alacritty_background
    ///    - Windows Terminal → detect_windows_terminal_background
    ///    - GNOME Terminal → detect_gnome_terminal_background
    ///    - Kitty → detect_kitty_background
    ///    - WezTerm → detect_wezterm_background
    ///    - Unknown → None
    /// 3. 検出結果を返す
    ///
    /// # Returns
    /// - `Ok(Some(PathBuf))`: 背景画像パスが検出された場合
    /// - `Ok(None)`: 背景画像が設定されていない、またはターミナルタイプが不明な場合
    /// - `Err(TwfError)`: 検出処理中にエラーが発生した場合
    ///
    /// # Requirements
    /// - 2.8.1: ターミナルエミュレータの設定ファイルから背景画像パスを自動検出できること
    /// - 2.8.4: ターミナルタイプに応じて適切な検出関数を呼び出すこと
    ///
    /// # 例
    /// ```no_run
    /// use twf::detector::auto::AutoDetector;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = AutoDetector::new();
    /// let bg_image = detector.detect_background_image().await?;
    ///
    /// match bg_image {
    ///     Some(path) => println!("背景画像が見つかりました: {:?}", path),
    ///     None => println!("背景画像が見つかりませんでした"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_background_image(&self) -> Result<Option<PathBuf>> {
        match self.terminal_type {
            TerminalType::ITerm2 => {
                detect_iterm2_background().await
            }
            TerminalType::Alacritty => {
                detect_alacritty_background().await
            }
            TerminalType::WindowsTerminal => {
                detect_windows_terminal_background().await
            }
            TerminalType::GnomeTerminal => {
                detect_gnome_terminal_background().await
            }
            TerminalType::Kitty => {
                detect_kitty_background().await
            }
            TerminalType::WezTerm => {
                detect_wezterm_background().await
            }
            TerminalType::Unknown => {
                // ターミナルタイプが不明な場合はNoneを返す
                Ok(None)
            }
        }
    }
}

impl Default for AutoDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auto_detector_new() {
        // AutoDetectorのインスタンス化をテスト
        let detector = AutoDetector::new();
        
        // ターミナルタイプが取得できることを確認
        let terminal_type = detector.terminal_type();
        
        // ターミナルタイプが有効な値であることを確認
        // （実際の値は環境に依存するため、Unknownを含むすべての値が有効）
        println!("検出されたターミナルタイプ: {:?}", terminal_type);
    }
    
    #[test]
    fn test_auto_detector_default() {
        // Defaultトレイトの実装をテスト
        let detector = AutoDetector::default();
        let terminal_type = detector.terminal_type();
        
        println!("デフォルトで検出されたターミナルタイプ: {:?}", terminal_type);
    }
    
    #[tokio::test]
    async fn test_detect_background_image_unknown_terminal() {
        // Unknownターミナルの場合、Noneが返されることを確認
        let detector = AutoDetector {
            terminal_type: TerminalType::Unknown,
        };
        
        let result = detector.detect_background_image().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
    
    #[tokio::test]
    async fn test_detect_background_image_all_terminal_types() {
        // すべてのターミナルタイプで検出関数が呼び出せることを確認
        let terminal_types = vec![
            TerminalType::ITerm2,
            TerminalType::Alacritty,
            TerminalType::WindowsTerminal,
            TerminalType::GnomeTerminal,
            TerminalType::Kitty,
            TerminalType::WezTerm,
            TerminalType::Unknown,
        ];
        
        for terminal_type in terminal_types {
            let detector = AutoDetector { terminal_type };
            let result = detector.detect_background_image().await;
            
            // エラーが発生した場合は、エラーメッセージを表示
            match result {
                Ok(path_opt) => {
                    println!("ターミナルタイプ {:?}: {:?}", terminal_type, path_opt);
                }
                Err(e) => {
                    // Windows Terminalなど、環境依存のエラーは許容する
                    println!("ターミナルタイプ {:?}: エラー（環境依存）: {}", terminal_type, e);
                }
            }
        }
    }
    
    #[tokio::test]
    async fn test_detect_background_image_integration() {
        // 実際の環境での統合テスト
        let detector = AutoDetector::new();
        let result = detector.detect_background_image().await;
        
        // エラーが発生しないことを確認
        assert!(result.is_ok());
        
        match result.unwrap() {
            Some(path) => {
                println!("背景画像が検出されました: {:?}", path);
                // パスが存在することを確認
                assert!(path.exists(), "検出されたパスが存在しません: {:?}", path);
            }
            None => {
                println!("背景画像が検出されませんでした（正常な動作）");
            }
        }
    }
}
