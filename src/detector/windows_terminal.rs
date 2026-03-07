// Windows Terminal設定パーサー

use crate::models::{Result, TwfError};
use std::path::PathBuf;

/// Windows Terminalの背景画像を検出
///
/// Windows Terminalの設定ファイル（JSON）から背景画像のパスを取得します。
/// 設定ファイルは以下の場所を確認します：
/// - %LOCALAPPDATA%\Packages\Microsoft.WindowsTerminal_*\LocalState\settings.json
///
/// # 処理フロー
/// 1. 設定ファイルのパスを取得
/// 2. JSON設定ファイルをパース
/// 3. 現在のプロファイルまたはデフォルトプロファイルから背景画像パスを取得
/// 4. 画像パスが存在する場合は返す
///
/// # Returns
/// - `Ok(Some(PathBuf))`: 背景画像が設定されている場合、そのパスを返す
/// - `Ok(None)`: 背景画像が設定されていない場合
/// - `Err(TwfError)`: 設定ファイルのパースに失敗した場合
///
/// # Requirements
/// - 2.8.1: Windows Terminalの設定ファイルから背景画像パスを取得できること
/// - 2.8.2: JSON設定ファイルのパース処理を実装
pub async fn detect_windows_terminal_background() -> Result<Option<PathBuf>> {
    // 1. 設定ファイルのパスを取得
    let settings_path = find_windows_terminal_settings()?;
    
    if !settings_path.exists() {
        return Ok(None);
    }
    
    // 2. JSON設定ファイルをパース
    let content = tokio::fs::read_to_string(&settings_path)
        .await
        .map_err(|e| TwfError::ConfigParseError(format!("設定ファイルの読み込みに失敗: {}", e)))?;
    
    let settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| TwfError::ConfigParseError(format!("JSONのパースに失敗: {}", e)))?;
    
    // 3. 現在のプロファイルまたはデフォルトプロファイルから背景画像パスを取得
    let bg_image_path = extract_background_image_path(&settings)?;
    
    // 4. 画像パスが存在する場合は返す
    if let Some(path) = bg_image_path {
        let expanded_path = expand_env_vars(&path);
        if expanded_path.exists() {
            return Ok(Some(expanded_path));
        }
    }
    
    Ok(None)
}

/// Windows Terminalの設定ファイルパスを検索
///
/// %LOCALAPPDATA%\Packages\Microsoft.WindowsTerminal_*\LocalState\settings.json
/// を検索します。ワイルドカード部分は実際のパッケージ名に展開されます。
fn find_windows_terminal_settings() -> Result<PathBuf> {
    // LOCALAPPDATA環境変数を取得
    let local_app_data = std::env::var("LOCALAPPDATA")
        .map_err(|_| TwfError::ConfigParseError("LOCALAPPDATA環境変数が見つかりません".to_string()))?;
    
    let packages_dir = PathBuf::from(local_app_data).join("Packages");
    
    if !packages_dir.exists() {
        return Err(TwfError::ConfigParseError("Packagesディレクトリが見つかりません".to_string()));
    }
    
    // Microsoft.WindowsTerminal_* パターンに一致するディレクトリを検索
    let entries = std::fs::read_dir(&packages_dir)
        .map_err(|e| TwfError::ConfigParseError(format!("Packagesディレクトリの読み込みに失敗: {}", e)))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| TwfError::ConfigParseError(format!("ディレクトリエントリの読み込みに失敗: {}", e)))?;
        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();
        
        // Microsoft.WindowsTerminal_ で始まるディレクトリを探す
        if dir_name_str.starts_with("Microsoft.WindowsTerminal_") {
            let settings_path = entry.path().join("LocalState").join("settings.json");
            if settings_path.exists() {
                return Ok(settings_path);
            }
        }
    }
    
    Err(TwfError::ConfigParseError("Windows Terminal設定ファイルが見つかりません".to_string()))
}

/// JSON設定から背景画像パスを抽出
///
/// Windows Terminalの設定構造：
/// ```json
/// {
///   "defaultProfile": "{guid}",
///   "profiles": {
///     "defaults": {
///       "backgroundImage": "path/to/image.png"
///     },
///     "list": [
///       {
///         "guid": "{guid}",
///         "backgroundImage": "path/to/image.png"
///       }
///     ]
///   }
/// }
/// ```
fn extract_background_image_path(settings: &serde_json::Value) -> Result<Option<PathBuf>> {
    // デフォルトプロファイルのGUIDを取得
    let default_profile_guid = settings
        .get("defaultProfile")
        .and_then(|v| v.as_str());
    
    // プロファイルリストを取得
    let profiles = settings
        .get("profiles")
        .ok_or_else(|| TwfError::ConfigParseError("profiles セクションが見つかりません".to_string()))?;
    
    // まず、デフォルトプロファイルから背景画像を探す
    if let Some(guid) = default_profile_guid {
        if let Some(list) = profiles.get("list").and_then(|v| v.as_array()) {
            for profile in list {
                if let Some(profile_guid) = profile.get("guid").and_then(|v| v.as_str()) {
                    if profile_guid == guid {
                        if let Some(bg_image) = profile.get("backgroundImage").and_then(|v| v.as_str()) {
                            return Ok(Some(PathBuf::from(bg_image)));
                        }
                    }
                }
            }
        }
    }
    
    // デフォルトプロファイルに背景画像がない場合、profiles.defaults を確認
    if let Some(defaults) = profiles.get("defaults") {
        if let Some(bg_image) = defaults.get("backgroundImage").and_then(|v| v.as_str()) {
            return Ok(Some(PathBuf::from(bg_image)));
        }
    }
    
    // 最初のプロファイルから背景画像を探す（フォールバック）
    if let Some(list) = profiles.get("list").and_then(|v| v.as_array()) {
        for profile in list {
            if let Some(bg_image) = profile.get("backgroundImage").and_then(|v| v.as_str()) {
                return Ok(Some(PathBuf::from(bg_image)));
            }
        }
    }
    
    Ok(None)
}

