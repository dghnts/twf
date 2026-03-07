// Property 9: 設定のラウンドトリップ（バックアップとロールバック）
//
// このプロパティテストは、バックアップとロールバック機能の正確性を検証します。
// 任意の設定ファイルに対して、バックアップを作成してから新しい設定を適用し、
// その後ロールバックを実行すると、元の設定ファイルの内容が完全に復元されることを確認します。
//
// **Validates: Requirements 2.5.4**

use proptest::prelude::*;
use std::fs;
use tempfile::tempdir;
use twf::applier::backup::BackupManager;
use twf::applier::rollback::rollback;

/// Property 9.1: バックアップとロールバックのラウンドトリップ
///
/// 任意の設定ファイル内容に対して、以下の操作を実行します：
/// 1. 元の設定ファイルを作成
/// 2. バックアップを作成
/// 3. 設定ファイルを変更
/// 4. ロールバックを実行
/// 5. 元の内容が完全に復元されることを検証
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    
    #[test]
    fn prop_config_roundtrip(
        original_content in "\\PC{10,1000}",  // 任意のUnicode文字列（10〜1000文字）
        modified_content in "\\PC{10,1000}",  // 変更後の内容
    ) {
        // 非同期ランタイムを作成
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            // テンポラリディレクトリを作成
            let temp_dir = tempdir().unwrap();
            let config_path = temp_dir.path().join("test_config.txt");
            let backup_dir = temp_dir.path().join("backups");
            
            // 元の設定ファイルを作成
            fs::write(&config_path, &original_content).unwrap();
            
            // BackupManagerを作成
            let manager = BackupManager::new(backup_dir.clone());
            
            // バックアップを作成
            let backup_info = manager.create_backup(&config_path).await.unwrap();
            
            // バックアップファイルが存在することを確認
            prop_assert!(
                backup_info.backup_path.exists(),
                "バックアップファイルが作成されませんでした"
            );
            
            // 設定ファイルを変更
            fs::write(&config_path, &modified_content).unwrap();
            
            // 変更が反映されたことを確認
            let modified = fs::read_to_string(&config_path).unwrap();
            prop_assert_eq!(
                modified,
                modified_content,
                "設定ファイルの変更が反映されませんでした"
            );
            
            // ロールバックを実行
            rollback(&backup_info).await.unwrap();
            
            // 元の内容が復元されたことを検証
            let restored_content = fs::read_to_string(&config_path).unwrap();
            prop_assert_eq!(
                restored_content,
                original_content,
                "ロールバック後の内容が元の内容と一致しません"
            );
            
            Ok(())
        })?;
    }
}

/// Property 9.2: バックアップファイルの内容が元のファイルと同じであること
///
/// バックアップを作成した直後、バックアップファイルの内容が
/// 元のファイルの内容と完全に一致することを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    
    #[test]
    fn prop_backup_preserves_content(
        content in "\\PC{10,1000}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let temp_dir = tempdir().unwrap();
            let config_path = temp_dir.path().join("test_config.txt");
            let backup_dir = temp_dir.path().join("backups");
            
            // 元のファイルを作成
            fs::write(&config_path, &content).unwrap();
            
            // BackupManagerを作成
            let manager = BackupManager::new(backup_dir);
            
            // バックアップを作成
            let backup_info = manager.create_backup(&config_path).await.unwrap();
            
            // バックアップファイルの内容を読み込み
            let backup_content = fs::read_to_string(&backup_info.backup_path).unwrap();
            
            // 元のファイルの内容と一致することを検証
            prop_assert_eq!(
                backup_content,
                content,
                "バックアップファイルの内容が元のファイルと一致しません"
            );
            
            Ok(())
        })?;
    }
}

/// Property 9.3: 複数回のバックアップとロールバックが正しく動作すること
///
/// 複数回のバックアップとロールバックを繰り返しても、
/// 正しく動作することを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    
    #[test]
    fn prop_multiple_backup_rollback_cycles(
        contents in prop::collection::vec("\\PC{10,500}", 2..5),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let temp_dir = tempdir().unwrap();
            let config_path = temp_dir.path().join("test_config.txt");
            let backup_dir = temp_dir.path().join("backups");
            
            let manager = BackupManager::new(backup_dir);
            
            // 各内容に対してバックアップ→変更→ロールバックのサイクルを実行
            for (i, content) in contents.iter().enumerate() {
                // 現在の内容を書き込み
                fs::write(&config_path, content).unwrap();
                
                // バックアップを作成
                let backup_info = manager.create_backup(&config_path).await.unwrap();
                
                // ファイルを変更
                fs::write(&config_path, "modified content").unwrap();
                
                // ロールバック
                rollback(&backup_info).await.unwrap();
                
                // 元の内容が復元されたことを確認
                let restored = fs::read_to_string(&config_path).unwrap();
                prop_assert_eq!(
                    restored,
                    content.as_str(),
                    "サイクル {} でロールバックが失敗しました",
                    i
                );
            }
            
            Ok(())
        })?;
    }
}

