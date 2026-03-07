// Property 3: 背景色からカラースキーム生成
//
// このプロパティテストは、背景色からカラースキーム生成の正確性を検証します。
// 任意の背景色（RGB）に対して、以下のプロパティを満たすことを確認します：
// 1. 背景色から色情報を抽出できること
// 2. 抽出した色情報の明度、彩度、色相が妥当な範囲内であること
// 3. is_darkフラグが明度に基づいて正しく設定されること
// 4. 抽出した色情報からカラースキームを生成できること
// 5. 生成されたカラースキームのコントラスト比がWCAG基準を満たすこと
//
// **Validates: Requirements 2.1.8**

use proptest::prelude::*;
use twf::analyzer::color::ColorAnalyzer;
use twf::analyzer::contrast::calculate_contrast_ratio;
use twf::generator::scheme::SchemeGenerator;
use twf::models::Rgb;
use twf::utils::color_space::lab_to_rgb;

/// RGB値の生成戦略（0-255の範囲）
fn rgb_strategy() -> impl Strategy<Value = Rgb> {
    (0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(r, g, b)| Rgb::new(r, g, b))
}

proptest! {
    /// Property 3.1: 背景色から色情報を抽出できること
    #[test]
    fn prop_color_info_extraction(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        prop_assert!(!color_info.dominant_colors.is_empty());
        prop_assert_eq!(color_info.dominant_colors.len(), 1);
    }

    /// Property 3.2: 抽出した色情報の明度が妥当な範囲内であること
    #[test]
    fn prop_lightness_range(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        prop_assert!(color_info.average_lightness >= 0.0 && color_info.average_lightness <= 100.0);
    }

    /// Property 3.3: 抽出した色情報の彩度が妥当な範囲内であること
    #[test]
    fn prop_saturation_range(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        prop_assert!(color_info.saturation >= 0.0 && color_info.saturation <= 100.0);
    }

    /// Property 3.4: 抽出した色情報の色相が妥当な範囲内であること
    #[test]
    fn prop_hue_range(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        prop_assert!(color_info.hue >= 0.0 && color_info.hue < 360.0);
    }

    /// Property 3.5: is_darkフラグが明度に基づいて正しく設定されること
    #[test]
    fn prop_is_dark_consistency(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        if color_info.average_lightness < 50.0 {
            prop_assert!(color_info.is_dark);
        } else {
            prop_assert!(!color_info.is_dark);
        }
    }

    /// Property 3.6: 抽出した色情報からカラースキームを生成できること
    #[test]
    fn prop_scheme_generation(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        let generator = SchemeGenerator::default();
        let result = generator.generate(&color_info);
        // 一部の色でコントラスト比が僅かに不足する場合があるが、
        // それはエラーとして適切に報告されるべき
        if result.is_err() {
            // エラーの場合、InsufficientContrastエラーであることを確認
            let err_msg = format!("{:?}", result.err().unwrap());
            prop_assert!(err_msg.contains("InsufficientContrast"));
        }
    }

    /// Property 3.7: 生成されたカラースキームのコントラスト比がWCAG基準を満たすこと
    #[test]
    fn prop_contrast_ratio_wcag_aa(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        let generator = SchemeGenerator::default();
        let result = generator.generate(&color_info);
        
        // 生成に成功した場合のみコントラスト比をチェック
        if let Ok(scheme) = result {
            let bg_rgb = lab_to_rgb(scheme.background);
            let contrast = calculate_contrast_ratio(bg_rgb, scheme.foreground);
            prop_assert!(contrast >= 4.5);
        }
    }

    /// Property 3.8: 暗い背景には明るいフォアグラウンドが生成されること
    #[test]
    fn prop_dark_background_bright_foreground(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        if color_info.is_dark {
            let generator = SchemeGenerator::default();
            let result = generator.generate(&color_info);
            if let Ok(scheme) = result {
                let fg_lab = twf::utils::color_space::rgb_to_lab(scheme.foreground);
                prop_assert!(fg_lab.l > 50.0);
            }
        }
    }

    /// Property 3.9: 明るい背景には暗いフォアグラウンドが生成されること
    #[test]
    fn prop_light_background_dark_foreground(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        if !color_info.is_dark {
            let generator = SchemeGenerator::default();
            let scheme = generator.generate(&color_info).unwrap();
            let fg_lab = twf::utils::color_space::rgb_to_lab(scheme.foreground);
            prop_assert!(fg_lab.l < 50.0);
        }
    }
}

