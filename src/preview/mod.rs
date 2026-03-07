// プレビュー表示モジュール

use crate::models::{ColorScheme, FontConfig};

/// プレビューレンダラー
/// 
/// カラースキームとフォント設定のプレビューを表示します。
pub struct PreviewRenderer;

impl PreviewRenderer {
    /// プレビューを表示
    /// 
    /// カラースキームとフォント設定のプレビューをターミナルに表示します。
    /// 
    /// # Arguments
    /// 
    /// * `scheme` - カラースキーム
    /// * `font` - フォント設定
    pub fn render(scheme: &ColorScheme, font: &FontConfig) {
        println!("\n=== TWF カラースキーム プレビュー ===\n");
        
        // 1. ANSI 16色のサンプル
        Self::render_ansi_colors(scheme);
        
        // 2. フォント設定
        Self::render_font_config(font);
        
        // 3. コントラスト比情報
        Self::render_contrast_info(scheme);
        
        println!("\n=====================================\n");
    }
    
    /// ANSI 16色のサンプルを表示
    fn render_ansi_colors(scheme: &ColorScheme) {
        println!("ANSI 16色:");
        println!();
        
        // 基本8色
        println!("基本色:");
        Self::print_color_sample("Black", &scheme.ansi_colors.black);
        Self::print_color_sample("Red", &scheme.ansi_colors.red);
        Self::print_color_sample("Green", &scheme.ansi_colors.green);
        Self::print_color_sample("Yellow", &scheme.ansi_colors.yellow);
        Self::print_color_sample("Blue", &scheme.ansi_colors.blue);
        Self::print_color_sample("Magenta", &scheme.ansi_colors.magenta);
        Self::print_color_sample("Cyan", &scheme.ansi_colors.cyan);
        Self::print_color_sample("White", &scheme.ansi_colors.white);
        
        println!();
        
        // 明るい8色
        println!("明るい色:");
        Self::print_color_sample("Bright Black", &scheme.ansi_colors.bright_black);
        Self::print_color_sample("Bright Red", &scheme.ansi_colors.bright_red);
        Self::print_color_sample("Bright Green", &scheme.ansi_colors.bright_green);
        Self::print_color_sample("Bright Yellow", &scheme.ansi_colors.bright_yellow);
        Self::print_color_sample("Bright Blue", &scheme.ansi_colors.bright_blue);
        Self::print_color_sample("Bright Magenta", &scheme.ansi_colors.bright_magenta);
        Self::print_color_sample("Bright Cyan", &scheme.ansi_colors.bright_cyan);
        Self::print_color_sample("Bright White", &scheme.ansi_colors.bright_white);
        
        println!();
    }
    
    /// 色のサンプルを表示
    fn print_color_sample(name: &str, color: &crate::models::Rgb) {
        // True Colorエスケープシーケンスを使用
        print!("\x1b[38;2;{};{};{}m", color.r, color.g, color.b);
        print!("■ ");
        print!("\x1b[0m");
        println!("{:<15} RGB({:3}, {:3}, {:3})", name, color.r, color.g, color.b);
    }
    
    /// フォント設定を表示
    fn render_font_config(font: &FontConfig) {
        println!("フォント設定:");
        println!("  推奨ウェイト: {:?}", font.weight);
        
        if !font.recommended_fonts.is_empty() {
            println!("  推奨フォント:");
            for (i, font_name) in font.recommended_fonts.iter().take(5).enumerate() {
                println!("    {}. {}", i + 1, font_name);
            }
        }
        
        println!();
    }
    
    /// コントラスト比情報を表示
    fn render_contrast_info(scheme: &ColorScheme) {
        use crate::analyzer::contrast::calculate_contrast_ratio;
        use crate::utils::color_space::lab_to_rgb;
        
        let bg_rgb = lab_to_rgb(scheme.background);
        let contrast = calculate_contrast_ratio(bg_rgb, scheme.foreground);
        
        println!("コントラスト比:");
        println!("  背景色とフォアグラウンド: {:.2}:1", contrast);
        
        if contrast >= 7.0 {
            println!("  評価: AAA (優秀)");
        } else if contrast >= 4.5 {
            println!("  評価: AA (良好)");
        } else {
            println!("  評価: 基準未満");
        }
        
        println!();
        
        // True Colorサポート情報
        if scheme.supports_true_color {
            println!("True Color (24bit): サポート");
        } else if scheme.palette_256.is_some() {
            println!("256色: サポート");
        } else {
            println!("16色: サポート");
        }
    }
}
