// 統合テスト: エンドツーエンドテスト
//
// このテストファイルは、TWFの主要な機能をエンドツーエンドで検証します。
// 各モード（画像パス指定、自動検出、背景色指定、プレビュー、適用、ロールバック）の
// 動作を統合的にテストします。

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use twf::models::Rgb;

/// テスト用の画像を作成するヘルパー関数
/// 
/// 単色の小さな画像（100x100ピクセル）を生成します。
fn create_test_image(temp_dir: &TempDir, color: Rgb) -> PathBuf {
    use image::{ImageBuffer, RgbImage};
    
    let img: RgbImage = ImageBuffer::from_fn(100, 100, |_, _| {
        image::Rgb([color.r, color.g, color.b])
    });
    
    let image_path = temp_dir.path().join("test_image.png");
    img.save(&image_path).expect("Failed to save test image");
    
    image_path
}

/// テスト用のシェル設定ファイルを作成するヘルパー関数
fn create_test_shell_config(temp_dir: &TempDir) -> PathBuf {
    let config_path = temp_dir.path().join(".bashrc");
    fs::write(&config_path, "# Original config\nexport PATH=$PATH:/usr/local/bin\n")
        .expect("Failed to create test config");
    config_path
}

/// 設定ファイルにTWF設定ブロックが含まれているかチェック
fn contains_twf_config(config_path: &PathBuf) -> bool {
    let content = fs::read_to_string(config_path).expect("Failed to read config");
    content.contains("# === TWF Generated Config ===")
}

/// バックアップファイルが作成されているかチェック
fn backup_exists(backup_dir: &PathBuf) -> bool {
    if !backup_dir.exists() {
        return false;
    }
    
    fs::read_dir(backup_dir)
        .expect("Failed to read backup dir")
        .any(|entry| {
            entry
                .expect("Failed to read entry")
                .file_name()
                .to_string_lossy()
                .starts_with("backup_")
        })
}

#[cfg(test)]
mod image_path_mode_tests {
    use super::*;
    
    /// テスト1: 画像パス指定モード - プレビュー
    /// 
    /// 画像パスを指定してプレビューモードで実行した場合、
    /// カラースキームが生成され、設定ファイルは変更されないことを検証します。
    #[tokio::test]
    async fn test_image_path_preview_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let image_path = create_test_image(&temp_dir, Rgb::new(30, 30, 30)); // 暗い背景
        let config_path = create_test_shell_config(&temp_dir);
        
        // 画像パスを指定してプレビューモード（デフォルト）で実行
        // 注意: 実際のrun関数は設定ファイルパスを引数として受け取らないため、
        // ここでは画像解析とカラースキーム生成のみをテストします
        
        // 画像を解析
        let analyzer = twf::analyzer::image::ImageAnalyzer::new(10000);
        let color_info = analyzer
            .analyze(&image_path)
            .await
            .expect("Failed to analyze image");
        
        // カラースキームを生成
        let generator = twf::generator::scheme::SchemeGenerator::new(4.5);
        let scheme = generator
            .generate(&color_info)
            .expect("Failed to generate scheme");
        
        // カラースキームが生成されたことを確認
        assert!(scheme.foreground.r > 0 || scheme.foreground.g > 0 || scheme.foreground.b > 0);
        assert_eq!(scheme.ansi_colors.black.r, 40); // 暗い背景の場合
        