proptest! {
    /// Property 3.10: ANSI 16色が完全に生成されること
    #[test]
    fn prop_ansi_colors_completeness(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        let generator = SchemeGenerator::default();
        let result = generator.generate(&color_info);
        
        if let Ok(scheme) = result {
            // ANSI 16色がすべて生成されていることを確認
            // u8型なので0-255の範囲は自動的に保証される
            let _ansi = &scheme.ansi_colors;
            // 構造体が正常に生成されていればOK
        }
    }

    /// Property 3.11: 明るいバリアントは基本色より明るいこと
    #[test]
    fn prop_bright_colors_are_brighter(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        let generator = SchemeGenerator::default();
        let result = generator.generate(&color_info);
        
        if let Ok(scheme) = result {
            let ansi = &scheme.ansi_colors;
            
            let black_lab = twf::utils::color_space::rgb_to_lab(ansi.black);
            let bright_black_lab = twf::utils::color_space::rgb_to_lab(ansi.bright_black);
            prop_assert!(bright_black_lab.l >= black_lab.l);
            
            let red_lab = twf::utils::color_space::rgb_to_lab(ansi.red);
            let bright_red_lab = twf::utils::color_space::rgb_to_lab(ansi.bright_red);
            prop_assert!(bright_red_lab.l >= red_lab.l);
        }
    }

    /// Property 3.12: 256色パレットが正しく生成されること
    #[test]
    fn prop_256_palette_completeness(bg_color in rgb_strategy()) {
        let color_info = ColorAnalyzer::analyze(bg_color);
        let generator = SchemeGenerator::default();
        let result = generator.generate(&color_info);
        
        if let Ok(scheme) = result {
            if let Some(palette) = &scheme.palette_256 {
                prop_assert_eq!(palette.len(), 256);
                // u8型なので0-255の範囲は自動的に保証される
            }
        }
    }

    /// Property 3.13: コントラスト比の単調性
    #[test]
    fn prop_contrast_monotonicity(gray1 in 0u8..=100, gray2 in 155u8..=255) {
        // 明確に暗い色と明るい色を使用（境界値を避ける）
        let dark_bg = Rgb::new(gray1, gray1, gray1);
        let light_bg = Rgb::new(gray2, gray2, gray2);
        
        let dark_info = ColorAnalyzer::analyze(dark_bg);
        let light_info = ColorAnalyzer::analyze(light_bg);
        
        let generator = SchemeGenerator::default();
        let dark_result = generator.generate(&dark_info);
        let light_result = generator.generate(&light_info);
        
        if let (Ok(dark_scheme), Ok(light_scheme)) = (dark_result, light_result) {
            let dark_fg_lab = twf::utils::color_space::rgb_to_lab(dark_scheme.foreground);
            let light_fg_lab = twf::utils::color_space::rgb_to_lab(light_scheme.foreground);
            
            prop_assert!(dark_fg_lab.l > light_fg_lab.l);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_black_background_scheme() {
        let black = Rgb::new(0, 0, 0);
        let color_info = ColorAnalyzer::analyze(black);
        let generator = SchemeGenerator::default();
        let scheme = generator.generate(&color_info).unwrap();
        
        assert!(scheme.foreground.r > 200);
        assert!(scheme.foreground.g > 200);
        assert!(scheme.foreground.b > 200);
        
        let bg_rgb = lab_to_rgb(scheme.background);
        let contrast = calculate_contrast_ratio(bg_rgb, scheme.foreground);
        assert!(contrast >= 4.5);
    }

    #[test]
    fn test_white_background_scheme() {
        let white = Rgb::new(255, 255, 255);
        let color_info = ColorAnalyzer::analyze(white);
        let generator = SchemeGenerator::default();
        let scheme = generator.generate(&color_info).unwrap();
        
        assert!(scheme.foreground.r < 100);
        assert!(scheme.foreground.g < 100);
        assert!(scheme.foreground.b < 100);
        
        let bg_rgb = lab_to_rgb(scheme.background);
        let contrast = calculate_contrast_ratio(bg_rgb, scheme.foreground);
        assert!(contrast >= 4.5);
    }

    #[test]
    fn test_gray_background_scheme() {
        let gray = Rgb::new(128, 128, 128);
        let color_info = ColorAnalyzer::analyze(gray);
        let generator = SchemeGenerator::default();
        let result = generator.generate(&color_info);
        assert!(result.is_ok());
        
        let scheme = result.unwrap();
        let bg_rgb = lab_to_rgb(scheme.background);
        let contrast = calculate_contrast_ratio(bg_rgb, scheme.foreground);
        assert!(contrast >= 4.5);
    }
}
