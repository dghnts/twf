// バックアップ管理

use crate::models::{BackupInfo, Result, TwfError};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;

/// バックアップマネージャー
pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    /// 新しいBackupManagerを作成
    pub fn new(backup_dir: PathBuf) -> Self {
        Self { backup_dir }
    }

    /// バックアップを作成
    /// 
    /// タイムスタンプ付きのバックアップファイルを作成し、
    /// バックアップ情報を返します。
    /// 
    /// # Arguments
    /// * `path` - バックアップ対象のファイルパス
    /// 
    /// # Returns
    /// * `Result<BackupInfo>` - バックアップ情報
    pub async fn create_backup(&self, path: &Path) -> Result<BackupInfo> {
        // 1. バックアップディレクトリを作成
        fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| TwfError::BackupError(format!("バックアップディレクトリの作成に失敗: {}", e)))?;

        // 2. タイムスタンプ付きのバックアップファイル名を生成
        let timestamp = Utc::now();
        let timestamp_str = timestamp.format("%Y%m%d_%H%M%S");
        let original_filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| TwfError::BackupError("ファイル名の取得に失敗".to_string()))?;
        let backup_filename = format!("{}_{}", timestamp_str, original_filename);
        let backup_path = self.backup_dir.join(&backup_filename);

        // 3. 元のファイルをコピー
        fs::copy(path, &backup_path)
            .await
            .map_err(|e| TwfError::BackupError(format!("ファイルのコピーに失敗: {}", e)))?;

        // 4. チェックサムを計算
        let checksum = calculate_checksum(&backup_path).await?;

        // 5. バックアップ情報を作成
        let backup_info = BackupInfo {
            original_path: path.to_path_buf(),
            backup_path: backup_path.clone(),
            timestamp,
            checksum,
        };

        // 6. バックアップ情報をJSONファイルとして保存
        save_backup_info(&backup_info, &self.backup_dir).await?;

        Ok(backup_info)
    }

    /// 最新のバックアップを取得
    /// 
    /// バックアップディレクトリから最新のバックアップ情報を取得します。
    /// 
    /// # Returns
    /// * `Result<Option<BackupInfo>>` - 最新のバックアップ情報（存在しない場合はNone）
    pub async fn get_latest_backup(&self) -> Result<Option<BackupInfo>> {
        // バックアップディレクトリが存在しない場合はNoneを返す
        if !self.backup_dir.exists() {
            return Ok(None);
        }

        // バックアップ情報ファイル（.json）を検索
        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| TwfError::BackupError(format!("バックアップディレクトリの読み込みに失敗: {}", e)))?;

        let mut backup_infos = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            TwfError::BackupError(format!("ディレクトリエントリの読み込みに失敗: {}", e))
        })? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // JSONファイルを読み込んでBackupInfoにデシリアライズ
                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Ok(backup_info) = serde_json::from_str::<BackupInfo>(&content) {
                        backup_infos.push(backup_info);
                    }
                }
            }
        }

        // タイムスタンプでソートして最新のものを返す
        backup_infos.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(backup_infos.into_iter().next())
    }
}

/// ファイルのチェックサムを計算（SHA-256）
async fn calculate_checksum(path: &Path) -> Result<String> {
    let content = fs::read(path)
        .await
        .map_err(|e| TwfError::BackupError(format!("ファイルの読み込みに失敗: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

/// バックアップ情報をJSONファイルとして保存
async fn save_backup_info(backup_info: &BackupInfo, backup_dir: &Path) -> Result<()> {
    let timestamp_str = backup_info.timestamp.format("%Y%m%d_%H%M%S");
    let info_filename = format!("backup_{}.json", timestamp_str);
    let info_path = backup_dir.join(info_filename);

    let json = serde_json::to_string_pretty(backup_info)
        .map_err(|e| TwfError::BackupError(format!("JSONのシリアライズに失敗: {}", e)))?;

    fs::write(&info_path, json)
        .await
        .map_err(|e| TwfError::BackupError(format!("バックアップ情報の保存に失敗: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_backup() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test_config.txt");
        let backup_dir = temp_dir.path().join("backups");

        // テストファイルを作成
        fs::write(&test_file, "original content")
            .await
            .unwrap();

        // BackupManagerを作成
        let manager = BackupManager::new(backup_dir.clone());

        // バックアップを作成
        let backup_info = manager.create_backup(&test_file).await.unwrap();

        // バックアップファイルが存在することを確認
        assert!(backup_info.backup_path.exists());

        // バックアップファイルの内容が元のファイルと同じことを確認
        let backup_content = fs::read_to_string(&backup_info.backup_path)
            .await
            .unwrap();
        assert_eq!(backup_content, "original content");

        // チェックサムが計算されていることを確認
        assert!(!backup_info.checksum.is_empty());
    }

    #[tokio::test]
    async fn test_get_latest_backup() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test_config.txt");
        let backup_dir = temp_dir.path().join("backups");

        // テストファイルを作成
        fs::write(&test_file, "content 1").await.unwrap();

        // BackupManagerを作成
        let manager = BackupManager::new(backup_dir.clone());

        // 最初のバックアップを作成
        let backup1 = manager.create_backup(&test_file).await.unwrap();

        // 少し待機（タイムスタンプが異なることを保証）
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // ファイルを更新
        fs::write(&test_file, "content 2").await.unwrap();

        // 2番目のバックアップを作成
        let backup2 = manager.create_backup(&test_file).await.unwrap();

        // 最新のバックアップを取得
        let latest = manager.get_latest_backup().await.unwrap();

        // 最新のバックアップが2番目のバックアップであることを確認
        assert!(latest.is_some());
        let latest = latest.unwrap();
        assert_eq!(latest.backup_path, backup2.backup_path);
        assert!(latest.timestamp >= backup1.timestamp);
    }

    #[tokio::test]
    async fn test_get_latest_backup_empty() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let backup_dir = temp_dir.path().join("backups");

        // BackupManagerを作成
        let manager = BackupManager::new(backup_dir);

        // バックアップが存在しない場合はNoneが返されることを確認
        let latest = manager.get_latest_backup().await.unwrap();
        assert!(latest.is_none());
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
    }
}
