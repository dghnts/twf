// Kitty設定パーサー

use crate::models::{Result, TwfError};
use std::path::PathBuf;

/// Kittyの背景画像を検出
///
/// Kittyの設定ファイル（kitty.conf）から背景画像のパスを取得します。
/// 設定ファイルは以下の場所を確認します：
/// - ~/.config/kitty/kitty.conf
///
/// # 処理フロー
/// 1. 設定ファイルのパスを取得（~/.config/kitty/kitty.conf）
/// 2. 設定ファイルをパース
/// 3. background_image設定を取得
/// 4. 画像パスが存在する場合は返す
///
/// # Returns
/// - `Ok(Some(PathBuf))`: 背景画像が設定されている場合、そのパスを返す
/// - `Ok(None)`: 背景画像が設定されていない場合
/// - `Err(TwfError)`: 設定ファイルのパースに失敗した場合
///
/// # Requirements
/// - 2.8.1: Kittyの設定から背景画像パスを取得できること
/// - 2.8.2: Kitty設定ファイルのパース処理を実装
pub async fn detect_kitty_background() -> Result<Option<PathBuf>> {
    // 設定ファイルのパスを取得
    let config_dir = dirs::config_dir()
        .ok_or_else(|| TwfError::ConfigParseError("設定ディレクトリが見つかりません".to_string()))?;
    
    let config_path = config_dir.join("kitty").join("kitty.conf");
    
    // 設定ファイルが存在しない場合
    if !config_path.exists() {
        return Ok(None);
    }
    
    // 設定ファイルをパース
    let bg_image_path = parse_kitty_config(&config_path).await?;
    
    // 背景画像パスが見つかった場合
    if let Some(path) = bg_image_path {
        // パスを展開（~を展開）
        let expanded_path = expand_tilde(&path);
        
        // ファイルが存在するか確認
        if expanded_path.exists() {
            return Ok(Some(expanded_path));
        }
    }
    
    // 背景画像が見つからなかった
    Ok(None)
}

/// Kitty設定ファイルをパース
///
/// Kittyの設定ファイルは以下の形式です：
/// ```
/// background_image /path/to/image.png
/// background_image_layout tiled
/// background_opacity 0.9
/// ```
///
/// # Arguments
/// - `config_path`: 設定ファイルのパス
///
/// # Returns
/// - `Ok(Some(PathBuf))`: background_image設定が見つかった場合
/// - `Ok(None)`: background_image設定が見つからなかった場合
/// - `Err(TwfError)`: ファイルの読み込みに失敗した場合
async fn parse_kitty_config(config_path: &PathBuf) -> Result<Option<PathBuf>> {
    let content = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| TwfError::ConfigParseError(format!("設定ファイルの読み込みに失敗: {}", e)))?;
    
    // 各行を解析
    for line in content.lines() {
        // コメント行と空行をスキップ
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // "background_image" で始まる行を探す
        if line.starts_with("background_image") {
            // スペースで分割
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            // "background_image <path>" の形式を期待
            if parts.len() >= 2 && parts[0] == "background_image" {
                let path_str = parts[1];
                
                // "none" の場合はスキップ
                if path_str.to_lowercase() == "none" {
                    continue;
                }
                
                return Ok(Some(PathBuf::from(path_str)));
            }
        }
    }
    
    // background_image設定が見つからなかった
    Ok(None)
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
    
    /// テスト用のKitty設定ファイルを作成
    fn create_test_kitty_config(dir: &std::path::Path, has_background: bool) -> PathBuf {
        let config_dir = dir.join("kitty");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("kitty.conf");
        
        let content = if has_background {
            r#"
# Kitty設定ファイル

# フォント設定
font_family JetBrains Mono
font_size 12.0

# 背景画像設定
background_image /tmp/test_wallpaper.png
background_image_layout tiled
background_opacity 0.9

# カラースキーム
foreground #dddddd
background #000000
"#
        } else {
            r#"
# Kitty設定ファイル

# フォント設定
font_family JetBrains Mono
font_size 12.0

# 背景設定（画像なし）
background_opacity 0.9

# カラースキーム
foreground #dddddd
background #000000
"#
        };
        
        fs::write(&config_path, content).unwrap();
        config_path
    }
    
    /// テスト用のKitty設定ファイルを作成（background_image none）
    fn create_test_kitty_config_with_none(dir: &std::path::Path) -> PathBuf {
        let config_dir = dir.join("kitty");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("kitty.conf");
        
        let content = r#"
# Kitty設定ファイル

# 背景画像を無効化
background_image none
background_opacity 0.9
"#;
        
        fs::write(&config_path, content).unwrap();
        config_path
    }
    
    #[tokio::test]
    async fn test_parse_kitty_config_with_background() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_kitty_config(temp_dir.path(), true);
        
        let result = parse_kitty_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test_wallpaper.png"));
    }
    
    #[tokio::test]
    async fn test_parse_kitty_config_without_background() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_kitty_config(temp_dir.path(), false);
        
        let result = parse_kitty_config(&config_path).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_parse_kitty_config_with_none() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_kitty_config_with_none(temp_dir.path());
        
        let result = parse_kitty_config(&config_path).await.unwrap();
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
    
    #[tokio::test]
    async fn test_parse_kitty_config_with_comments() {
        // コメント行を含む設定ファイルをテスト
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("kitty");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("kitty.conf");
        let content = r#"
# これはコメント
# background_image /tmp/commented_out.png

# 実際の設定
background_image /tmp/actual_wallpaper.png
"#;
        fs::write(&config_path, content).unwrap();
        
        let result = parse_kitty_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/actual_wallpaper.png"));
    }
    
    #[tokio::test]
    async fn test_parse_kitty_config_with_tilde() {
        // チルダを含むパスをテスト
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("kitty");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("kitty.conf");
        let content = r#"
background_image ~/Pictures/wallpaper.png
"#;
        fs::write(&config_path, content).unwrap();
        
        let result = parse_kitty_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("~/Pictures/wallpaper.png"));
    }
}