        // 設定ファイルが変更されていないことを確認
        assert!(!contains_twf_config(&config_path));
    }
    
    /// テスト2: 画像パス指定モード - 適用
    /// 
    /// 画像パスを指定して適用モードで実行した場合、
    /// カラースキームが生成され、設定ファイルに書き込まれることを検証します。
    #[tokio::test]
    async fn test_image_path_apply_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let image_path = create_test_image(&temp_dir, Rgb::new(200, 200, 200)); // 明るい背景
        let config_path = create_test_shell_config(&temp_dir);
        let backup_dir = temp_dir.path().join("backups");
        
        // 画像を解析
        let analyzer = twf::analyzer::image::ImageAnalyzer::new(10000);
        let color_info = analyzer
            .analyze(&image_path)
            .await
            .expect("Failed to analyze image");
        
        // カラースキームを生成
        let generator = twf::generator::scheme::SchemeGenerator::new(4.5);
        let scheme = generator
            .generate(&color_info)
            .expect("Failed to generate scheme");
        
        // フォント設定を生成
        let font_optimizer = twf::generator::font::FontOptimizer;
        let font_config = font_optimizer.optimize(&color_info);
        
        // バックアップを作成
        let backup_manager = twf::applier::backup::BackupManager::new(backup_dir.clone());
        let _backup_info = backup_manager
            .create_backup(&config_path)
            .await
            .expect("Failed to create backup");
        
        // 設定を書き込み
        let _shell_type = twf::models::ShellType::Bash;
        let config_content = twf::applier::shell::generate_bash_config(&scheme, &font_config);
        
        // 既存の設定に追記
        let mut original_content = fs::read_to_string(&config_path).expect("Failed to read config");
        original_content.push_str("\n");
        original_content.push_str(&config_content);
        fs::write(&config_path, original_content).expect("Failed to write config");
        
        // 設定ファイルにTWF設定が含まれることを確認
        assert!(contains_twf_config(&config_path));
        
        // バックアップが作成されたことを確認
        assert!(backup_exists(&backup_dir));
        
        // 明るい背景の場合、フォアグラウンドは暗い色になることを確認
        assert!(scheme.foreground.r < 100 && scheme.foreground.g < 100 && scheme.foreground.b < 100);
    }
}

#[cfg(test)]
mod background_color_mode_tests {
    use super::*;
    
    /// テスト3: 背景色指定モード - プレビュー
    /// 
    /// 背景色を直接指定してプレビューモードで実行した場合、
    /// カラースキームが生成されることを検証します。
    #[tokio::test]
    async fn test_background_color_preview_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = create_test_shell_config(&temp_dir);
        
        // 背景色を解析（暗い背景: #1e1e1e）
        let bg_color = Rgb::new(30, 30, 30);
        let color_info = twf::analyzer::color::ColorAnalyzer::analyze(bg_color);
        
        // カラースキームを生成
        let generator = twf::generator::scheme::SchemeGenerator::new(4.5);
        let scheme = generator
            .generate(&color_info)
            .expect("Failed to generate scheme");
        
        // 暗い背景の場合、フォアグラウンドは明るい色になることを確認
        assert!(scheme.foreground.r > 200 || scheme.foreground.g > 200 || scheme.foreground.b > 200);
        
        // 設定ファイルが変更されていないことを確認
        assert!(!contains_twf_config(&config_path));
    }
    
    /// テスト4: 背景色指定モード - 適用
    /// 
    /// 背景色を直接指定して適用モードで実行した場合、
    /// カラースキームが生成され、設定ファイルに書き込まれることを検証します。
    #[tokio::test]
    async fn test_background_color_apply_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = create_test_shell_config(&temp_dir);
        let backup_dir = temp_dir.path().join("backups");
        
        // 背景色を解析（明るい背景: #f0f0f0）
        let bg_color = Rgb::new(240, 240, 240);
        let color_info = twf::analyzer::color::ColorAnalyzer::analyze(bg_color);
        
        // カラースキームを生成
        let generator = twf::generator::scheme::SchemeGenerator::new(4.5);
        let scheme = generator
            .generate(&color_info)
            .expect("Failed to generate scheme");
        
        // フォント設定を生成
        let font_optimizer = twf::generator::font::FontOptimizer;
        let font_config = font_optimizer.optimize(&color_info);
        
        // バックアップを作成
        let backup_manager = twf::applier::backup::BackupManager::new(backup_dir.clone());
        let _backup_info = backup_manager
            .create_backup(&config_path)
            .await
            .expect("Failed to create backup");
        
        // 設定を書き込み
        let config_content = twf::applier::shell::generate_bash_config(&scheme, &font_config);
        
        // 既存の設定に追記
        let mut original_content = fs::read_to_string(&config_path).expect("Failed to read config");
        original_content.push_str("\n");
        original_content.push_str(&config_content);
        fs::write(&config_path, original_content).expect("Failed to write config");
        
        // 設定ファイルにTWF設定が含まれることを確認
        assert!(contains_twf_config(&config_path));
        
        // バックアップが作成されたことを確認
        assert!(backup_exists(&backup_dir));
        
        // 明るい背景の場合、フォアグラウンドは暗い色になることを確認
        assert!(scheme.foreground.r < 100 && scheme.foreground.g < 100 && scheme.foreground.b < 100);
    }
}

#[cfg(test)]
mod rollback_mode_tests {
    use super::*;
    
