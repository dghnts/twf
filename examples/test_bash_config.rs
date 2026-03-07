// Bash設定生成のテスト例

use twf::applier::shell::generate_bash_config;
use twf::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};

fn main() {
    // テスト用のカラースキームを作成
    let ansi_colors = AnsiColors {
        black: Rgb::new(0, 0, 0),
        red: Rgb::new(255, 0, 0),
        green: Rgb::new(0, 255, 0),
        yellow: Rgb::new(255, 255, 0),
        blue: Rgb::new(0, 0, 255),
        magenta: Rgb::new(255, 0, 255),
        cyan: Rgb::new(0, 255, 255),
        white: Rgb::new(255, 255, 255),
        bright_black: Rgb::new(128, 128, 128),
        bright_red: Rgb::new(255, 128, 128),
        bright_green: Rgb::new(128, 255, 128),
        bright_yellow: Rgb::new(255, 255, 128),
        bright_blue: Rgb::new(128, 128, 255),
        bright_magenta: Rgb::new(255, 128, 255),
        bright_cyan: Rgb::new(128, 255, 255),
        bright_white: Rgb::new(255, 255, 255),
    };
    
    let scheme = ColorScheme {
        foreground: Rgb::new(240, 240, 240),
        background: Lab { l: 20.0, a: 0.0, b: 0.0 },
        ansi_colors,
        palette_256: None,
        supports_true_color: true,
    };
    
    let font = FontConfig {
        weight: FontWeight::Normal,
        recommended_fonts: vec!["Fira Code".to_string(), "JetBrains Mono".to_string()],
    };
    
    // 設定を生成
    let config = generate_bash_config(&scheme, &font);
    
    println!("=== 生成されたBash/Zsh設定 ===\n");
    println!("{}", config);
}
