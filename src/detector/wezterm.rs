// WezTerm設定パーサー

use crate::models::{Result, TwfError};
use std::path::PathBuf;
use regex::Regex;

/// WezTermの背景画像を検出
///
/// WezTermの設定ファイル（wezterm.lua）から背景画像のパスを取得します。
/// 設定ファイルは以下の場所を順に確認します：
/// 1. ~/.config/wezterm/wezterm.lua
/// 2. ~/.wezterm.lua
///
/// # 処理フロー
/// 1. 設定ファイルのパスを取得
/// 2. Lua設定ファイルをパース（background_imageパターンを検索）
/// 3. 背景画像パスを取得
/// 4. 画像パスが存在する場合は返す
///
/// # Returns
/// - `Ok(Some(PathBuf))`: 背景画像が設定されている場合、そのパスを返す
/// - `Ok(None)`: 背景画像が設定されていない場合
/// - `Err(TwfError)`: 設定ファイルのパースに失敗した場合
///
/// # Requirements
/// - 2.8.1: WezTermの設定から背景画像パスを取得できること
/// - 2.8.2: WezTerm設定ファイルのパース処理を実装
///
/// # 注意
/// WezTermの設定はLuaスクリプトなので、完全なLuaパーサーではなく、
/// 正規表現やパターンマッチングで`background_image`設定を抽出する簡易的な実装です。
pub async fn detect_wezterm_background() -> Result<Option<PathBuf>> {
    // 設定ファイルのパスを取得
    let config_paths = get_wezterm_config_paths()?;
    
    // 各設定ファイルを順に確認
    for config_path in config_paths {
        if !config_path.exists() {
            continue;
        }
        
        // 設定ファイルをパース
        let bg_image_path = parse_wezterm_config(&config_path).await?;
        
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

/// WezTerm設定ファイルのパスを取得
///
/// 以下の順序で設定ファイルを確認します：
/// 1. ~/.config/wezterm/wezterm.lua
/// 2. ~/.wezterm.lua
fn get_wezterm_config_paths() -> Result<Vec<PathBuf>> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| TwfError::ConfigParseError("ホームディレクトリが見つかりません".to_string()))?;
    
    let config_dir = dirs::config_dir()
        .ok_or_else(|| TwfError::ConfigParseError("設定ディレクトリが見つかりません".to_string()))?;
    
    Ok(vec![
        config_dir.join("wezterm").join("wezterm.lua"),
        home_dir.join(".wezterm.lua"),
    ])
}