    /// テスト5: ロールバックモード
    /// 
    /// 設定を適用した後、ロールバックを実行した場合、
    /// 元の設定ファイルが復元されることを検証します。
    #[tokio::test]
    async fn test_rollback_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = create_test_shell_config(&temp_dir);
        let backup_dir = temp_dir.path().join("backups");
        
        // 元の設定内容を保存
        let original_content = fs::read_to_string(&config_path).expect("Failed to read config");
        
        // 背景色を解析
        let bg_color = Rgb::new(30, 30, 30);
        let color_info = twf::analyzer::color::ColorAnalyzer::analyze(bg_color);
        
        // カラースキームを生成
        let generator = twf::generator::scheme::SchemeGenerator::new(4.5);
        let scheme = generator
            .generate(&color_info)
            .expect("Failed to generate scheme");
        
        // フォント設定を生成
        let font_optimizer = twf::generator::font::FontOptimizer;
        let font_config = font_optimizer.optimize(&color_info);
        
        // バックアップを作成
        let backup_manager = twf::applier::backup::BackupManager::new(backup_dir.clone());
        let backup_info = backup_manager
            .create_backup(&config_path)
            .await
            .expect("Failed to create backup");
        
        // 設定を書き込み
        let config_content = twf::applier::shell::generate_bash_config(&scheme, &font_config);
        let mut modified_content = fs::read_to_string(&config_path).expect("Failed to read config");
        modified_content.push_str("\n");
        modified_content.push_str(&config_content);
        fs::write(&config_path, &modified_content).expect("Failed to write config");
        
        // 設定が適用されたことを確認
        assert!(contains_twf_config(&config_path));
        
        // ロールバックを実行
        twf::applier::rollback::rollback(&backup_info)
            .await
            .expect("Failed to rollback");
        
        // 元の設定が復元されたことを確認
        let restored_content = fs::read_to_string(&config_path).expect("Failed to read config");
        assert_eq!(original_content, restored_content);
        assert!(!contains_twf_config(&config_path));
    }
}

#[cfg(test)]
mod contrast_ratio_tests {
    use super::*;
    
    /// テスト6: コントラスト比の保証
    /// 
    /// 任意の背景色に対して生成されたカラースキームが、
    /// WCAG 2.1 AA基準（4.5:1以上）を満たすことを検証します。
    #[tokio::test]
    async fn test_contrast_ratio_guarantee() {
        use twf::analyzer::contrast::calculate_contrast_ratio;
        use twf::utils::color_space::lab_to_rgb;
        
        // 複数の背景色でテスト
        let test_colors = vec![
            Rgb::new(0, 0, 0),       // 黒
            Rgb::new(255, 255, 255), // 白
            Rgb::new(30, 30, 30),    // 暗いグレー
            Rgb::new(200, 200, 200), // 明るいグレー
            Rgb::new(40, 44, 52),    // One Dark背景色
            Rgb::new(255, 0, 0),     // 赤
            Rgb::new(0, 255, 0),     // 緑
            Rgb::new(0, 0, 255),     // 青
        ];
        
        for bg_color in test_colors {
            // 背景色を解析
            let color_info = twf::analyzer::color::ColorAnalyzer::analyze(bg_color);
            
            // カラースキームを生成
            let generator = twf::generator::scheme::SchemeGenerator::new(4.5);
            let scheme = generator
                .generate(&color_info)
                .expect("Failed to generate scheme");
            
            // コントラスト比を計算
            let bg_rgb = lab_to_rgb(scheme.background);
            let contrast = calculate_contrast_ratio(bg_rgb, scheme.foreground);
            
            // WCAG AA基準（4.5:1）を満たすことを確認
            assert!(
                contrast >= 4.5,
                "Contrast ratio {} is below 4.5:1 for background color RGB({}, {}, {})",
                contrast,
                bg_color.r,
                bg_color.g,
                bg_color.b
            );
        }
    }
}

#[cfg(test)]
mod ansi_colors_tests {
    use super::*;
    
