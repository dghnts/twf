// エラー型のテスト

use twf::models::{TwfError, Result};
use std::path::PathBuf;

#[test]
fn test_error_display_messages() {
    // 各エラーケースのメッセージが適切に表示されることを確認
    
    let error = TwfError::ImageLoadError("test.png".to_string());
    let message = format!("{}", error);
    assert!(message.contains("画像の読み込みに失敗しました"));
    assert!(message.contains("解決方法"));
    
    let error = TwfError::ImageAnalysisError("解析失敗".to_string());
    let message = format!("{}", error);
    assert!(message.contains("画像の解析に失敗しました"));
    
    let error = TwfError::TerminalDetectionError;
    let message = format!("{}", error);
    assert!(message.contains("ターミナルタイプを判定できませんでした"));
    assert!(message.contains("--image"));
    assert!(message.contains("--color"));
    
    let error = TwfError::BackgroundColorDetectionError;
    let message = format!("{}", error);
    assert!(message.contains("背景色が検出できませんでした"));
    
    let error = TwfError::InsufficientContrast { actual: 3.5, required: 4.5 };
    let message = format!("{}", error);
    assert!(message.contains("コントラスト比が不足しています"));
    assert!(message.contains("3.50"));
    assert!(message.contains("4.50"));
    
    let error = TwfError::ShellDetectionError;
    let message = format!("{}", error);
    assert!(message.contains("シェルタイプを判定できませんでした"));
    
    let error = TwfError::ConfigFileNotFound(PathBuf::from("/path/to/config"));
    let message = format!("{}", error);
    assert!(message.contains("設定ファイルが見つかりません"));
}

#[test]
fn test_error_conversion() {
    // std::io::ErrorからTwfErrorへの変換が正しく動作することを確認
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let twf_error: TwfError = io_error.into();
    
    match twf_error {
        TwfError::IoError(_) => {
            // 正しく変換された
        }
        _ => panic!("Expected IoError variant"),
    }
}

#[test]
fn test_result_type() {
    // Result型のエイリアスが正しく動作することを確認
    fn test_function() -> Result<i32> {
        Ok(42)
    }
    
    let result = test_function();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    fn test_error_function() -> Result<i32> {
        Err(TwfError::TerminalDetectionError)
    }
    
    let result = test_error_function();
    assert!(result.is_err());
}

#[test]
fn test_error_messages_contain_japanese() {
    // すべてのエラーメッセージが日本語を含むことを確認（Requirements 2.7.7）
    let errors = vec![
        TwfError::ImageLoadError("test".to_string()),
        TwfError::ImageAnalysisError("test".to_string()),
        TwfError::ConfigParseError("test".to_string()),
        TwfError::TerminalDetectionError,
        TwfError::BackgroundColorDetectionError,
        TwfError::ContrastCalculationError("test".to_string()),
        TwfError::ConfigApplyError("test".to_string()),
        TwfError::BackupError("test".to_string()),
        TwfError::RollbackError("test".to_string()),
        TwfError::ColorConversionError("test".to_string()),
        TwfError::InsufficientContrast { actual: 3.0, required: 4.5 },
        TwfError::ParseError("test".to_string()),
        TwfError::ShellDetectionError,
        TwfError::ConfigFileNotFound(PathBuf::from("/test")),
    ];
    
    for error in errors {
        let message = format!("{}", error);
        // 日本語文字が含まれていることを確認（ひらがな、カタカナ、漢字のいずれか）
        assert!(
            message.chars().any(|c| {
                ('\u{3040}'..='\u{309F}').contains(&c) || // ひらがな
                ('\u{30A0}'..='\u{30FF}').contains(&c) || // カタカナ
                ('\u{4E00}'..='\u{9FFF}').contains(&c)    // 漢字
            }),
            "Error message should contain Japanese characters: {}",
            message
        );
    }
}

#[test]
fn test_error_messages_contain_solutions() {
    // 主要なエラーメッセージに解決方法が含まれることを確認
    let errors_with_solutions = vec![
        TwfError::ImageLoadError("test".to_string()),
        TwfError::ImageAnalysisError("test".to_string()),
        TwfError::TerminalDetectionError,
        TwfError::BackgroundColorDetectionError,
        TwfError::ConfigApplyError("test".to_string()),
        TwfError::BackupError("test".to_string()),
        TwfError::RollbackError("test".to_string()),
        TwfError::InsufficientContrast { actual: 3.0, required: 4.5 },
        TwfError::ShellDetectionError,
        TwfError::ConfigFileNotFound(PathBuf::from("/test")),
    ];
    
    for error in errors_with_solutions {
        let message = format!("{}", error);
        assert!(
            message.contains("解決方法") || message.contains("原因"),
            "Error message should contain solution or cause: {}",
            message
        );
    }
}
