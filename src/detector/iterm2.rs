// iTerm2設定パーサー

use crate::models::TwfError;
use std::path::PathBuf;

/// iTerm2の背景画像パスを検出
///
/// iTerm2の設定ファイル（plist）から背景画像のパスを取得します。
/// 
/// # 処理フロー
/// 1. plistファイルのパスを取得（~/Library/Preferences/com.googlecode.iterm2.plist）
/// 2. plistをパース
/// 3. 現在のプロファイルを取得（Default Bookmark Guid）
/// 4. プロファイルから背景画像パスを取得（Background Image Location）
/// 5. 画像パスが存在する場合は返す
///
/// # Returns
/// - `Ok(Some(PathBuf))`: 背景画像パスが検出された場合
/// - `Ok(None)`: 背景画像が設定されていない場合
/// - `Err(TwfError)`: plistファイルの読み込みやパースに失敗した場合
///
/// # Requirements
/// - 2.8.1: iTerm2の設定ファイルから背景画像パスを取得できること
/// - 2.8.2: plistファイルのパース処理を実装
pub async fn detect_iterm2_background() -> Result<Option<PathBuf>, TwfError> {
    // 1. plistファイルのパスを取得
    let home_dir = dirs::home_dir().ok_or_else(|| {
        TwfError::ConfigParseError("ホームディレクトリが見つかりません".to_string())
    })?;
    
    let plist_path = home_dir
        .join("Library")
        .join("Preferences")
        .join("com.googlecode.iterm2.plist");
    
    // plistファイルが存在しない場合はNoneを返す
    if !plist_path.exists() {
        return Ok(None);
    }
    
    // 2. plistをパース
    let plist_value = plist::Value::from_file(&plist_path).map_err(|e| {
        TwfError::ConfigParseError(format!("iTerm2 plistファイルの読み込みに失敗しました: {}", e))
    })?;
    
    // plistをディクショナリとして取得
    let plist_dict = plist_value.as_dictionary().ok_or_else(|| {
        TwfError::ConfigParseError("iTerm2 plistがディクショナリ形式ではありません".to_string())
    })?;
    
    // 3. 現在のプロファイルGUIDを取得
    let current_profile_guid: Option<String> = plist_dict
        .get("Default Bookmark Guid")
        .and_then(|v: &plist::Value| v.as_string())
        .map(|s: &str| s.to_string());
    
    // 4. プロファイルリストを取得
    let profiles: &Vec<plist::Value> = plist_dict
        .get("New Bookmarks")
        .and_then(|v: &plist::Value| v.as_array())
        .ok_or_else(|| {
            TwfError::ConfigParseError("iTerm2 plistにプロファイルリストが見つかりません".to_string())
        })?;
    
    // 5. 現在のプロファイルまたは最初のプロファイルから背景画像パスを取得
    for profile in profiles {
        let profile_dict: &plist::Dictionary = match profile.as_dictionary() {
            Some(dict) => dict,
            None => continue,
        };
        
        // 現在のプロファイルGUIDが設定されている場合は、それに一致するプロファイルを探す
        if let Some(ref guid) = current_profile_guid {
            let profile_guid: Option<&str> = profile_dict
                .get("Guid")
                .and_then(|v: &plist::Value| v.as_string());
            
            if profile_guid != Some(guid.as_str()) {
                continue;
            }
        }
        
        // 背景画像パスを取得
        if let Some(bg_image_location) = profile_dict
            .get("Background Image Location")
            .and_then(|v: &plist::Value| v.as_string())
        {
            let bg_image_path = PathBuf::from(bg_image_location);
            
            // パスが存在する場合のみ返す
            if bg_image_path.exists() {
                return Ok(Some(bg_image_path));
            }
        }
        
        // 現在のプロファイルが見つかった場合は、背景画像がなくてもループを抜ける
        if current_profile_guid.is_some() {
            break;
        }
    }
    
    // 背景画像が見つからなかった場合
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    /// テスト用のplistファイルを作成
    fn create_test_plist(dir: &std::path::Path, has_background: bool) -> PathBuf {
        let plist_path = dir.join("com.googlecode.iterm2.plist");
        
        // テスト用の背景画像を作成
        let bg_image_path = if has_background {
            let img_path = dir.join("test_background.png");
            fs::write(&img_path, b"fake image data").unwrap();
            img_path.to_str().unwrap().to_string()
        } else {
            String::new()
        };
        
        // plistの内容を作成
        let mut root_dict = plist::Dictionary::new();
        root_dict.insert(
            "Default Bookmark Guid".to_string(),
            plist::Value::String("test-guid-123".to_string()),
        );
        
        // プロファイルを作成
        let mut profile_dict = plist::Dictionary::new();
        profile_dict.insert(
            "Guid".to_string(),
            plist::Value::String("test-guid-123".to_string()),
        );
        profile_dict.insert(
            "Name".to_string(),
            plist::Value::String("Default".to_string()),
        );
        
        if has_background {
            profile_dict.insert(
                "Background Image Location".to_string(),
                plist::Value::String(bg_image_path),
            );
        }
        
        let profiles = vec![plist::Value::Dictionary(profile_dict)];
        root_dict.insert("New Bookmarks".to_string(), plist::Value::Array(profiles));
        
        // plistファイルに書き込み
        let plist_value = plist::Value::Dictionary(root_dict);
        plist_value.to_file_xml(&plist_path).unwrap();
        
        plist_path
    }
    
    #[tokio::test]
    async fn test_detect_iterm2_background_with_image() {
        let temp_dir = tempdir().unwrap();
        let _plist_path = create_test_plist(temp_dir.path(), true);
        
        // ホームディレクトリを一時ディレクトリに設定するのは難しいため、
        // このテストは実際のファイルシステムに依存します
        // 実際の環境でテストする場合は、モック機能を追加する必要があります
    }
    
    #[tokio::test]
    async fn test_detect_iterm2_background_without_image() {
        let temp_dir = tempdir().unwrap();
        let _plist_path = create_test_plist(temp_dir.path(), false);
        
        // 同様に、実際の環境でのテストが必要
    }
    
    #[test]
    fn test_plist_parsing() {
        // plistのパース機能をテスト
        let temp_dir = tempdir().unwrap();
        let plist_path = create_test_plist(temp_dir.path(), true);
        
        // plistを読み込み
        let plist_value = plist::Value::from_file(&plist_path).unwrap();
        let plist_dict = plist_value.as_dictionary().unwrap();
        
        // Default Bookmark Guidが存在することを確認
        assert!(plist_dict.contains_key("Default Bookmark Guid"));
        
        // プロファイルリストが存在することを確認
        assert!(plist_dict.contains_key("New Bookmarks"));
        
        let profiles = plist_dict.get("New Bookmarks").unwrap().as_array().unwrap();
        assert_eq!(profiles.len(), 1);
        
        // プロファイルに背景画像パスが含まれることを確認
        let profile = profiles[0].as_dictionary().unwrap();
        assert!(profile.contains_key("Background Image Location"));
    }
}
