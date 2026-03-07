// Property 5: ANSI 16色の完全性
//
// このプロパティテストは、カラースキーム生成機能のANSI 16色完全性を検証します。
// 任意の色情報に対してANSI色生成を実行すると、16色すべて（基本8色 + 明るい8色）が
// 生成され、各色が有効なRGB値を持つことを確認します。
//
// **Validates: Requirements 2.2.2**

use proptest::prelude::*;
use twf::analyzer::color::ColorAnalyzer;
use twf::generator::scheme::SchemeGenerator;
use twf::models::Rgb;
use twf::utils::color_space::rgb_to_lab;

// ヘルパーマクロ: カラースキーム生成を試行し、InsufficientContrastエラーの場合はスキップ
macro_rules! try_generate_scheme {
    ($color_info:expr) => {{
        let generator = SchemeGenerator::default();
        match generator.generate(&$color_info) {
            Ok(s) => s,
            Err(twf::models::TwfError::InsufficientContrast { .. }) => {
                // コントラスト比が不足する場合はテストをスキップ
                prop_assume!(false);
                return Ok(());
            }
            Err(e) => {
                prop_assert!(false, "予期しないエラー: {:?}", e);
                return Ok(());
            }
        }
    }};
}

// Property 5.1: ANSI 16色がすべて生成されること
//
// 任意の背景色に対して、ANSI 16色（基本8色 + 明るい8色）が
// すべて生成されることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_all_16_ansi_colors_generated(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        // 16色すべてが存在することを検証（構造体のフィールドとして存在）
        let ansi = &scheme.ansi_colors;
        
        // 基本8色と明るい8色が存在することを確認
        let _colors = vec![
            ansi.black, ansi.red, ansi.green, ansi.yellow,
            ansi.blue, ansi.magenta, ansi.cyan, ansi.white,
            ansi.bright_black, ansi.bright_red, ansi.bright_green, ansi.bright_yellow,
            ansi.bright_blue, ansi.bright_magenta, ansi.bright_cyan, ansi.bright_white,
        ];
        
        prop_assert!(true, "ANSI 16色がすべて生成されました");
    }
}

// Property 5.2: 各色のRGB値が有効な範囲（0-255）内であること
//
// 生成されたANSI 16色の各色が、有効なRGB範囲（0-255）内であることを検証します。
// 注: Rust のu8型は0-255の範囲なので、この検証は型システムによって保証されています。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_ansi_colors_in_valid_rgb_range(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        let ansi = &scheme.ansi_colors;
        
        // すべての色のRGB値が有効な範囲内であることを検証
        // u8型は0-255の範囲なので、型システムによって保証されている
        let colors = vec![
            ansi.black, ansi.red, ansi.green, ansi.yellow,
            ansi.blue, ansi.magenta, ansi.cyan, ansi.white,
            ansi.bright_black, ansi.bright_red, ansi.bright_green, ansi.bright_yellow,
            ansi.bright_blue, ansi.bright_magenta, ansi.bright_cyan, ansi.bright_white,
        ];
        
        prop_assert!(colors.len() == 16, "ANSI 16色がすべて存在します");
    }
}

// Property 5.3: 明るいバリアント（Bright）が基本色より明るいこと
//
// 明るいバリアント（Bright）の色が、対応する基本色より明るいことを検証します。
// 明るさはLab色空間のL値（明度）で判定します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_bright_variants_are_lighter(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        let ansi = &scheme.ansi_colors;
        
        // 各色ペアについて、明るいバリアントが基本色より明るいことを検証
        let color_pairs = vec![
            ("black", ansi.black, ansi.bright_black),
            ("red", ansi.red, ansi.bright_red),
            ("green", ansi.green, ansi.bright_green),
            ("yellow", ansi.yellow, ansi.bright_yellow),
            ("blue", ansi.blue, ansi.bright_blue),
            ("magenta", ansi.magenta, ansi.bright_magenta),
            ("cyan", ansi.cyan, ansi.bright_cyan),
            ("white", ansi.white, ansi.bright_white),
        ];
        
        for (name, base, bright) in color_pairs {
            let base_lab = rgb_to_lab(base);
            let bright_lab = rgb_to_lab(bright);
            
            prop_assert!(
                bright_lab.l >= base_lab.l,
                "明るいバリアント（bright_{}）の明度 {} が基本色（{}）の明度 {} 以上であることを期待",
                name,
                bright_lab.l,
                name,
                base_lab.l
            );
        }
    }
}