/// 環境変数を展開
///
/// Windows Terminalの設定では、環境変数を使用できます（例: %USERPROFILE%）
fn expand_env_vars(path: &PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    
    // %VARIABLE% 形式の環境変数を展開
    let expanded = shellexpand::env(&path_str)
        .unwrap_or(std::borrow::Cow::Borrowed(&path_str));
    
    PathBuf::from(expanded.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    /// テスト用のJSON設定ファイルを作成
    fn create_test_settings(dir: &std::path::Path, has_background: bool, use_defaults: bool) -> PathBuf {
        let settings_path = dir.join("settings.json");
        
        let content = if has_background {
            if use_defaults {
                // profiles.defaults に背景画像を設定
                r#"{
  "defaultProfile": "{12345678-1234-1234-1234-123456789012}",
  "profiles": {
    "defaults": {
      "backgroundImage": "C:\\Users\\Test\\Pictures\\wallpaper.png"
    },
    "list": [
      {
        "guid": "{12345678-1234-1234-1234-123456789012}",
        "name": "Windows PowerShell"
      }
    ]
  }
}"#
            } else {
                // 特定のプロファイルに背景画像を設定
                r#"{
  "defaultProfile": "{12345678-1234-1234-1234-123456789012}",
  "profiles": {
    "list": [
      {
        "guid": "{12345678-1234-1234-1234-123456789012}",
        "name": "Windows PowerShell",
        "backgroundImage": "C:\\Users\\Test\\Pictures\\wallpaper.png"
      }
    ]
  }
}"#
            }
        } else {
            // 背景画像なし
            r#"{
  "defaultProfile": "{12345678-1234-1234-1234-123456789012}",
  "profiles": {
    "list": [
      {
        "guid": "{12345678-1234-1234-1234-123456789012}",
        "name": "Windows PowerShell"
      }
    ]
  }
}"#
        };
        
        fs::write(&settings_path, content).unwrap();
        settings_path
    }
    
    #[test]
    fn test_extract_background_image_from_profile() {
        let temp_dir = tempdir().unwrap();
        let settings_path = create_test_settings(temp_dir.path(), true, false);
        
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        
        let result = extract_background_image_path(&settings).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("C:\\Users\\Test\\Pictures\\wallpaper.png"));
    }
    
    #[test]
    fn test_extract_background_image_from_defaults() {
        let temp_dir = tempdir().unwrap();
        let settings_path = create_test_settings(temp_dir.path(), true, true);
        
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        
        let result = extract_background_image_path(&settings).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("C:\\Users\\Test\\Pictures\\wallpaper.png"));
    }
    
    #[test]
    fn test_extract_background_image_not_found() {
        let temp_dir = tempdir().unwrap();
        let settings_path = create_test_settings(temp_dir.path(), false, false);
        
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        
        let result = extract_background_image_path(&settings).unwrap();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_expand_env_vars() {
        // 環境変数を含むパスをテスト
        std::env::set_var("TEST_VAR", "TestValue");
        
        let path = PathBuf::from("$TEST_VAR/Pictures/wallpaper.png");
        let expanded = expand_env_vars(&path);
        
        assert!(expanded.to_string_lossy().contains("TestValue"));
        
        // 環境変数がない場合はそのまま返す
        let path_no_var = PathBuf::from("C:\\Users\\Test\\Pictures\\wallpaper.png");
        let expanded_no_var = expand_env_vars(&path_no_var);
        assert_eq!(path_no_var, expanded_no_var);
    }
    
    #[tokio::test]
    async fn test_detect_windows_terminal_background_with_image() {
        // 実際のWindows Terminal設定ファイルを使用したテストは、
        // 実環境でのみ実行可能です。
        // ここでは、モック機能を使用したテストを実装する必要があります。
    }
    
    #[test]
    fn test_json_parsing_with_multiple_profiles() {
        let json_content = r#"{
  "defaultProfile": "{guid-2}",
  "profiles": {
    "defaults": {
      "fontSize": 12
    },
    "list": [
      {
        "guid": "{guid-1}",
        "name": "Profile 1",
        "backgroundImage": "C:\\path1.png"
      },
      {
        "guid": "{guid-2}",
        "name": "Profile 2",
        "backgroundImage": "C:\\path2.png"
      }
    ]
  }
}"#;
        
        let settings: serde_json::Value = serde_json::from_str(json_content).unwrap();
        let result = extract_background_image_path(&settings).unwrap();
        
        // デフォルトプロファイル（guid-2）の背景画像が取得されるべき
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("C:\\path2.png"));
    }
}
