// TWF (Terminal Wallpaper Fit) - メインエントリーポイント

mod cli;
mod detector;
mod analyzer;
mod generator;
mod applier;
mod preview;
mod models;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::CliArgs;
use models::InputSource;

#[tokio::main]
async fn main() -> Result<()> {
    // CLI引数をパース
    let args = CliArgs::parse();
    
    // 詳細出力モードの設定
    if args.verbose {
        println!("🔍 詳細出力モードが有効です");
        println!("引数: {:?}", args);
    }
    
    // メイン処理フローを実行
    run(args).await
}

/// メイン処理フロー
/// 
/// 入力ソースの決定 → 色情報の取得 → カラースキーム生成 → モード別処理
/// フォールバック戦略を実装（画像検出 → 背景色検出 → デフォルト色）
/// 
/// # Arguments
/// 
/// * `args` - CLI引数
/// 
/// # Returns
/// 
/// 処理結果（成功時はOk(())、失敗時はエラー）
/// 
/// # Requirements
/// 
/// - 2.1.6: 背景画像が設定されていない場合、ターミナルの現在の背景色を検出できること
/// - 2.1.9: 背景色の検出に失敗した場合は、デフォルトの背景色を仮定して処理を継続できること
/// - 2.8.4: 画像パスが引数で指定されていない場合、自動検出を試みること
async fn run(args: CliArgs) -> Result<()> {
    use analyzer::color::ColorAnalyzer;
    use analyzer::image::ImageAnalyzer;
    use applier::shell::ConfigApplier;
    use detector::auto::AutoDetector;
    use detector::bg_color::BgColorDetector;
    use generator::font::FontOptimizer;
    use generator::scheme::SchemeGenerator;
    use models::Rgb;
    use preview::PreviewRenderer;
    use std::time::Duration;
    
    println!("TWF (Terminal Wallpaper Fit) v{}", env!("CARGO_PKG_VERSION"));
    println!();
    
    // 1. 入力ソースの決定
    let input_source = determine_input_source(&args);
    
    // 2. 色情報の取得（フォールバック戦略を実装）
    let color_info = match input_source {
        InputSource::ImagePath(ref path) => {
            // 画像パスが指定されている場合
            if args.verbose {
                println!("📁 画像を解析中: {}", path.display());
            } else {
                println!("🎨 画像を解析中...");
            }
            
            let analyzer = ImageAnalyzer::new(10000);
            analyzer.analyze(path).await?
        }
        InputSource::Color(ref color_str) => {
            // 背景色が指定されている場合
            if args.verbose {
                println!("🎨 背景色を解析中: {}", color_str);
            } else {
                println!("🎨 背景色を解析中...");
            }
            
            let color = parse_color(color_str)?;
            ColorAnalyzer::analyze(color)
        }
        InputSource::AutoDetect => {
            // 自動検出モード（フォールバック戦略）
            if args.verbose {
                println!("🔍 自動検出モードを開始");
            }
            
            // ステップ1: 背景画像の自動検出を試みる
            println!("🔍 ターミナルを検出中...");
            let detector = AutoDetector::new();
            
            if args.verbose {
                println!("検出されたターミナル: {:?}", detector.terminal_type());
            }
            
            println!("🔍 背景画像を検出中...");
            match detector.detect_background_image().await {
                Ok(Some(image_path)) => {
                    // 背景画像が検出された場合
                    println!("✓ 背景画像を検出: {}", image_path.display());
                    println!("🎨 画像を解析中...");
                    
                    let analyzer = ImageAnalyzer::new(10000);
                    analyzer.analyze(&image_path).await?
                }
                Ok(None) | Err(_) => {
                    // 背景画像が検出できなかった場合
                    if args.verbose {
                        println!("背景画像が検出できませんでした。背景色検出にフォールバック...");
                    }
                    
                    // ステップ2: 背景色の検出を試みる
                    println!("🔍 背景色を検出中...");
                    let bg_detector = BgColorDetector::new(Duration::from_millis(1000));
                    
                    match bg_detector.detect_background_color() {
                        Ok(Some(bg_color)) => {
                            // 背景色が検出された場合
                            println!("✓ 背景色を検出: RGB({}, {}, {})", bg_color.r, bg_color.g, bg_color.b);
                            println!("🎨 背景色を解析中...");
                            
                            ColorAnalyzer::analyze(bg_color)
                        }
                        Ok(None) | Err(_) => {
                            // 背景色も検出できなかった場合
                            if args.verbose {
                                println!("背景色が検出できませんでした。デフォルト背景色を使用...");
                            }
                            
                            // ステップ3: デフォルト背景色を使用
                            println!("⚠ 背景情報を検出できませんでした。デフォルト背景色（黒）を使用します。");
                            let default_color = Rgb::new(0, 0, 0); // 黒
                            ColorAnalyzer::analyze(default_color)
                        }
                    }
                }
            }
        }
    };
    
    if args.verbose {
        println!("✓ 色情報を取得しました");
        println!("  明度: {:.2}", color_info.average_lightness);
        println!("  彩度: {:.2}", color_info.saturation);
        println!("  色相: {:.2}", color_info.hue);
        println!("  暗い背景: {}", color_info.is_dark);
    } else {
        println!("✓ 解析完了");
    }
    println!();
    
    // 3. カラースキームとフォント設定の生成
    if args.verbose {
        println!("🎨 カラースキームを生成中...");
    }
    
    let scheme_generator = SchemeGenerator::default();
    let scheme = scheme_generator.generate(&color_info)?;
    
    if args.verbose {
        println!("✓ カラースキームを生成しました");
    }
    
    let font_optimizer = FontOptimizer;
    let font_config = font_optimizer.optimize(&color_info);
    
    if args.verbose {
        println!("✓ フォント設定を生成しました");
        println!();
    }
    
    // 4. モードに応じた処理
    if args.rollback {
        // ロールバックモード
        println!("🔄 設定をロールバック中...");
        
        use applier::backup::BackupManager;
        use applier::rollback::rollback;
        
        let backup_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("twf")
            .join("backups");
        
        let backup_manager = BackupManager::new(backup_dir);
        
        match backup_manager.get_latest_backup().await? {
            Some(backup_info) => {
                rollback(&backup_info).await?;
                println!("✓ ロールバック完了: {}", backup_info.backup_path.display());
            }
            None => {
                println!("⚠ バックアップが見つかりませんでした。");
            }
        }
    } else if args.apply {
        // 適用モード
        let backup_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("twf")
            .join("backups");
        
        let applier = ConfigApplier::new(backup_dir);
        applier.apply(&scheme, &font_config).await?;
    } else {
        // プレビューモード（デフォルト）
        PreviewRenderer::render(&scheme, &font_config);
        
        println!("設定を適用するには、以下のコマンドを実行してください:");
        println!("  twf --apply");
        println!();
        println!("または、画像パスを指定して適用:");
        println!("  twf --image <画像パス> --apply");
    }
    
    Ok(())
}

