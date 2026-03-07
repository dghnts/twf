// Alacritty設定パーサー

use crate::models::{Result, TwfError};
use std::path::PathBuf;

/// Alacrittyの背景画像を検出
///
/// Alacrittyの設定ファイル（YAML/TOML）から背景画像のパスを取得します。
/// 設定ファイルは以下の場所を順に確認します：
/// 1. ~/.config/alacritty/alacritty.yml
/// 2. ~/.config/alacritty/alacritty.toml
///
/// # Returns
/// - `Ok(Some(PathBuf))`: 背景画像が設定されている場合、そのパスを返す
/// - `Ok(None)`: 背景画像が設定されていない場合
/// - `Err(TwfError)`: 設定ファイルのパースに失敗した場合
pub async fn detect_alacritty_background() -> Result<Option<PathBuf>> {
    // 設定ファイルのパスを取得
    let config_dir = dirs::config_dir()
        .ok_or_else(|| TwfError::ConfigParseError("設定ディレクトリが見つかりません".to_string()))?;
    
    let config_paths = vec![
        config_dir.join("alacritty").join("alacritty.yml"),
        config_dir.join("alacritty").join("alacritty.toml"),
    ];
    
    // 各設定ファイルを順に確認
    for config_path in config_paths {
        if !config_path.exists() {
            continue;
        }
        
        // ファイルの拡張子に応じてパース方法を選択
        let bg_image_path = if config_path.extension().and_then(|s| s.to_str()) == Some("yml") {
            parse_yaml_config(&config_path).await?
        } else if config_path.extension().and_then(|s| s.to_str()) == Some("toml") {
            parse_toml_config(&config_path).await?
        } else {
            continue;
        };
        
        // 背景画像パスが見つかった場合
        if let Some(path) = bg_image_path {
            // パスを展開（~を展開）
            let expanded_path = expand_tilde(&path);
            
            // ファイルが存在するか確認
            if expanded_path.exists() {
                return Ok(Some(expanded_path));
            }
        }
    }
    
    // 背景画像が見つからなかった
    Ok(None)
}

/// YAML設定ファイルをパース
async fn parse_yaml_config(config_path: &PathBuf) -> Result<Option<PathBuf>> {
    let content = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| TwfError::ConfigParseError(format!("設定ファイルの読み込みに失敗: {}", e)))?;
    
    let config: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| TwfError::ConfigParseError(format!("YAMLのパースに失敗: {}", e)))?;
    
    // window.decorations.background_image を取得
    // または window.background_image を取得（古い形式）
    let bg_image = config
        .get("window")
        .and_then(|w| w.get("decorations"))
        .and_then(|d| d.get("background_image"))
        .or_else(|| {
            config
                .get("window")
                .and_then(|w| w.get("background_image"))
        })
        .and_then(|v| v.as_str())
        .map(|s| PathBuf::from(s));
    
    Ok(bg_image)
}

/// TOML設定ファイルをパース
async fn parse_toml_config(config_path: &PathBuf) -> Result<Option<PathBuf>> {
    let content = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| TwfError::ConfigParseError(format!("設定ファイルの読み込みに失敗: {}", e)))?;
    
    let config: toml::Value = toml::from_str(&content)
        .map_err(|e| TwfError::ConfigParseError(format!("TOMLのパースに失敗: {}", e)))?;
    
    // [window.decorations] background_image = "..." を取得
    // または [window] background_image = "..." を取得（古い形式）
    let bg_image = config
        .get("window")
        .and_then(|w| w.get("decorations"))
        .and_then(|d| d.get("background_image"))
        .or_else(|| {
            config
                .get("window")
                .and_then(|w| w.get("background_image"))
        })
        .and_then(|v| v.as_str())
        .map(|s| PathBuf::from(s));
    
    Ok(bg_image)
}

/// チルダ（~）をホームディレクトリに展開
fn expand_tilde(path: &PathBuf) -> PathBuf {
    if let Some(path_str) = path.to_str() {
        if path_str.starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                return home_dir.join(&path_str[2..]);
            }
        }
    }
    path.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    /// テスト用のYAML設定ファイルを作成
    fn create_test_yaml_config(dir: &std::path::Path, has_background: bool) -> PathBuf {
        let config_dir = dir.join("alacritty");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("alacritty.yml");
        
        let content = if has_background {
            r#"
window:
  decorations:
    background_image: /tmp/test_wallpaper.png
  opacity: 0.9
"#
        } else {
            r#"
window:
  opacity: 0.9
"#
        };
        
        fs::write(&config_path, content).unwrap();
        config_path
    }
    
    /// テスト用のTOML設定ファイルを作成
    fn create_test_toml_config(dir: &std::path::Path, has_background: bool) -> PathBuf {
        let config_dir = dir.join("alacritty");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("alacritty.toml");
        
        let content = if has_background {
            r#"
[window]
opacity = 0.9

[window.decorations]
background_image = "/tmp/test_wallpaper.png"
"#
        } else {
            r#"
[window]
opacity = 0.9
"#
        };
        
        fs::write(&config_path, content).unwrap();
        config_path
    }
    
    #[tokio::test]
    async fn test_parse_yaml_config_with_background() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_yaml_config(temp_dir.path(), true);
        
        let result = parse_yaml_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test_wallpaper.png"));
    }
    
    #[tokio::test]
    async fn test_parse_yaml_config_without_background() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_yaml_config(temp_dir.path(), false);
        
        let result = parse_yaml_config(&config_path).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_parse_toml_config_with_background() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_toml_config(temp_dir.path(), true);
        
        let result = parse_toml_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test_wallpaper.png"));
    }
    
    #[tokio::test]
    async fn test_parse_toml_config_without_background() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_toml_config(temp_dir.path(), false);
        
        let result = parse_toml_config(&config_path).await.unwrap();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_expand_tilde() {
        let path = PathBuf::from("~/Pictures/wallpaper.png");
        let expanded = expand_tilde(&path);
        
        // チルダが展開されていることを確認
        assert!(!expanded.to_str().unwrap().starts_with("~"));
        
        // 絶対パスの場合は変更されない
        let abs_path = PathBuf::from("/tmp/wallpaper.png");
        let expanded_abs = expand_tilde(&abs_path);
        assert_eq!(abs_path, expanded_abs);
    }
    
    #[test]
    fn test_yaml_old_format() {
        // 古い形式のYAML設定（window.background_image）をテスト
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("alacritty");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("alacritty.yml");
        let content = r#"
window:
  background_image: /tmp/old_format_wallpaper.png
  opacity: 0.9
"#;
        fs::write(&config_path, content).unwrap();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(parse_yaml_config(&config_path)).unwrap();
        
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/old_format_wallpaper.png"));
    }
    
    #[test]
    fn test_toml_old_format() {
        // 古い形式のTOML設定（window.background_image）をテスト
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("alacritty");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("alacritty.toml");
        let content = r#"
[window]
background_image = "/tmp/old_format_wallpaper.png"
opacity = 0.9
"#;
        fs::write(&config_path, content).unwrap();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(parse_toml_config(&config_path)).unwrap();
        
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/old_format_wallpaper.png"));
    }
}