/// Property 9.4: バックアップファイルが正しい場所に作成されること
///
/// バックアップファイルがバックアップディレクトリ内に作成され、
/// 適切なファイル名（タイムスタンプ付き）を持つことを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    
    #[test]
    fn prop_backup_file_location(
        content in "\\PC{10,500}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let temp_dir = tempdir().unwrap();
            let config_path = temp_dir.path().join("test_config.txt");
            let backup_dir = temp_dir.path().join("backups");
            
            // 元のファイルを作成
            fs::write(&config_path, &content).unwrap();
            
            // BackupManagerを作成
            let manager = BackupManager::new(backup_dir.clone());
            
            // バックアップを作成
            let backup_info = manager.create_backup(&config_path).await.unwrap();
            
            // バックアップファイルがバックアップディレクトリ内にあることを確認
            prop_assert!(
                backup_info.backup_path.starts_with(&backup_dir),
                "バックアップファイルが正しいディレクトリに作成されませんでした"
            );
            
            // バックアップファイル名にタイムスタンプが含まれることを確認
            let filename = backup_info.backup_path.file_name().unwrap().to_str().unwrap();
            prop_assert!(
                filename.contains("_"),
                "バックアップファイル名にタイムスタンプが含まれていません: {}",
                filename
            );
            
            // 元のファイル名が含まれることを確認
            prop_assert!(
                filename.contains("test_config.txt"),
                "バックアップファイル名に元のファイル名が含まれていません: {}",
                filename
            );
            
            Ok(())
        })?;
    }
}

/// Property 9.5: ロールバック後、ファイルの内容が完全に一致すること
///
/// ロールバック後、ファイルの内容がバイト単位で完全に一致することを検証します。
/// これには改行コード、空白文字、特殊文字なども含まれます。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    
    #[test]
    fn prop_rollback_exact_match(
        original_content in prop::collection::vec(any::<u8>(), 10..1000),
        modified_content in prop::collection::vec(any::<u8>(), 10..1000),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let temp_dir = tempdir().unwrap();
            let config_path = temp_dir.path().join("test_config.bin");
            let backup_dir = temp_dir.path().join("backups");
            
            // 元のファイルを作成（バイナリデータ）
            fs::write(&config_path, &original_content).unwrap();
            
            // BackupManagerを作成
            let manager = BackupManager::new(backup_dir);
            
            // バックアップを作成
            let backup_info = manager.create_backup(&config_path).await.unwrap();
            
            // ファイルを変更
            fs::write(&config_path, &modified_content).unwrap();
            
            // ロールバック
            rollback(&backup_info).await.unwrap();
            
            // 元の内容が完全に復元されたことを検証（バイト単位）
            let restored_content = fs::read(&config_path).unwrap();
            prop_assert_eq!(
                restored_content,
                original_content,
                "ロールバック後の内容がバイト単位で一致しません"
            );
            
            Ok(())
        })?;
    }
}

/// Property 9.6: バックアップとロールバックがパニックしないこと
///
/// 任意の入力に対して、バックアップとロールバック処理が
/// パニックせずに完了することを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    
    #[test]
    fn prop_no_panic(
        content in "\\PC{0,1000}",  // 空文字列も含む
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let temp_dir = tempdir().unwrap();
            let config_path = temp_dir.path().join("test_config.txt");
            let backup_dir = temp_dir.path().join("backups");
            
            // ファイルを作成
            fs::write(&config_path, &content).unwrap();
            
            // BackupManagerを作成
            let manager = BackupManager::new(backup_dir);
            
            // バックアップを作成（パニックしないことを確認）
            let backup_result = manager.create_backup(&config_path).await;
            prop_assert!(
                backup_result.is_ok(),
                "バックアップ作成がエラーを返しました: {:?}",
                backup_result
            );
            
            let backup_info = backup_result.unwrap();
            
            // ファイルを変更
            fs::write(&config_path, "modified").unwrap();
            
            // ロールバック（パニックしないことを確認）
            let rollback_result = rollback(&backup_info).await;
            prop_assert!(
                rollback_result.is_ok(),
                "ロールバックがエラーを返しました: {:?}",
                rollback_result
            );
            
            Ok(())
        })?;
    }
}