/// 色文字列をパースしてRGB色に変換
/// 
/// サポートされる形式:
/// - "#RRGGBB" (例: "#1e1e1e")
/// - "rgb(R, G, B)" (例: "rgb(30, 30, 30)")
/// 
/// # Arguments
/// 
/// * `color_str` - 色文字列
/// 
/// # Returns
/// 
/// パースされたRGB色
fn parse_color(color_str: &str) -> Result<models::Rgb> {
    use models::{Rgb, TwfError};
    
    let color_str = color_str.trim();
    
    // "#RRGGBB" 形式
    if color_str.starts_with('#') {
        let hex = color_str.trim_start_matches('#');
        
        if hex.len() != 6 {
            return Err(TwfError::ParseError(format!(
                "不正な色形式: {}。#RRGGBBの形式で指定してください。",
                color_str
            ))
            .into());
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| TwfError::ParseError(format!("不正な16進数: {}", &hex[0..2])))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| TwfError::ParseError(format!("不正な16進数: {}", &hex[2..4])))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| TwfError::ParseError(format!("不正な16進数: {}", &hex[4..6])))?;
        
        return Ok(Rgb::new(r, g, b));
    }
    
    // "rgb(R, G, B)" 形式
    if color_str.starts_with("rgb(") && color_str.ends_with(')') {
        let rgb_str = color_str
            .trim_start_matches("rgb(")
            .trim_end_matches(')');
        
        let parts: Vec<&str> = rgb_str.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 3 {
            return Err(TwfError::ParseError(format!(
                "不正な色形式: {}。rgb(R, G, B)の形式で指定してください。",
                color_str
            ))
            .into());
        }
        
        let r = parts[0]
            .parse::<u8>()
            .map_err(|_| TwfError::ParseError(format!("不正な数値: {}", parts[0])))?;
        let g = parts[1]
            .parse::<u8>()
            .map_err(|_| TwfError::ParseError(format!("不正な数値: {}", parts[1])))?;
        let b = parts[2]
            .parse::<u8>()
            .map_err(|_| TwfError::ParseError(format!("不正な数値: {}", parts[2])))?;
        
        return Ok(Rgb::new(r, g, b));
    }
    
    Err(TwfError::ParseError(format!(
        "不正な色形式: {}。#RRGGBBまたはrgb(R, G, B)の形式で指定してください。",
        color_str
    ))
    .into())
}

