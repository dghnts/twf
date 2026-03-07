// GNOME Terminal設定パーサー

use crate::models::{Result, TwfError};
use std::path::PathBuf;

/// GNOME Terminalの背景画像を検出
///
/// GNOME Terminalは標準では背景画像をサポートしていません。
/// dconf/gsettingsを使用して設定を確認しますが、通常は背景画像設定は存在しません。
///
/// # 処理フロー
/// 1. dconf/gsettingsコマンドを使用して設定を取得
/// 2. 背景画像パスを取得（存在する場合）
/// 3. 画像パスが存在する場合は返す
///
/// # Returns
/// - `Ok(Some(PathBuf))`: 背景画像パスが検出された場合（稀）
/// - `Ok(None)`: 背景画像が設定されていない場合（通常）
/// - `Err(TwfError)`: 設定の取得に失敗した場合
///
/// # Requirements
/// - 2.8.1: GNOME Terminalの設定から背景画像パスを取得できること
/// - 2.8.2: dconf/gsettingsからの設定取得を実装
pub async fn detect_gnome_terminal_background() -> Result<Option<PathBuf>> {
    // GNOME Terminalは標準では背景画像をサポートしていないため、
    // 基本的にはNoneを返します。
    // 
    // 将来的に拡張機能やカスタム設定で背景画像がサポートされる可能性があるため、
    // 以下のような実装を残しておきます：
    
    // dconfコマンドが利用可能か確認
    if !is_dconf_available().await {
        // dconfが利用できない場合はNoneを返す
        return Ok(None);
    }
    
    // GNOME Terminalのプロファイルリストを取得
    let profiles = get_gnome_terminal_profiles().await?;
    
    // 各プロファイルから背景画像設定を確認
    for profile in profiles {
        if let Some(bg_image) = get_background_image_for_profile(&profile).await? {
            // 背景画像パスが存在する場合は返す
            if bg_image.exists() {
                return Ok(Some(bg_image));
            }
        }
    }
    
    // 背景画像が見つからなかった（通常のケース）
    Ok(None)
}

/// dconfコマンドが利用可能か確認
async fn is_dconf_available() -> bool {
    tokio::process::Command::new("which")
        .arg("dconf")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// GNOME Terminalのプロファイルリストを取得
async fn get_gnome_terminal_profiles() -> Result<Vec<String>> {
    // dconfコマンドでプロファイルリストを取得
    let output = tokio::process::Command::new("dconf")
        .arg("list")
        .arg("/org/gnome/terminal/legacy/profiles:/")
        .output()
        .await
        .map_err(|e| TwfError::ConfigParseError(format!("dconfコマンドの実行に失敗: {}", e)))?;
    
    if !output.status.success() {
        return Ok(Vec::new());
    }
    
    // 出力をパース
    let stdout = String::from_utf8_lossy(&output.stdout);
    let profiles: Vec<String> = stdout
        .lines()
        .filter(|line| line.ends_with('/'))
        .map(|line| line.trim_end_matches('/').to_string())
        .collect();
    
    Ok(profiles)
}

/// 指定されたプロファイルの背景画像パスを取得
async fn get_background_image_for_profile(profile: &str) -> Result<Option<PathBuf>> {
    // GNOME Terminalの標準設定には背景画像の項目がないため、
    // カスタム設定や拡張機能で追加されている可能性のあるキーを確認
    let possible_keys = vec![
        "background-image",
        "background-image-file",
        "background-picture",
    ];
    
    for key in possible_keys {
        let path_str = format!("/org/gnome/terminal/legacy/profiles:/{}/{}", profile, key);
        
        let output = tokio::process::Command::new("dconf")
            .arg("read")
            .arg(&path_str)
            .output()
            .await
            .map_err(|e| TwfError::ConfigParseError(format!("dconfコマンドの実行に失敗: {}", e)))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let value = stdout.trim();
            
            // 値が存在し、空でない場合
            if !value.is_empty() && value != "''" {
                // シングルクォートを削除
                let path_str = value.trim_matches('\'');
                let path = PathBuf::from(path_str);
                
                // チルダを展開
                let expanded_path = expand_tilde(&path);
                
                return Ok(Some(expanded_path));
            }
        }
    }
    
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
    
    #[tokio::test]
    async fn test_detect_gnome_terminal_background() {
        // GNOME Terminalは通常背景画像をサポートしていないため、
        // Noneが返されることを確認
        let result = detect_gnome_terminal_background().await;
        
        // エラーが発生しないことを確認
        assert!(result.is_ok());
        
        // 通常はNoneが返される
        // （環境によっては背景画像が設定されている可能性もあるため、
        //  Noneであることを強制はしない）
    }
    
    #[tokio::test]
    async fn test_is_dconf_available() {
        // dconfコマンドの存在確認
        let available = is_dconf_available().await;
        
        // Linux環境でGNOMEがインストールされている場合はtrueになる可能性がある
        // テスト環境に依存するため、結果の値は確認しない
        println!("dconf available: {}", available);
    }
    
    #[test]
    fn test_expand_tilde() {
        let path = PathBuf::from("~/Pictures/wallpaper.png");
        let expanded = expand_tilde(&path);
        
        // チルダが展開されていることを確認
        if dirs::home_dir().is_some() {
            assert!(!expanded.to_str().unwrap().starts_with("~"));
        }
        
        // 絶対パスの場合は変更されない
        let abs_path = PathBuf::from("/tmp/wallpaper.png");
        let expanded_abs = expand_tilde(&abs_path);
        assert_eq!(abs_path, expanded_abs);
    }
    
    #[tokio::test]
    async fn test_get_gnome_terminal_profiles() {
        // プロファイルリストの取得をテスト
        let result = get_gnome_terminal_profiles().await;
        
        // dconfが利用できない環境ではエラーになる可能性があるため、
        // エラーの場合はスキップ
        if let Ok(profiles) = result {
            println!("Found {} GNOME Terminal profiles", profiles.len());
            for profile in profiles {
                println!("  - {}", profile);
            }
        }
    }
}
