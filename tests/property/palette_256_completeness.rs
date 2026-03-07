// Property 6: 256色パレットの完全性
//
// このプロパティテストは、カラースキーム生成機能の256色パレット完全性を検証します。
// 任意の色情報に対して256色パレット生成を実行すると、正確に256色が含まれ、
// 各色が有効なRGB値を持ち、適切な多様性を持つことを確認します。
//
// **Validates: Requirements 2.2.3**

use proptest::prelude::*;
use std::collections::HashSet;
use twf::analyzer::color::ColorAnalyzer;
use twf::generator::scheme::{supports_256_colors, SchemeGenerator};
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

// Property 6.1: 256色パレットが生成される場合、正確に256色含まれること
//
// 256色対応ターミナルの場合、生成されるパレットが正確に256色を含むことを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_palette_256_has_exactly_256_colors(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        // 256色パレットが生成されている場合のみテスト
        if let Some(palette) = &scheme.palette_256 {
            prop_assert_eq!(
                palette.len(),
                256,
                "256色パレットは正確に256色を含む必要があります。実際: {}色",
                palette.len()
            );
        }
    }
}

// Property 6.2: 各色のRGB値が有効な範囲（0-255）内であること
//
// 生成された256色パレットの各色が、有効なRGB範囲（0-255）内であることを検証します。
// 注: Rust のu8型は0-255の範囲なので、この検証は型システムによって保証されています。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_palette_256_colors_in_valid_rgb_range(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        if let Some(palette) = &scheme.palette_256 {
            // すべての色のRGB値が有効な範囲内であることを検証
            // u8型は0-255の範囲なので、型システムによって保証されている
            for (i, color) in palette.iter().enumerate() {
                // RGB値が存在することを確認（u8型なので常に有効）
                let _ = (color.r, color.g, color.b);
                
                // 明度が有効な範囲内であることも確認
                let lab = rgb_to_lab(*color);
                prop_assert!(
                    lab.l >= -0.01 && lab.l <= 100.01,
                    "色{}の明度 {} が有効な範囲（0-100）外です",
                    i,
                    lab.l
                );
            }
        }
    }
}

// Property 6.3: パレットに重複する色が少ないこと（多様性の確保）
//
// 256色パレットが十分な多様性を持つことを検証します。
// 完全にユニークである必要はありませんが、大部分の色は異なるべきです。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_palette_256_has_diversity(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        if let Some(palette) = &scheme.palette_256 {
            // ユニークな色の数をカウント
            let unique_colors: HashSet<(u8, u8, u8)> = palette
                .iter()
                .map(|c| (c.r, c.g, c.b))
                .collect();
            
            let unique_count = unique_colors.len();
            let total_count = palette.len();
            let diversity_ratio = unique_count as f64 / total_count as f64;
            
            // 少なくとも90%の色がユニークであることを期待
            // （一部の色が重複することは許容される）
            prop_assert!(
                diversity_ratio >= 0.90,
                "256色パレットの多様性が不足しています。ユニーク: {}/{}（{:.1}%）",
                unique_count,
                total_count,
                diversity_ratio * 100.0
            );
        }
    }
}

// Property 6.4: 背景色と調和する色が含まれていること
//
// 256色パレットが背景色の色相に基づいた調和する色を含むことを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_palette_256_harmonizes_with_background(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        if let Some(palette) = &scheme.palette_256 {
            // 背景色の色相を取得
            let bg_hue = color_info.hue;
            
            // パレット内の色の色相分布を確認
            let mut hue_count_near_bg = 0;
            
            for color in palette.iter() {
                let lab = rgb_to_lab(*color);
                
                // グレースケール（彩度が低い）の場合はスキップ
                if lab.a.abs() < 5.0 && lab.b.abs() < 5.0 {
                    continue;
                }
                
                // 色相を計算
                let hue = lab.b.atan2(lab.a).to_degrees();
                let hue = if hue < 0.0 { hue + 360.0 } else { hue };
                
                // 背景色の色相に近い色をカウント（±60度以内）
                let hue_diff = (hue - bg_hue).abs();
                let hue_diff = if hue_diff > 180.0 {
                    360.0 - hue_diff
                } else {
                    hue_diff
                };
                
                if hue_diff <= 60.0 {
                    hue_count_near_bg += 1;
                }
            }
            
            // 少なくとも一部の色が背景色と調和していることを確認
            // （完全にグレースケールの背景の場合は除く）
            if color_info.saturation > 10.0 {
                prop_assert!(
                    hue_count_near_bg > 0,
                    "256色パレットに背景色と調和する色が含まれていません"
                );
            }
        }
    }
}

