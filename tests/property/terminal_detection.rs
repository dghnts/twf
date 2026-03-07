// Property 10: ターミナルタイプの判定
//
// このプロパティテストは、ターミナルタイプ判定機能の正確性を検証します。
// 任意の環境変数の組み合わせに対して、既知のターミナル（iTerm2、Alacritty、
// Windows Terminal、GNOME Terminal、Kitty、WezTerm）のいずれか、
// または`Unknown`が返されることを確認します。
//
// **Validates: Requirements 2.8.3**

use proptest::prelude::*;
use serial_test::serial;
use std::env;
use twf::detector::terminal::detect_terminal;
use twf::models::TerminalType;

/// テスト用の環境変数名
const ENV_VARS: &[&str] = &[
    "TERM_PROGRAM",
    "ALACRITTY_SOCKET",
    "ALACRITTY_LOG",
    "WT_SESSION",
    "GNOME_TERMINAL_SERVICE",
    "KITTY_WINDOW_ID",
];

/// 環境変数を保存する構造体
struct EnvBackup {
    vars: Vec<(String, Option<String>)>,
}

impl EnvBackup {
    /// 現在の環境変数を保存
    fn new() -> Self {
        let vars = ENV_VARS
            .iter()
            .map(|&key| (key.to_string(), env::var(key).ok()))
            .collect();
        Self { vars }
    }