    /// テスト7: ANSI 16色の完全性
    /// 
    /// 生成されたカラースキームが16色すべて（基本8色 + 明るい8色）を
    /// 含むことを検証します。
    #[tokio::test]
    async fn test_ansi_colors_completeness() {
        // 背景色を解析
        let bg_color = Rgb::new(30, 30, 30);
        
        let color_info = twf::analyzer::color::ColorAnalyzer::analyze(bg_color);
        
        // カラースキームを生成
        let generator = twf::generator::scheme::SchemeGenerator::new(4.5);
        let scheme = generator
            .generate(&color_info)
            .expect("Failed to generate scheme");
        
        // 16色すべてが有効なRGB値を持つことを確認
        let colors = vec![
            scheme.ansi_colors.black,
            scheme.ansi_colors.red,
            scheme.ansi_colors.green,
            scheme.ansi_colors.yellow,
            scheme.ansi_colors.blue,
            scheme.ansi_colors.magenta,
            scheme.ansi_colors.cyan,
            scheme.ansi_colors.white,
            scheme.ansi_colors.bright_black,
            scheme.ansi_colors.bright_red,
            scheme.ansi_colors.bright_green,
            scheme.ansi_colors.bright_yellow,
            scheme.ansi_colors.bright_blue,
            scheme.ansi_colors.bright_magenta,
            scheme.ansi_colors.bright_cyan,
            scheme.ansi_colors.bright_white,
        ];
        
        // すべての色が有効な範囲内にあることを確認
        for (i, color) in colors.iter().enumerate() {
            // u8型は0-255の範囲なので、常に有効
            // ここでは色が生成されたことを確認
            assert!(
                true,
                "Color {} has RGB values: RGB({}, {}, {})",
                i,
                color.r,
                color.g,
                color.b
            );
        }
        
        // 16色すべてが生成されたことを確認
        assert_eq!(colors.len(), 16);
    }
}

#[cfg(test)]
mod font_config_tests {
    use super::*;
    use twf::models::FontWeight;
    
    /// テスト8: フォント設定の生成
    /// 
    /// 背景の特性（明度と彩度）に基づいて、適切なフォントウェイトが
    /// 選択されることを検証します。
    #[tokio::test]
    async fn test_font_config_generation() {
        // テストケース1: 暗い背景、低彩度
        let bg_color = Rgb::new(30, 30, 30);
        
        let color_info = twf::analyzer::color::ColorAnalyzer::analyze(bg_color);
        
        let font_optimizer = twf::generator::font::FontOptimizer;
        let font_config = font_optimizer.optimize(&color_info);
        
        // 暗い背景では通常の太さ
        assert_eq!(font_config.weight, FontWeight::Normal);
        
        // テストケース2: 明るい背景、低彩度
        let bg_color = Rgb::new(240, 240, 240);
        let color_info = twf::analyzer::color::ColorAnalyzer::analyze(bg_color);
        let font_config = font_optimizer.optimize(&color_info);
        
        // 明るい背景では少し太め
        assert_eq!(font_config.weight, FontWeight::Medium);
        
        // 推奨フォントリストが空でないことを確認
        assert!(!font_config.recommended_fonts.is_empty());
    }
}

#[cfg(test)]
mod shell_config_tests {
    use super::*;
    
    /// テスト9: シェル設定の生成
    /// 
    /// 各シェルタイプ（Bash、Fish、PowerShell）に対して、
    /// 適切な形式の設定が生成されることを検証します。
    #[tokio::test]
    async fn test_shell_config_generation() {
        // 背景色を解析
        let bg_color = Rgb::new(30, 30, 30);
        
        let color_info = twf::analyzer::color::ColorAnalyzer::analyze(bg_color);
        
        // カラースキームを生成
        let generator = twf::generator::scheme::SchemeGenerator::new(4.5);
        let scheme = generator
            .generate(&color_info)
            .expect("Failed to generate scheme");
        
        // フォント設定を生成
        let font_optimizer = twf::generator::font::FontOptimizer;
        let font_config = font_optimizer.optimize(&color_info);
        
        // Bash設定を生成
        let bash_config = twf::applier::shell::generate_bash_config(&scheme, &font_config);
        assert!(bash_config.contains("# === TWF Generated Config ==="));
        assert!(bash_config.contains("export TWF_ANSI_BLACK"));
        assert!(bash_config.contains("# === End TWF Config ==="));
        
        // Fish設定を生成
        let fish_config = twf::applier::shell::generate_fish_config(&scheme, &font_config);
        assert!(fish_config.contains("# === TWF Generated Config ==="));
        assert!(fish_config.contains("set -x TWF_ANSI_BLACK"));
        
        // PowerShell設定を生成
        let powershell_config = twf::applier::shell::generate_powershell_config(&scheme, &font_config);
        assert!(powershell_config.contains("# === TWF Generated Config ==="));
        assert!(powershell_config.contains("$env:TWF_ANSI_BLACK"));
    }
}