// 追加のユニットテスト

#[cfg(test)]
mod unit_tests {
    use super::*;
    use tokio::fs;
    
    /// 空のファイルに対するバックアップとロールバック
    #[tokio::test]
    async fn test_empty_file_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("empty.txt");
        let backup_dir = temp_dir.path().join("backups");
        
        // 空のファイルを作成
        fs::write(&config_path, "").await.unwrap();
        
        let manager = BackupManager::new(backup_dir);
        let backup_info = manager.create_backup(&config_path).await.unwrap();
        
        // ファイルを変更
        fs::write(&config_path, "not empty").await.unwrap();
        
        // ロールバック
        rollback(&backup_info).await.unwrap();
        
        // 空のファイルが復元されたことを確認
        let restored = fs::read_to_string(&config_path).await.unwrap();
        assert_eq!(restored, "");
    }
    
    /// 大きなファイルに対するバックアップとロールバック
    #[tokio::test]
    async fn test_large_file_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("large.txt");
        let backup_dir = temp_dir.path().join("backups");
        
        // 大きなファイルを作成（1MB）
        let large_content = "a".repeat(1024 * 1024);
        fs::write(&config_path, &large_content).await.unwrap();
        
        let manager = BackupManager::new(backup_dir);
        let backup_info = manager.create_backup(&config_path).await.unwrap();
        
        // ファイルを変更
        fs::write(&config_path, "small").await.unwrap();
        
        // ロールバック
        rollback(&backup_info).await.unwrap();
        
        // 大きなファイルが復元されたことを確認
        let restored = fs::read_to_string(&config_path).await.unwrap();
        assert_eq!(restored, large_content);
    }
    
    /// 改行コードを含むファイルに対するバックアップとロールバック
    #[tokio::test]
    async fn test_newline_preservation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("newlines.txt");
        let backup_dir = temp_dir.path().join("backups");
        
        // 様々な改行コードを含むファイルを作成
        let content_with_newlines = "line1\nline2\r\nline3\rline4\n";
        fs::write(&config_path, content_with_newlines).await.unwrap();
        
        let manager = BackupManager::new(backup_dir);
        let backup_info = manager.create_backup(&config_path).await.unwrap();
        
        // ファイルを変更
        fs::write(&config_path, "no newlines").await.unwrap();
        
        // ロールバック
        rollback(&backup_info).await.unwrap();
        
        // 改行コードが保持されたことを確認
        let restored = fs::read_to_string(&config_path).await.unwrap();
        assert_eq!(restored, content_with_newlines);
    }
    
    /// Unicode文字を含むファイルに対するバックアップとロールバック
    #[tokio::test]
    async fn test_unicode_preservation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("unicode.txt");
        let backup_dir = temp_dir.path().join("backups");
        
        // Unicode文字を含むファイルを作成
        let unicode_content = "日本語\n中文\n한글\n🎨🔧✨";
        fs::write(&config_path, unicode_content).await.unwrap();
        
        let manager = BackupManager::new(backup_dir);
        let backup_info = manager.create_backup(&config_path).await.unwrap();
        
        // ファイルを変更
        fs::write(&config_path, "ASCII only").await.unwrap();
        
        // ロールバック
        rollback(&backup_info).await.unwrap();
        
        // Unicode文字が保持されたことを確認
        let restored = fs::read_to_string(&config_path).await.unwrap();
        assert_eq!(restored, unicode_content);
    }
    
    /// 最新のバックアップを取得するテスト
    #[tokio::test]
    async fn test_get_latest_backup_multiple() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.txt");
        let backup_dir = temp_dir.path().join("backups");
        
        let manager = BackupManager::new(backup_dir);
        
        // 複数のバックアップを作成
        fs::write(&config_path, "version 1").await.unwrap();
        let backup1 = manager.create_backup(&config_path).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        fs::write(&config_path, "version 2").await.unwrap();
        let backup2 = manager.create_backup(&config_path).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        fs::write(&config_path, "version 3").await.unwrap();
        let backup3 = manager.create_backup(&config_path).await.unwrap();
        
        // 最新のバックアップを取得
        let latest = manager.get_latest_backup().await.unwrap();
        
        // 最新のバックアップがbackup3であることを確認
        assert!(latest.is_some());
        let latest = latest.unwrap();
        assert_eq!(latest.backup_path, backup3.backup_path);
        assert!(latest.timestamp >= backup2.timestamp);
        assert!(latest.timestamp >= backup1.timestamp);
    }
}