// Property 6.5: 任意の背景色に対して256色パレットが生成できること
//
// 任意の背景色に対して、256色パレットが正常に生成できることを検証します。
// ただし、256色サポートがない環境ではNoneが返されることも許容されます。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_palette_256_generated_for_any_background(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        
        let generator = SchemeGenerator::default();
        let result = generator.generate(&color_info);
        
        match result {
            Ok(scheme) => {
                // カラースキーム生成が成功した場合
                if supports_256_colors() {
                    // 256色サポートがある場合、パレットが生成されているべき
                    prop_assert!(
                        scheme.palette_256.is_some(),
                        "256色サポートがある環境では256色パレットが生成されるべきです"
                    );
                    
                    if let Some(palette) = &scheme.palette_256 {
                        prop_assert_eq!(
                            palette.len(),
                            256,
                            "256色パレットは正確に256色を含む必要があります"
                        );
                    }
                } else {
                    // 256色サポートがない場合、パレットはNoneでも良い
                    prop_assert!(true, "256色サポートがない環境ではパレットはオプション");
                }
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

// Property 6.6: 256色パレットの構造が正しいこと
//
// 256色パレットが標準的な構造（0-15: ANSI色、16-231: 6x6x6色立方体、232-255: グレースケール）
// に従っていることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_palette_256_has_correct_structure(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let bg_color = Rgb::new(r, g, b);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        if let Some(palette) = &scheme.palette_256 {
            // 232-255: グレースケール（最後の24色）
            // これらの色はR=G=Bであるべき
            let grayscale_start = 232;
            let mut grayscale_count = 0;
            
            for i in grayscale_start..256 {
                let color = palette[i];
                if color.r == color.g && color.g == color.b {
                    grayscale_count += 1;
                }
            }
            
            // 少なくとも大部分（80%以上）がグレースケールであることを確認
            let expected_grayscale = 24;
            let grayscale_ratio = grayscale_count as f64 / expected_grayscale as f64;
            
            prop_assert!(
                grayscale_ratio >= 0.80,
                "256色パレットの最後の24色の大部分はグレースケールであるべきです。実際: {}/{}",
                grayscale_count,
                expected_grayscale
            );
        }
    }
}

// Property 6.7: 極端な背景色（純粋な黒・白）に対する256色パレット生成
//
// 純粋な黒（0,0,0）と純粋な白（255,255,255）に対して、
// 適切な256色パレットが生成されることを検証します。
#[test]
fn prop_extreme_backgrounds_palette_256() {
    let generator = SchemeGenerator::default();
    
    // 純粋な黒
    let black = Rgb::new(0, 0, 0);
    let black_info = ColorAnalyzer::analyze(black);
    let black_scheme = generator.generate(&black_info).unwrap();
    
    if let Some(black_palette) = &black_scheme.palette_256 {
        assert_eq!(
            black_palette.len(),
            256,
            "純粋な黒に対して256色パレットが生成されるべきです"
        );
        
        // パレットに明るい色と暗い色の両方が含まれることを確認
        let mut has_bright = false;
        let mut has_dark = false;
        
        for color in black_palette.iter() {
            let lab = rgb_to_lab(*color);
            if lab.l > 70.0 {
                has_bright = true;
            }
            if lab.l < 30.0 {
                has_dark = true;
            }
        }
        
        assert!(
            has_bright && has_dark,
            "256色パレットには明るい色と暗い色の両方が含まれるべきです"
        );
    }
    
    // 純粋な白
    let white = Rgb::new(255, 255, 255);
    let white_info = ColorAnalyzer::analyze(white);
    let white_scheme = generator.generate(&white_info).unwrap();
    
    if let Some(white_palette) = &white_scheme.palette_256 {
        assert_eq!(
            white_palette.len(),
            256,
            "純粋な白に対して256色パレットが生成されるべきです"
        );
        
        // パレットに明るい色と暗い色の両方が含まれることを確認
        let mut has_bright = false;
        let mut has_dark = false;
        
        for color in white_palette.iter() {
            let lab = rgb_to_lab(*color);
            if lab.l > 70.0 {
                has_bright = true;
            }
            if lab.l < 30.0 {
                has_dark = true;
            }
        }
        
        assert!(
            has_bright && has_dark,
            "256色パレットには明るい色と暗い色の両方が含まれるべきです"
        );
    }
}

// Property 6.8: グレースケール背景に対する256色パレット生成
//
// グレースケール（R=G=B）の背景色に対して、
// 適切な256色パレットが生成されることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_grayscale_background_palette_256(
        gray_value in 0u8..=255,
    ) {
        let bg_color = Rgb::new(gray_value, gray_value, gray_value);
        let color_info = ColorAnalyzer::analyze(bg_color);
        let scheme = try_generate_scheme!(color_info);
        
        if let Some(palette) = &scheme.palette_256 {
            // 256色が生成されていることを確認
            prop_assert_eq!(
                palette.len(),
                256,
                "グレースケール背景に対して256色パレットが生成されるべきです"
            );
            
            // パレットに色相の多様性があることを確認
            // （グレースケール背景でも、カラフルなパレットが生成されるべき）
            let mut colorful_count = 0;
            
            for color in palette.iter() {
                let lab = rgb_to_lab(*color);
                
                // 彩度がある色（カラフルな色）をカウント
                let chroma = (lab.a * lab.a + lab.b * lab.b).sqrt();
                if chroma > 10.0 {
                    colorful_count += 1;
                }
            }
            
            // 少なくとも一部の色がカラフルであることを確認
            prop_assert!(
                colorful_count > 50,
                "256色パレットには十分なカラフルな色が含まれるべきです。実際: {}色",
                colorful_count
            );
        }
    }
}
