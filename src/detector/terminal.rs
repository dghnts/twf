// ターミナル種別判定

use crate::models::TerminalType;
use std::env;

/// 現在のターミナルエミュレータを判定
/// 
/// 環境変数をチェックして、実行中のターミナルエミュレータを特定します。
/// 優先度順に判定を行い、判定できない場合はUnknownを返します。
/// 
/// # 判定に使用する環境変数
/// - TERM_PROGRAM: iTerm2、WezTermの判定に使用
/// - ALACRITTY_SOCKET, ALACRITTY_LOG: Alacrittyの判定に使用
/// - WT_SESSION: Windows Terminalの判定に使用
/// - GNOME_TERMINAL_SERVICE: GNOME Terminalの判定に使用
/// - KITTY_WINDOW_ID: Kittyの判定に使用
/// 
/// # 戻り値
/// 検出されたターミナルタイプ。判定できない場合は`TerminalType::Unknown`
/// 
/// # 例
/// ```
/// use twf::detector::terminal::detect_terminal;
/// 
/// let terminal_type = detect_terminal();
/// println!("検出されたターミナル: {:?}", terminal_type);
/// ```
pub fn detect_terminal() -> TerminalType {
    // TERM_PROGRAMをチェック（iTerm2、WezTerm）
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            "iTerm.app" => return TerminalType::ITerm2,
            "WezTerm" => return TerminalType::WezTerm,
            _ => {}
        }
    }
    
    // Alacrittyをチェック
    if env::var("ALACRITTY_SOCKET").is_ok() || env::var("ALACRITTY_LOG").is_ok() {
        return TerminalType::Alacritty;
    }
    
    // Windows Terminalをチェック
    if env::var("WT_SESSION").is_ok() {
        return TerminalType::WindowsTerminal;
    }
    
    // GNOME Terminalをチェック
    if env::var("GNOME_TERMINAL_SERVICE").is_ok() {
        return TerminalType::GnomeTerminal;
    }
    
    // Kittyをチェック
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return TerminalType::Kitty;
    }
    
    // 判定できない場合はUnknown
    TerminalType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    /// テスト用に環境変数を設定してターミナル判定をテストするヘルパー関数
    fn test_with_env(key: &str, value: &str, expected: TerminalType) {
        // すべての関連環境変数を保存
        let vars_to_save = [
            "TERM_PROGRAM",
            "ALACRITTY_SOCKET",
            "ALACRITTY_LOG",
            "WT_SESSION",
            "GNOME_TERMINAL_SERVICE",
            "KITTY_WINDOW_ID",
        ];
        
        let originals: Vec<_> = vars_to_save
            .iter()
            .map(|&k| (k, env::var(k).ok()))
            .collect();
        
        // すべての環境変数をクリア
        for &k in &vars_to_save {
            env::remove_var(k);
        }
        
        // テスト用の環境変数を設定
        env::set_var(key, value);
        
        // ターミナル判定を実行
        let result = detect_terminal();
        
        // 結果を検証
        assert_eq!(result, expected, "環境変数 {}={} でターミナルタイプが正しく判定されませんでした", key, value);
        
        // すべての環境変数を元に戻す
        for (k, original) in originals {
            match original {
                Some(val) => env::set_var(k, val),
                None => env::remove_var(k),
            }
        }
    }
    
    #[test]
    fn test_detect_iterm2() {
        test_with_env("TERM_PROGRAM", "iTerm.app", TerminalType::ITerm2);
    }
    
    #[test]
    fn test_detect_wezterm() {
        test_with_env("TERM_PROGRAM", "WezTerm", TerminalType::WezTerm);
    }
    
    #[test]
    fn test_detect_alacritty_socket() {
        test_with_env("ALACRITTY_SOCKET", "/tmp/alacritty.sock", TerminalType::Alacritty);
    }
    
    #[test]
    fn test_detect_alacritty_log() {
        test_with_env("ALACRITTY_LOG", "/tmp/alacritty.log", TerminalType::Alacritty);
    }
    
    #[test]
    fn test_detect_windows_terminal() {
        test_with_env("WT_SESSION", "12345678-1234-1234-1234-123456789012", TerminalType::WindowsTerminal);
    }
    
    #[test]
    fn test_detect_gnome_terminal() {
        test_with_env("GNOME_TERMINAL_SERVICE", ":1.234", TerminalType::GnomeTerminal);
    }
    
    #[test]
    fn test_detect_kitty() {
        test_with_env("KITTY_WINDOW_ID", "1", TerminalType::Kitty);
    }
    
    #[test]
    fn test_detect_unknown() {
        // すべての関連環境変数を削除
        let vars_to_remove = [
            "TERM_PROGRAM",
            "ALACRITTY_SOCKET",
            "ALACRITTY_LOG",
            "WT_SESSION",
            "GNOME_TERMINAL_SERVICE",
            "KITTY_WINDOW_ID",
        ];
        
        // 既存の環境変数を保存
        let originals: Vec<_> = vars_to_remove
            .iter()
            .map(|&key| (key, env::var(key).ok()))
            .collect();
        
        // すべての環境変数を削除
        for &key in &vars_to_remove {
            env::remove_var(key);
        }
        
        // ターミナル判定を実行
        let result = detect_terminal();
        assert_eq!(result, TerminalType::Unknown, "環境変数が設定されていない場合、Unknownを返すべきです");
        
        // 環境変数を元に戻す
        for (key, original) in originals {
            match original {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
        }
    }
    
    #[test]
    fn test_priority_iterm2_over_others() {
        // iTerm2が最優先であることを確認
        // 複数の環境変数が設定されている場合でも、iTerm2が優先される
        env::set_var("TERM_PROGRAM", "iTerm.app");
        env::set_var("KITTY_WINDOW_ID", "1");
        
        let result = detect_terminal();
        assert_eq!(result, TerminalType::ITerm2, "TERM_PROGRAM=iTerm.appが最優先されるべきです");
        
        // クリーンアップ
        env::remove_var("TERM_PROGRAM");
        env::remove_var("KITTY_WINDOW_ID");
    }
}
