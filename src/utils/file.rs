// ファイル操作ユーティリティ

use crate::models::{Result, TwfError};
use std::io::{self, Write};
use std::path::Path;
use tokio::fs;

/// ファイルに追記
/// 
/// 指定されたファイルに内容を追記します。
/// ファイルが存在しない場合は新規作成します。
/// 
/// # Arguments
/// * `path` - 追記対象のファイルパス
/// * `content` - 追記する内容
/// 
/// # Returns
/// * `Result<()>` - 成功時はOk(())、失敗時はエラー
pub async fn append_to_file(path: &Path, content: &str) -> Result<()> {
    // ファイルが存在しない場合は親ディレクトリを作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| TwfError::FileOperationError(format!("ディレクトリの作成に失敗: {}", e)))?;
    }

    // ファイルの内容を読み込む（存在しない場合は空文字列）
    let mut existing_content = if path.exists() {
        fs::read_to_string(path)
            .await
            .map_err(|e| TwfError::FileOperationError(format!("ファイルの読み込みに失敗: {}", e)))?
    } else {
        String::new()
    };

    // 既存の内容が改行で終わっていない場合は改行を追加
    if !existing_content.is_empty() && !existing_content.ends_with('\n') {
        existing_content.push('\n');
    }

    // 新しい内容を追加
    existing_content.push_str(content);

    // ファイルに書き込み
    fs::write(path, existing_content)
        .await
        .map_err(|e| TwfError::FileOperationError(format!("ファイルの書き込みに失敗: {}", e)))?;

    Ok(())
}

/// 既存のTWFブロックを削除
/// 
/// ファイルから既存のTWF設定ブロックを削除します。
/// "=== TWF Generated Config ===" から "=== End TWF Config ===" までのブロックを削除します。
/// 
/// # Arguments
/// * `path` - 対象のファイルパス
/// 
/// # Returns
/// * `Result<()>` - 成功時はOk(())、失敗時はエラー
pub async fn remove_existing_twf_block(path: &Path) -> Result<()> {
    // ファイルが存在しない場合は何もしない
    if !path.exists() {
        return Ok(());
    }

    // ファイルの内容を読み込む
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| TwfError::FileOperationError(format!("ファイルの読み込みに失敗: {}", e)))?;

    // TWFブロックを削除
    let mut new_lines: Vec<&str> = Vec::new();
    let mut in_twf_block = false;

    for line in content.lines() {
        if line.contains("=== TWF Generated Config ===") {
            in_twf_block = true;
            continue;
        }

        if line.contains("=== End TWF Config ===") {
            in_twf_block = false;
            continue;
        }

        if !in_twf_block {
            new_lines.push(line);
        }
    }

    // 新しい内容をファイルに書き込み
    let new_content = new_lines.join("\n");
    fs::write(path, new_content.as_bytes())
        .await
        .map_err(|e| TwfError::FileOperationError(format!("ファイルの書き込みに失敗: {}", e)))?;

    Ok(())
}

/// ユーザー確認
/// 
/// ユーザーに設定の適用を確認します。
/// 
/// # Returns
/// * `Result<bool>` - ユーザーがYesを選択した場合はtrue、Noを選択した場合はfalse
pub fn confirm_apply() -> Result<bool> {
    print!("設定をシェル設定ファイルに適用しますか？ (y/n): ");
    io::stdout()
        .flush()
        .map_err(|e| TwfError::FileOperationError(format!("標準出力のフラッシュに失敗: {}", e)))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| TwfError::FileOperationError(format!("標準入力の読み込みに失敗: {}", e)))?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_append_to_file_new_file() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // 新しいファイルに追記
        append_to_file(&test_file, "Hello, World!").await.unwrap();

        // ファイルの内容を確認
        let content = fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_append_to_file_existing_file() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // 既存のファイルを作成
        fs::write(&test_file, "Line 1").await.unwrap();

        // ファイルに追記
        append_to_file(&test_file, "Line 2").await.unwrap();

        // ファイルの内容を確認
        let content = fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(content, "Line 1\nLine 2");
    }

    #[tokio::test]
    async fn test_remove_existing_twf_block() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // TWFブロックを含むファイルを作成
        let content = r#"# User config
export PATH=/usr/local/bin:$PATH

# === TWF Generated Config ===
export TWF_ANSI_RED='\e[38;2;255;0;0m'
export TWF_ANSI_GREEN='\e[38;2;0;255;0m'
# === End TWF Config ===

# More user config
alias ll='ls -la'
"#;
        fs::write(&test_file, content).await.unwrap();

        // TWFブロックを削除
        remove_existing_twf_block(&test_file).await.unwrap();

        // ファイルの内容を確認
        let new_content = fs::read_to_string(&test_file).await.unwrap();
        assert!(!new_content.contains("TWF Generated Config"));
        assert!(!new_content.contains("TWF_ANSI_RED"));
        assert!(new_content.contains("User config"));
        assert!(new_content.contains("More user config"));
    }

    #[tokio::test]
    async fn test_remove_existing_twf_block_no_block() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // TWFブロックを含まないファイルを作成
        let content = "# User config\nexport PATH=/usr/local/bin:$PATH\n";
        fs::write(&test_file, content).await.unwrap();

        // TWFブロックを削除（何も変更されないはず）
        remove_existing_twf_block(&test_file).await.unwrap();

        // ファイルの内容を確認（末尾の改行は削除される可能性がある）
        let new_content = fs::read_to_string(&test_file).await.unwrap();
        // 末尾の改行を除いて比較
        assert_eq!(new_content.trim_end(), content.trim_end());
    }
}
