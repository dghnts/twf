// ロールバック処理

use crate::models::{BackupInfo, Result, TwfError};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

/// バックアップから設定を復元
/// 
/// BackupInfoを受け取り、バックアップファイルを元の場所に復元します。
/// 復元前にチェックサムを検証して、バックアップファイルの整合性を確認します。
/// 
/// # Arguments
/// * `backup_info` - バックアップ情報
/// 
/// # Returns
/// * `Result<()>` - 成功した場合はOk(())、失敗した場合はエラー
/// 
/// # Errors
/// * バックアップファイルが存在しない場合
/// * チェックサムが一致しない場合（バックアップファイルが破損している）
/// * ファイルのコピーに失敗した場合
pub async fn rollback(backup_info: &BackupInfo) -> Result<()> {
    // 1. バックアップファイルが存在するか確認
    if !backup_info.backup_path.exists() {
        return Err(TwfError::RollbackError(format!(
            "バックアップファイルが見つかりません: {}",
            backup_info.backup_path.display()
        )));
    }

    // 2. バックアップファイルのチェックサムを検証
    let current_checksum = calculate_checksum(&backup_info.backup_path).await?;
    if current_checksum != backup_info.checksum {
        return Err(TwfError::RollbackError(format!(
            "バックアップファイルのチェックサムが一致しません。ファイルが破損している可能性があります。\n期待値: {}\n実際の値: {}",
            backup_info.checksum,
            current_checksum
        )));
    }

    // 3. バックアップファイルを元の場所にコピー
    fs::copy(&backup_info.backup_path, &backup_info.original_path)
        .await
        .map_err(|e| {
            TwfError::RollbackError(format!(
                "バックアップファイルの復元に失敗しました: {}",
                e
            ))
        })?;

    Ok(())
}

/// ファイルのチェックサムを計算（SHA-256）
async fn calculate_checksum(path: &Path) -> Result<String> {
    let content = fs::read(path).await.map_err(|e| {
        TwfError::RollbackError(format!("ファイルの読み込みに失敗: {}", e))
    })?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BackupInfo;
    use chrono::Utc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_rollback_success() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let original_path = temp_dir.path().join("config.txt");
        let backup_path = temp_dir.path().join("backup.txt");

        // 元のファイルを作成
        let original_content = "original content";
        fs::write(&original_path, original_content)
            .await
            .unwrap();

        // バックアップファイルを作成
        let backup_content = "backup content";
        fs::write(&backup_path, backup_content).await.unwrap();

        // チェックサムを計算
        let checksum = calculate_checksum(&backup_path).await.unwrap();

        // BackupInfoを作成
        let backup_info = BackupInfo {
            original_path: original_path.clone(),
            backup_path: backup_path.clone(),
            timestamp: Utc::now(),
            checksum,
        };

        // 元のファイルを変更
        fs::write(&original_path, "modified content")
            .await
            .unwrap();

        // ロールバックを実行
        let result = rollback(&backup_info).await;
        assert!(result.is_ok());

        // 元のファイルがバックアップの内容に復元されたことを確認
        let restored_content = fs::read_to_string(&original_path).await.unwrap();
        assert_eq!(restored_content, backup_content);
    }

    #[tokio::test]
    async fn test_rollback_backup_file_not_found() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let original_path = temp_dir.path().join("config.txt");
        let backup_path = temp_dir.path().join("nonexistent_backup.txt");

        // BackupInfoを作成（存在しないバックアップファイル）
        let backup_info = BackupInfo {
            original_path,
            backup_path,
            timestamp: Utc::now(),
            checksum: "dummy_checksum".to_string(),
        };

        // ロールバックを実行
        let result = rollback(&backup_info).await;

        // エラーが返されることを確認
        assert!(result.is_err());
        match result {
            Err(TwfError::RollbackError(msg)) => {
                assert!(msg.contains("バックアップファイルが見つかりません"));
            }
            _ => panic!("Expected RollbackError"),
        }
    }

    #[tokio::test]
    async fn test_rollback_checksum_mismatch() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let original_path = temp_dir.path().join("config.txt");
        let backup_path = temp_dir.path().join("backup.txt");

        // 元のファイルを作成
        fs::write(&original_path, "original content")
            .await
            .unwrap();

        // バックアップファイルを作成
        fs::write(&backup_path, "backup content").await.unwrap();

        // BackupInfoを作成（間違ったチェックサム）
        let backup_info = BackupInfo {
            original_path,
            backup_path: backup_path.clone(),
            timestamp: Utc::now(),
            checksum: "wrong_checksum".to_string(),
        };

        // ロールバックを実行
        let result = rollback(&backup_info).await;

        // エラーが返されることを確認
        assert!(result.is_err());
        match result {
            Err(TwfError::RollbackError(msg)) => {
                assert!(msg.contains("チェックサムが一致しません"));
            }
            _ => panic!("Expected RollbackError"),
        }
    }

    #[tokio::test]
    async fn test_calculate_checksum() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // テストファイルを作成
        fs::write(&test_file, "test content").await.unwrap();

        // チェックサムを計算
        let checksum = calculate_checksum(&test_file).await.unwrap();

        // チェックサムが64文字の16進数文字列であることを確認（SHA-256）
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));

        // 同じ内容のファイルは同じチェックサムを持つことを確認
        let checksum2 = calculate_checksum(&test_file).await.unwrap();
        assert_eq!(checksum, checksum2);
    }

    #[tokio::test]
    async fn test_rollback_preserves_original_content() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let original_path = temp_dir.path().join("config.txt");
        let backup_path = temp_dir.path().join("backup.txt");

        // 元のファイルを作成
        let original_content = "line 1\nline 2\nline 3\n";
        fs::write(&original_path, original_content)
            .await
            .unwrap();

        // バックアップファイルを作成（元のファイルと同じ内容）
        fs::write(&backup_path, original_content).await.unwrap();

        // チェックサムを計算
        let checksum = calculate_checksum(&backup_path).await.unwrap();

        // BackupInfoを作成
        let backup_info = BackupInfo {
            original_path: original_path.clone(),
            backup_path,
            timestamp: Utc::now(),
            checksum,
        };

        // 元のファイルを変更
        fs::write(&original_path, "completely different content")
            .await
            .unwrap();

        // ロールバックを実行
        rollback(&backup_info).await.unwrap();

        // 元のファイルが完全に復元されたことを確認
        let restored_content = fs::read_to_string(&original_path).await.unwrap();
        assert_eq!(restored_content, original_content);
    }
}