/// 入力ソースを決定する
/// 
/// 優先順位:
/// 1. --image オプションで画像パスが指定されている場合 → ImagePath
/// 2. --color オプションで背景色が指定されている場合 → Color
/// 3. --detect オプションが指定されている場合、または何も指定されていない場合 → AutoDetect
/// 
/// # Arguments
/// 
/// * `args` - CLI引数
/// 
/// # Returns
/// 
/// 入力ソース（ImagePath、AutoDetect、Color）
/// 
/// # Requirements
/// 
/// - 2.7.2: 画像パスをオプション引数として受け取れること
/// - 2.7.3: 自動検出を強制できること
/// - 2.7.4: 背景色を直接指定できること
fn determine_input_source(args: &CliArgs) -> InputSource {
    // 優先順位1: 画像パスが指定されている場合
    if let Some(ref image_path) = args.image {
        if args.verbose {
            println!("📁 入力ソース: 画像パス指定 ({})", image_path.display());
        }
        return InputSource::ImagePath(image_path.clone());
    }
    
    // 優先順位2: 背景色が指定されている場合
    if let Some(ref color) = args.color {
        if args.verbose {
            println!("🎨 入力ソース: 背景色指定 ({})", color);
        }
        return InputSource::Color(color.clone());
    }
    
    // 優先順位3: 自動検出が指定されている場合、または何も指定されていない場合
    if args.detect || (args.image.is_none() && args.color.is_none()) {
        if args.verbose {
            println!("🔍 入力ソース: 自動検出");
        }
        return InputSource::AutoDetect;
    }
    
    // デフォルト: 自動検出
    if args.verbose {
        println!("🔍 入力ソース: 自動検出（デフォルト）");
    }
    InputSource::AutoDetect
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    /// テスト用のCliArgs構造体を作成するヘルパー関数
    fn create_test_args() -> CliArgs {
        CliArgs {
            image: None,
            detect: false,
            color: None,
            preview: false,
            apply: false,
            rollback: false,
            verbose: false,
        }
    }
    
    #[test]
    fn test_determine_input_source_with_image_path() {
        // 画像パスが指定されている場合
        let mut args = create_test_args();
        args.image = Some(PathBuf::from("/path/to/image.png"));
        
        let source = determine_input_source(&args);
        
        match source {
            InputSource::ImagePath(path) => {
                assert_eq!(path, PathBuf::from("/path/to/image.png"));
            }
            _ => panic!("Expected ImagePath, got {:?}", source),
        }
    }
    
    #[test]
    fn test_determine_input_source_with_color() {
        // 背景色が指定されている場合
        let mut args = create_test_args();
        args.color = Some("#1e1e1e".to_string());
        
        let source = determine_input_source(&args);
        
        match source {
            InputSource::Color(color) => {
                assert_eq!(color, "#1e1e1e");
            }
            _ => panic!("Expected Color, got {:?}", source),
        }
    }
    
    #[test]
    fn test_determine_input_source_with_detect_flag() {
        // --detect フラグが指定されている場合
        let mut args = create_test_args();
        args.detect = true;
        
        let source = determine_input_source(&args);
        
        match source {
            InputSource::AutoDetect => {
                // 正しく自動検出が選択された
            }
            _ => panic!("Expected AutoDetect, got {:?}", source),
        }
    }
    
    #[test]
    fn test_determine_input_source_default() {
        // 何も指定されていない場合（デフォルト）
        let args = create_test_args();
        
        let source = determine_input_source(&args);
        
        match source {
            InputSource::AutoDetect => {
                // 正しく自動検出が選択された（デフォルト）
            }
            _ => panic!("Expected AutoDetect, got {:?}", source),
        }
    }
    
    #[test]
    fn test_determine_input_source_priority_image_over_color() {
        // 画像パスと背景色の両方が指定されている場合、画像パスが優先される
        let mut args = create_test_args();
        args.image = Some(PathBuf::from("/path/to/image.png"));
        args.color = Some("#1e1e1e".to_string());
        
        let source = determine_input_source(&args);
        
        match source {
            InputSource::ImagePath(path) => {
                assert_eq!(path, PathBuf::from("/path/to/image.png"));
            }
            _ => panic!("Expected ImagePath (priority over Color), got {:?}", source),
        }
    }
    
    #[test]
    fn test_determine_input_source_priority_image_over_detect() {
        // 画像パスと--detectの両方が指定されている場合、画像パスが優先される
        let mut args = create_test_args();
        args.image = Some(PathBuf::from("/path/to/image.png"));
        args.detect = true;
        
        let source = determine_input_source(&args);
        
        match source {
            InputSource::ImagePath(path) => {
                assert_eq!(path, PathBuf::from("/path/to/image.png"));
            }
            _ => panic!("Expected ImagePath (priority over detect), got {:?}", source),
        }
    }
    
    #[test]
    fn test_determine_input_source_priority_color_over_detect() {
        // 背景色と--detectの両方が指定されている場合、背景色が優先される
        let mut args = create_test_args();
        args.color = Some("#1e1e1e".to_string());
        args.detect = true;
        
        let source = determine_input_source(&args);
        
        match source {
            InputSource::Color(color) => {
                assert_eq!(color, "#1e1e1e");
            }
            _ => panic!("Expected Color (priority over detect), got {:?}", source),
        }
    }
    
    #[test]
    fn test_parse_color_hex_format() {
        // "#RRGGBB" 形式のパース
        let color = parse_color("#1e1e1e").unwrap();
        assert_eq!(color.r, 30);
        assert_eq!(color.g, 30);
        assert_eq!(color.b, 30);
        
        let color = parse_color("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        
        let color = parse_color("#00ff00").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
        
        let color = parse_color("#0000ff").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 255);
    }
    
    #[test]
    fn test_parse_color_rgb_format() {
        // "rgb(R, G, B)" 形式のパース
        let color = parse_color("rgb(30, 30, 30)").unwrap();
        assert_eq!(color.r, 30);
        assert_eq!(color.g, 30);
        assert_eq!(color.b, 30);
        
        let color = parse_color("rgb(255, 0, 0)").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        
        // スペースなし
        let color = parse_color("rgb(0,255,0)").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
    }
    
    #[test]
    fn test_parse_color_invalid_format() {
        // 不正な形式
        assert!(parse_color("invalid").is_err());
        assert!(parse_color("#12345").is_err()); // 5桁
        assert!(parse_color("#1234567").is_err()); // 7桁
        assert!(parse_color("rgb(255, 255)").is_err()); // 2つの値
        assert!(parse_color("rgb(255, 255, 255, 255)").is_err()); // 4つの値
        assert!(parse_color("rgb(256, 0, 0)").is_err()); // 範囲外
        assert!(parse_color("#gggggg").is_err()); // 不正な16進数
    }
    
    #[test]
    fn test_parse_color_with_whitespace() {
        // 前後の空白を許容
        let color = parse_color("  #1e1e1e  ").unwrap();
        assert_eq!(color.r, 30);
        assert_eq!(color.g, 30);
        assert_eq!(color.b, 30);
        
        let color = parse_color("  rgb(30, 30, 30)  ").unwrap();
        assert_eq!(color.r, 30);
        assert_eq!(color.g, 30);
        assert_eq!(color.b, 30);
    }
}