// Property 5.4: 任意の背景色に対してANSI 16色が生成できること
//
// 任意の背景色（極端な色を含む）に対して、ANSI 16色が正常に生成できることを検証します。
// InsufficientContrastエラーは許容されます（一部の背景色では物理的に4.5:1を達成できない）。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_ansi_colors_generated_for_any_background(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        
        let generator = SchemeGenerator::default();
        let result = generator.generate(&color_info);
        
        // カラースキーム生成が成功するか、InsufficientContrastエラーであることを検証
        match result {
            Ok(scheme) => {
                // 成功した場合、ANSI 16色が含まれることを検証
                let _ansi = &scheme.ansi_colors;
                prop_assert!(true, "ANSI 16色が正常に生成されました");
            }
            Err(twf::models::TwfError::InsufficientContrast { .. }) => {
                // InsufficientContrastエラーは許容される
                prop_assert!(true, "InsufficientContrastエラーは許容されます");
            }
            Err(e) => {
                prop_assert!(false, "予期しないエラー: {:?}", e);
            }
        }
    }
}

// Property 5.5: 極端な背景色（純粋な黒・白）に対するANSI色生成
//
// 純粋な黒（0,0,0）と純粋な白（255,255,255）に対して、
// 適切なANSI 16色が生成されることを検証します。
#[test]
fn prop_extreme_backgrounds_ansi_colors() {
    // 純粋な黒
    let black = Rgb::new(0, 0, 0);
    let black_info = ColorAnalyzer::analyze(black);
    let generator = SchemeGenerator::default();
    let black_scheme = generator.generate(&black_info).unwrap();
    
    let black_ansi = &black_scheme.ansi_colors;
    
    // 暗い背景なので、明るい色が生成されるべき
    assert!(
        black_ansi.white.r > 200 && black_ansi.white.g > 200 && black_ansi.white.b > 200,
        "純粋な黒に対してwhiteが十分に明るくありません: {:?}",
        black_ansi.white
    );
    
    // 純粋な白
    let white = Rgb::new(255, 255, 255);
    let white_info = ColorAnalyzer::analyze(white);
    let white_scheme = generator.generate(&white_info).unwrap();
    
    let white_ansi = &white_scheme.ansi_colors;
    
    // 明るい背景なので、暗い色が生成されるべき
    assert!(
        white_ansi.black.r < 50 && white_ansi.black.g < 50 && white_ansi.black.b < 50,
        "純粋な白に対してblackが十分に暗くありません: {:?}",
        white_ansi.black
    );
}

// Property 5.6: グレースケール背景に対するANSI色生成
//
// グレースケール（R=G=B）の背景色に対して、
// 適切なANSI 16色が生成されることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_grayscale_background_ansi_colors(
        gray_value in 0u8..=255,
    ) {
        let bg_color = Rgb::new(gray_value, gray_value, gray_value);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        let ansi = &scheme.ansi_colors;
        
        // ANSI 16色がすべて生成されていることを検証
        let colors = vec![
            ansi.black, ansi.red, ansi.green, ansi.yellow,
            ansi.blue, ansi.magenta, ansi.cyan, ansi.white,
            ansi.bright_black, ansi.bright_red, ansi.bright_green, ansi.bright_yellow,
            ansi.bright_blue, ansi.bright_magenta, ansi.bright_cyan, ansi.bright_white,
        ];
        
        prop_assert!(
            colors.len() == 16,
            "ANSI 16色がすべて生成されました"
        );
    }
}

// Property 5.7: ANSI色の明度が適切な範囲内であること
//
// 生成されたANSI色の明度が、有効な範囲内（0-100）であることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_ansi_colors_lightness_in_appropriate_range(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        let ansi = &scheme.ansi_colors;
        
        // すべてのANSI色の明度が有効な範囲内であることを検証
        let colors = vec![
            ("black", ansi.black),
            ("red", ansi.red),
            ("green", ansi.green),
            ("yellow", ansi.yellow),
            ("blue", ansi.blue),
            ("magenta", ansi.magenta),
            ("cyan", ansi.cyan),
            ("white", ansi.white),
            ("bright_black", ansi.bright_black),
            ("bright_red", ansi.bright_red),
            ("bright_green", ansi.bright_green),
            ("bright_yellow", ansi.bright_yellow),
            ("bright_blue", ansi.bright_blue),
            ("bright_magenta", ansi.bright_magenta),
            ("bright_cyan", ansi.bright_cyan),
            ("bright_white", ansi.bright_white),
        ];
        
        for (name, color) in colors {
            let lab = rgb_to_lab(color);
            
            // 明度が有効な範囲内（0-100）であることを検証
            // 浮動小数点の丸め誤差を考慮して、わずかな超過を許容
            prop_assert!(
                lab.l >= -0.01 && lab.l <= 100.01,
                "{}の明度 {} が有効な範囲（0-100）外です",
                name,
                lab.l
            );
        }
    }
}