/// WezTerm設定ファイルをパース
///
/// WezTermの設定ファイルはLuaスクリプトです。
/// 以下のようなパターンで背景画像が設定されます：
///
/// ```lua
/// config.background = {
///   {
///     source = { File = "/path/to/image.png" },
///   },
/// }
/// ```
///
/// または：
///
/// ```lua
/// config.background = {
///   {
///     source = { File = { path = "/path/to/image.png" } },
///   },
/// }
/// ```
///
/// # Arguments
/// - `config_path`: 設定ファイルのパス
///
/// # Returns
/// - `Ok(Some(PathBuf))`: background設定が見つかった場合
/// - `Ok(None)`: background設定が見つからなかった場合
/// - `Err(TwfError)`: ファイルの読み込みに失敗した場合
async fn parse_wezterm_config(config_path: &PathBuf) -> Result<Option<PathBuf>> {
    let content = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| TwfError::ConfigParseError(format!("設定ファイルの読み込みに失敗: {}", e)))?;
    
    // 正規表現パターンを定義
    // パターン1: source = { File = "/path/to/image.png" }
    // パターン2: source = { File = { path = "/path/to/image.png" } }
    // パターン3: source = { File = '/path/to/image.png' }
    let patterns = vec![
        r#"File\s*=\s*"([^"]+)""#,  // ダブルクォート
        r#"File\s*=\s*'([^']+)'"#,  // シングルクォート
        r#"path\s*=\s*"([^"]+)""#,  // path = "..." 形式
        r#"path\s*=\s*'([^']+)'"#,  // path = '...' 形式
    ];
    
    // 各行を解析（コメント行を除外）
    for line in content.lines() {
        let line = line.trim();
        
        // コメント行と空行をスキップ
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        
        // 各パターンを試す
        for pattern in &patterns {
            let re = Regex::new(pattern)
                .map_err(|e| TwfError::ConfigParseError(format!("正規表現のコンパイルに失敗: {}", e)))?;
            
            if let Some(captures) = re.captures(line) {
                if let Some(path_match) = captures.get(1) {
                    let path_str = path_match.as_str();
                    return Ok(Some(PathBuf::from(path_str)));
                }
            }
        }
    }
    
    // background設定が見つからなかった
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
    
    /// テスト用のWezTerm設定ファイルを作成（パターン1: File = "path"）
    fn create_test_wezterm_config_pattern1(dir: &std::path::Path) -> PathBuf {
        let config_dir = dir.join("wezterm");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("wezterm.lua");
        
        let content = r#"
local wezterm = require 'wezterm'
local config = {}

-- 背景画像設定
config.background = {
  {
    source = { File = "/tmp/test_wallpaper.png" },
    hsb = { brightness = 0.05 },
  },
}

-- フォント設定
config.font = wezterm.font 'JetBrains Mono'
config.font_size = 12.0

return config
"#;
        
        fs::write(&config_path, content).unwrap();
        config_path
    }
    
    /// テスト用のWezTerm設定ファイルを作成（パターン2: File = { path = "..." }）
    fn create_test_wezterm_config_pattern2(dir: &std::path::Path) -> PathBuf {
        let config_dir = dir.join("wezterm");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("wezterm.lua");
        
        let content = r#"
local wezterm = require 'wezterm'
local config = {}

-- 背景画像設定
config.background = {
  {
    source = { File = { path = "/tmp/test_wallpaper2.png" } },
    opacity = 0.9,
  },
}

return config
"#;
        
        fs::write(&config_path, content).unwrap();
        config_path
    }
    
    /// テスト用のWezTerm設定ファイルを作成（シングルクォート）
    fn create_test_wezterm_config_single_quote(dir: &std::path::Path) -> PathBuf {
        let config_dir = dir.join("wezterm");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("wezterm.lua");
        
        let content = r#"
local wezterm = require 'wezterm'
local config = {}

-- 背景画像設定
config.background = {
  {
    source = { File = '/tmp/test_wallpaper3.png' },
  },
}

return config
"#;
        
        fs::write(&config_path, content).unwrap();
        config_path
    }
    
    /// テスト用のWezTerm設定ファイルを作成（背景画像なし）
    fn create_test_wezterm_config_no_background(dir: &std::path::Path) -> PathBuf {
        let config_dir = dir.join("wezterm");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("wezterm.lua");
        
        let content = r#"
local wezterm = require 'wezterm'
local config = {}

-- フォント設定のみ
config.font = wezterm.font 'JetBrains Mono'
config.font_size = 12.0

return config
"#;
        
        fs::write(&config_path, content).unwrap();
        config_path
    }
    
    #[tokio::test]
    async fn test_parse_wezterm_config_pattern1() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_wezterm_config_pattern1(temp_dir.path());
        
        let result = parse_wezterm_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test_wallpaper.png"));
    }
    
    #[tokio::test]
    async fn test_parse_wezterm_config_pattern2() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_wezterm_config_pattern2(temp_dir.path());
        
        let result = parse_wezterm_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test_wallpaper2.png"));
    }
    
    #[tokio::test]
    async fn test_parse_wezterm_config_single_quote() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_wezterm_config_single_quote(temp_dir.path());
        
        let result = parse_wezterm_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test_wallpaper3.png"));
    }
    
    #[tokio::test]
    async fn test_parse_wezterm_config_no_background() {
        let temp_dir = tempdir().unwrap();
        let config_path = create_test_wezterm_config_no_background(temp_dir.path());
        
        let result = parse_wezterm_config(&config_path).await.unwrap();
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
    async fn test_parse_wezterm_config_with_tilde() {
        // チルダを含むパスをテスト
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("wezterm");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("wezterm.lua");
        let content = r#"
config.background = {
  {
    source = { File = "~/Pictures/wallpaper.png" },
  },
}
"#;
        fs::write(&config_path, content).unwrap();
        
        let result = parse_wezterm_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("~/Pictures/wallpaper.png"));
    }
    
    #[tokio::test]
    async fn test_parse_wezterm_config_with_comments() {
        // コメント行を含む設定ファイルをテスト
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("wezterm");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("wezterm.lua");
        let content = r#"
-- これはコメント
-- source = { File = "/tmp/commented_out.png" }

-- 実際の設定
config.background = {
  {
    source = { File = "/tmp/actual_wallpaper.png" },
  },
}
"#;
        fs::write(&config_path, content).unwrap();
        
        let result = parse_wezterm_config(&config_path).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/actual_wallpaper.png"));
    }
    
    #[test]
    fn test_get_wezterm_config_paths() {
        let paths = get_wezterm_config_paths().unwrap();
        
        // 2つのパスが返されることを確認
        assert_eq!(paths.len(), 2);
        
        // 最初のパスは ~/.config/wezterm/wezterm.lua
        assert!(paths[0].to_str().unwrap().contains("wezterm.lua"));
        
        // 2番目のパスは ~/.wezterm.lua
        assert!(paths[1].to_str().unwrap().ends_with(".wezterm.lua"));
    }
}