    /// 環境変数を復元
    fn restore(&self) {
        for (key, value) in &self.vars {
            match value {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
        }
    }
}

/// すべての環境変数をクリア
fn clear_all_env_vars() {
    for &key in ENV_VARS {
        env::remove_var(key);
    }
}

/// 環境変数を設定してターミナルタイプを検証するヘルパー関数
fn test_terminal_detection(key: &str, value: &str, expected: TerminalType) {
    let backup = EnvBackup::new();
    clear_all_env_vars();
    
    env::set_var(key, value);
    let result = detect_terminal();
    
    backup.restore();
    
    assert_eq!(
        result, expected,
        "環境変数 {}={} でターミナルタイプが正しく判定されませんでした。期待: {:?}, 実際: {:?}",
        key, value, expected, result
    );
}

// Property 10.1: iTerm2の判定
//
// TERM_PROGRAM="iTerm.app"が設定されている場合、
// 必ずITerm2を返すことを検証します。
#[test]
#[serial]
fn prop_detect_iterm2() {
    test_terminal_detection("TERM_PROGRAM", "iTerm.app", TerminalType::ITerm2);
}

// Property 10.2: WezTermの判定
//
// TERM_PROGRAM="WezTerm"が設定されている場合、
// 必ずWezTermを返すことを検証します。
#[test]
#[serial]
fn prop_detect_wezterm() {
    test_terminal_detection("TERM_PROGRAM", "WezTerm", TerminalType::WezTerm);
}

// Property 10.3: Alacrittyの判定（ALACRITTY_SOCKET）
//
// ALACRITTY_SOCKETが設定されている場合、
// 必ずAlacrittyを返すことを検証します。
#[test]
#[serial]
fn prop_detect_alacritty_socket() {
    test_terminal_detection("ALACRITTY_SOCKET", "/tmp/alacritty.sock", TerminalType::Alacritty);
}

// Property 10.4: Alacrittyの判定（ALACRITTY_LOG）
//
// ALACRITTY_LOGが設定されている場合、
// 必ずAlacrittyを返すことを検証します。
#[test]
#[serial]
fn prop_detect_alacritty_log() {
    test_terminal_detection("ALACRITTY_LOG", "/tmp/alacritty.log", TerminalType::Alacritty);
}

// Property 10.5: Windows Terminalの判定
//
// WT_SESSIONが設定されている場合、
// 必ずWindowsTerminalを返すことを検証します。
#[test]
#[serial]
fn prop_detect_windows_terminal() {
    test_terminal_detection(
        "WT_SESSION",
        "12345678-1234-1234-1234-123456789012",
        TerminalType::WindowsTerminal,
    );
}

// Property 10.6: GNOME Terminalの判定
//
// GNOME_TERMINAL_SERVICEが設定されている場合、
// 必ずGnomeTerminalを返すことを検証します。
#[test]
#[serial]
fn prop_detect_gnome_terminal() {
    test_terminal_detection(
        "GNOME_TERMINAL_SERVICE",
        ":1.234",
        TerminalType::GnomeTerminal,
    );
}

// Property 10.7: Kittyの判定
//
// KITTY_WINDOW_IDが設定されている場合、
// 必ずKittyを返すことを検証します。
#[test]
#[serial]
fn prop_detect_kitty() {
    test_terminal_detection("KITTY_WINDOW_ID", "1", TerminalType::Kitty);
}

// Property 10.8: 環境変数が設定されていない場合
//
// すべての環境変数が設定されていない場合、
// 必ずUnknownを返すことを検証します。
#[test]
#[serial]
fn prop_detect_unknown() {
    let backup = EnvBackup::new();
    clear_all_env_vars();
    
    let result = detect_terminal();
    
    backup.restore();
    
    assert_eq!(
        result,
        TerminalType::Unknown,
        "環境変数が設定されていない場合、Unknownを返すべきです"
    );
}

// Property 10.9: 優先順位の検証（iTerm2が最優先）
//
// 複数の環境変数が設定されている場合でも、
// TERM_PROGRAM="iTerm.app"が最優先されることを検証します。
#[test]
#[serial]
fn prop_priority_iterm2() {
    let backup = EnvBackup::new();
    clear_all_env_vars();
    
    // iTerm2と他のターミナルの環境変数を同時に設定
    env::set_var("TERM_PROGRAM", "iTerm.app");
    env::set_var("KITTY_WINDOW_ID", "1");
    env::set_var("ALACRITTY_SOCKET", "/tmp/alacritty.sock");
    
    let result = detect_terminal();
    
    backup.restore();
    
    assert_eq!(
        result,
        TerminalType::ITerm2,
        "TERM_PROGRAM=iTerm.appが最優先されるべきです"
    );
}

// Property 10.10: 優先順位の検証（WezTermがiTerm2の次）
//
// TERM_PROGRAM="WezTerm"と他の環境変数が設定されている場合、
// WezTermが優先されることを検証します。
#[test]
#[serial]
fn prop_priority_wezterm() {
    let backup = EnvBackup::new();
    clear_all_env_vars();
    
    // WezTermと他のターミナルの環境変数を同時に設定
    env::set_var("TERM_PROGRAM", "WezTerm");
    env::set_var("KITTY_WINDOW_ID", "1");
    env::set_var("ALACRITTY_SOCKET", "/tmp/alacritty.sock");
    
    let result = detect_terminal();
    
    backup.restore();
    
    assert_eq!(
        result,
        TerminalType::WezTerm,
        "TERM_PROGRAM=WezTermが優先されるべきです"
    );
}

// Property 10.11: 優先順位の検証（Alacrittyの優先順位）
//
// AlacrittyとWindows Terminal、GNOME Terminal、Kittyの環境変数が
// 同時に設定されている場合、Alacrittyが優先されることを検証します。
#[test]
#[serial]
fn prop_priority_alacritty() {
    let backup = EnvBackup::new();
    clear_all_env_vars();
    
    // Alacrittyと他のターミナルの環境変数を同時に設定
    env::set_var("ALACRITTY_SOCKET", "/tmp/alacritty.sock");
    env::set_var("WT_SESSION", "12345678-1234-1234-1234-123456789012");
    env::set_var("GNOME_TERMINAL_SERVICE", ":1.234");
    env::set_var("KITTY_WINDOW_ID", "1");
    
    let result = detect_terminal();
    
    backup.restore();
    
    assert_eq!(
        result,
        TerminalType::Alacritty,
        "Alacrittyが優先されるべきです"
    );
}

// Property 10.12: 優先順位の検証（Windows Terminalの優先順位）
//
// Windows TerminalとGNOME Terminal、Kittyの環境変数が
// 同時に設定されている場合、Windows Terminalが優先されることを検証します。
#[test]
#[serial]
fn prop_priority_windows_terminal() {
    let backup = EnvBackup::new();
    clear_all_env_vars();
    
    // Windows Terminalと他のターミナルの環境変数を同時に設定
    env::set_var("WT_SESSION", "12345678-1234-1234-1234-123456789012");
    env::set_var("GNOME_TERMINAL_SERVICE", ":1.234");
    env::set_var("KITTY_WINDOW_ID", "1");
    
    let result = detect_terminal();
    
    backup.restore();
    
    assert_eq!(
        result,
        TerminalType::WindowsTerminal,
        "Windows Terminalが優先されるべきです"
    );
}

// Property 10.13: 優先順位の検証（GNOME Terminalの優先順位）
//
// GNOME TerminalとKittyの環境変数が同時に設定されている場合、
// GNOME Terminalが優先されることを検証します。
#[test]
#[serial]
fn prop_priority_gnome_terminal() {
    let backup = EnvBackup::new();
    clear_all_env_vars();
    
    // GNOME TerminalとKittyの環境変数を同時に設定
    env::set_var("GNOME_TERMINAL_SERVICE", ":1.234");
    env::set_var("KITTY_WINDOW_ID", "1");
    
    let result = detect_terminal();
    
    backup.restore();
    
    assert_eq!(
        result,
        TerminalType::GnomeTerminal,
        "GNOME Terminalが優先されるべきです"
    );
}

proptest! {
    /// Property 10.14: 任意のTERM_PROGRAM値に対する判定
    ///
    /// 任意のTERM_PROGRAM値に対して、detect_terminal()が
    /// 有効なTerminalTypeを返すことを検証します。
    /// iTerm.appとWezTerm以外の値の場合、他の環境変数が
    /// 設定されていなければUnknownを返すべきです。
    #[test]
    #[serial]
    fn prop_arbitrary_term_program(term_program in "[a-zA-Z0-9._-]{1,50}") {
        let backup = EnvBackup::new();
        clear_all_env_vars();
        
        env::set_var("TERM_PROGRAM", &term_program);
        let result = detect_terminal();
        
        backup.restore();
        
        // 結果が有効なTerminalTypeであることを確認
        match term_program.as_str() {
            "iTerm.app" => prop_assert_eq!(result, TerminalType::ITerm2),
            "WezTerm" => prop_assert_eq!(result, TerminalType::WezTerm),
            _ => prop_assert_eq!(result, TerminalType::Unknown),
        }
    }

    /// Property 10.15: 任意の環境変数値に対する判定
    ///
    /// 任意の環境変数値が設定されている場合でも、
    /// detect_terminal()が必ず有効なTerminalTypeを返すことを検証します。
    /// パニックやエラーが発生しないことを確認します。
    #[test]
    #[serial]
    fn prop_arbitrary_env_values(
        alacritty_socket in "[a-zA-Z0-9/_.-]{0,100}",
        wt_session in "[a-zA-Z0-9-]{0,50}",
        gnome_service in "[a-zA-Z0-9.:_-]{0,50}",
        kitty_id in "[0-9]{0,10}"
    ) {
        let backup = EnvBackup::new();
        clear_all_env_vars();
        
        // 任意の値を設定
        if !alacritty_socket.is_empty() {
            env::set_var("ALACRITTY_SOCKET", &alacritty_socket);
        }
        if !wt_session.is_empty() {
            env::set_var("WT_SESSION", &wt_session);
        }
        if !gnome_service.is_empty() {
            env::set_var("GNOME_TERMINAL_SERVICE", &gnome_service);
        }
        if !kitty_id.is_empty() {
            env::set_var("KITTY_WINDOW_ID", &kitty_id);
        }
        
        // パニックせずに結果を返すことを確認
        let result = detect_terminal();
        
        backup.restore();
        
        // 結果が有効なTerminalTypeであることを確認
        prop_assert!(
            matches!(
                result,
                TerminalType::ITerm2
                    | TerminalType::Alacritty
                    | TerminalType::WindowsTerminal
                    | TerminalType::GnomeTerminal
                    | TerminalType::Kitty
                    | TerminalType::WezTerm
                    | TerminalType::Unknown
            ),
            "無効なTerminalTypeが返されました: {:?}",
            result
        );
    }

    /// Property 10.16: 環境変数の存在チェックの正確性
    ///
    /// 環境変数が設定されている場合と設定されていない場合で、
    /// 判定結果が適切に変わることを検証します。
    #[test]
    #[serial]
    fn prop_env_var_presence(set_alacritty in proptest::bool::ANY) {
        let backup = EnvBackup::new();
        clear_all_env_vars();
        
        if set_alacritty {
            env::set_var("ALACRITTY_SOCKET", "/tmp/test.sock");
        }
        
        let result = detect_terminal();
        
        backup.restore();
        
        if set_alacritty {
            prop_assert_eq!(
                result,
                TerminalType::Alacritty,
                "ALACRITTY_SOCKETが設定されている場合、Alacrittyを返すべきです"
            );
        } else {
            prop_assert_eq!(
                result,
                TerminalType::Unknown,
                "環境変数が設定されていない場合、Unknownを返すべきです"
            );
        }
    }
}
