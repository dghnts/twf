// Property 7: フォントウェイト選択の一貫性
//
// このプロパティテストは、フォント設定生成機能の正確性を検証します。
// 任意の色情報（明度と彩度を含む）に対して、背景の特性に基づいて
// 適切なフォントウェイトが選択されることを確認します。
//
// **Validates: Requirements 2.3.1, 2.3.2**

use proptest::prelude::*;
use twf::generator::font::FontOptimizer;
use twf::models::{ColorInfo, FontWeight, Lab};

/// 色情報を生成するストラテジー
fn color_info_strategy() -> impl Strategy<Value = ColorInfo> {
    (
        // average_lightness: 0.0 - 100.0
        0.0f64..=100.0,
        // saturation: 0.0 - 100.0
        0.0f64..=100.0,
        // hue: 0.0 - 360.0
        0.0f64..=360.0,
        // Lab色のL値: 0.0 - 100.0
        0.0f64..=100.0,
        // Lab色のa値: -128.0 - 127.0
        -128.0f64..=127.0,
        // Lab色のb値: -128.0 - 127.0
        -128.0f64..=127.0,
    )
        .prop_map(|(lightness, saturation, hue, l, a, b)| {
            let is_dark = lightness < 50.0;
            ColorInfo {
                dominant_colors: vec![Lab { l, a, b }],
                average_lightness: lightness,
                saturation,
                hue,
                is_dark,
            }
        })
}

proptest! {
    /// Property 7.1: 暗い背景では通常の太さ（Normal）が選択されること
    ///
    /// 暗い背景（is_dark == true）で彩度が50.0以下の場合、
    /// フォントウェイトがNormalになることを検証します。
    #[test]
    fn prop_dark_background_normal_weight(
        lightness in 0.0f64..50.0,
        saturation in 0.0f64..=50.0,
        hue in 0.0f64..=360.0,
    ) {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: lightness, a: 0.0, b: 0.0 }],
            average_lightness: lightness,
            saturation,
            hue,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        prop_assert_eq!(
            font_config.weight,
            FontWeight::Normal,
            "暗い背景（明度: {:.2}, 彩度: {:.2}）では通常の太さ（Normal）が選択されるべきです",
            lightness,
            saturation
        );
    }

    /// Property 7.2: 明るい背景では少し太め（Medium）が選択されること
    ///
    /// 明るい背景（is_dark == false）で彩度が50.0以下の場合、
    /// フォントウェイトがMediumになることを検証します。
    #[test]
    fn prop_light_background_medium_weight(
        lightness in 50.0f64..=100.0,
        saturation in 0.0f64..=50.0,
        hue in 0.0f64..=360.0,
    ) {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: lightness, a: 0.0, b: 0.0 }],
            average_lightness: lightness,
            saturation,
            hue,
            is_dark: false,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        prop_assert_eq!(
            font_config.weight,
            FontWeight::Medium,
            "明るい背景（明度: {:.2}, 彩度: {:.2}）では少し太め（Medium）が選択されるべきです",
            lightness,
            saturation
        );
    }

    /// Property 7.3: 彩度が高い（> 50.0）場合、フォントウェイトが増加すること
    ///
    /// 彩度が50.0を超える場合、フォントウェイトが増加することを検証します。
    /// - 暗い背景: Normal → Medium
    /// - 明るい背景: Medium → Bold
    #[test]
    fn prop_high_saturation_increases_weight(
        lightness in 0.0f64..=100.0,
        saturation in 50.01f64..=100.0,
        hue in 0.0f64..=360.0,
    ) {
        let is_dark = lightness < 50.0;
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: lightness, a: 0.0, b: 0.0 }],
            average_lightness: lightness,
            saturation,
            hue,
            is_dark,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        let expected_weight = if is_dark {
            FontWeight::Medium
        } else {
            FontWeight::Bold
        };
        
        prop_assert_eq!(
            font_config.weight,
            expected_weight,
            "高彩度（{:.2}）の背景（明度: {:.2}, is_dark: {}）ではフォントウェイトが増加するべきです",
            saturation,
            lightness,
            is_dark
        );
    }

    /// Property 7.4: フォントウェイトが有効な値であること
    ///
    /// 任意の色情報に対して、生成されるフォントウェイトが
    /// 有効な値（Light, Normal, Medium, Bold）のいずれかであることを検証します。
    #[test]
    fn prop_valid_font_weight(color_info in color_info_strategy()) {
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // フォントウェイトが有効な値であることを確認
        let valid_weights = vec![
            FontWeight::Light,
            FontWeight::Normal,
            FontWeight::Medium,
            FontWeight::Bold,
        ];
        
        prop_assert!(
            valid_weights.contains(&font_config.weight),
            "フォントウェイト {:?} は有効な値ではありません",
            font_config.weight
        );
    }

    /// Property 7.5: 同じ色情報に対して常に同じフォントウェイトが選択されること（一貫性）
    ///
    /// 同じ色情報に対して複数回フォント設定を生成した場合、
    /// 常に同じフォントウェイトが選択されることを検証します。
    #[test]
    fn prop_consistent_font_weight(color_info in color_info_strategy()) {
        let optimizer = FontOptimizer;
        
        // 同じ色情報で複数回生成
        let font_config1 = optimizer.optimize(&color_info);
        let font_config2 = optimizer.optimize(&color_info);
        let font_config3 = optimizer.optimize(&color_info);
        
        prop_assert_eq!(
            font_config1.weight,
            font_config2.weight,
            "同じ色情報に対して異なるフォントウェイトが選択されました"
        );
        
        prop_assert_eq!(
            font_config2.weight,
            font_config3.weight,
            "同じ色情報に対して異なるフォントウェイトが選択されました"
        );
    }

    /// Property 7.6: 任意の色情報に対してフォント設定が生成できること
    ///
    /// 任意の色情報に対して、フォント設定生成がパニックせずに
    /// 完了することを検証します。
    #[test]
    fn prop_font_config_generation_does_not_panic(color_info in color_info_strategy()) {
        let optimizer = FontOptimizer;
        
        // パニックせずに完了することを確認
        let font_config = optimizer.optimize(&color_info);
        
        // フォント設定が生成されたことを確認
        prop_assert!(
            !font_config.recommended_fonts.is_empty(),
            "推奨フォントリストが空です"
        );
    }

    /// Property 7.7: 境界値テスト - 彩度が正確に50.0の場合
    ///
    /// 彩度が正確に50.0の場合、フォントウェイトが増加しないことを検証します。
    #[test]
    fn prop_saturation_boundary_50(
        lightness in 0.0f64..=100.0,
        hue in 0.0f64..=360.0,
    ) {
        let is_dark = lightness < 50.0;
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: lightness, a: 0.0, b: 0.0 }],
            average_lightness: lightness,
            saturation: 50.0,  // 境界値
            hue,
            is_dark,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        let expected_weight = if is_dark {
            FontWeight::Normal
        } else {
            FontWeight::Medium
        };
        
        prop_assert_eq!(
            font_config.weight,
            expected_weight,
            "彩度が50.0の場合、フォントウェイトは増加しないべきです（明度: {:.2}, is_dark: {}）",
            lightness,
            is_dark
        );
    }

    /// Property 7.8: 境界値テスト - 明度が正確に50.0の場合
    ///
    /// 明度が正確に50.0の場合、is_darkがfalseになり、
    /// Mediumが選択されることを検証します。
    #[test]
    fn prop_lightness_boundary_50(
        saturation in 0.0f64..=50.0,
        hue in 0.0f64..=360.0,
    ) {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 50.0, a: 0.0, b: 0.0 }],
            average_lightness: 50.0,  // 境界値
            saturation,
            hue,
            is_dark: false,  // 50.0は明るい背景として扱われる
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        prop_assert_eq!(
            font_config.weight,
            FontWeight::Medium,
            "明度が50.0の場合、Mediumが選択されるべきです（彩度: {:.2}）",
            saturation
        );
    }

    /// Property 7.9: フォントウェイトの単調性
    ///
    /// 彩度が増加すると、フォントウェイトが増加するか同じままであることを検証します。
    /// （減少することはない）
    #[test]
    fn prop_font_weight_monotonicity(
        lightness in 0.0f64..=100.0,
        saturation1 in 0.0f64..=50.0,
        saturation2 in 50.01f64..=100.0,
        hue in 0.0f64..=360.0,
    ) {
        let is_dark = lightness < 50.0;
        
        // 低彩度の色情報
        let color_info1 = ColorInfo {
            dominant_colors: vec![Lab { l: lightness, a: 0.0, b: 0.0 }],
            average_lightness: lightness,
            saturation: saturation1,
            hue,
            is_dark,
        };
        
        // 高彩度の色情報
        let color_info2 = ColorInfo {
            dominant_colors: vec![Lab { l: lightness, a: 0.0, b: 0.0 }],
            average_lightness: lightness,
            saturation: saturation2,
            hue,
            is_dark,
        };
        
        let optimizer = FontOptimizer;
        let font_config1 = optimizer.optimize(&color_info1);
        let font_config2 = optimizer.optimize(&color_info2);
        
        // フォントウェイトを数値に変換
        let weight_to_num = |w: FontWeight| match w {
            FontWeight::Light => 1,
            FontWeight::Normal => 2,
            FontWeight::Medium => 3,
            FontWeight::Bold => 4,
        };
        
        let weight1_num = weight_to_num(font_config1.weight);
        let weight2_num = weight_to_num(font_config2.weight);
        
        prop_assert!(
            weight2_num >= weight1_num,
            "彩度が増加すると、フォントウェイトは増加するか同じままであるべきです（彩度1: {:.2} -> 彩度2: {:.2}, ウェイト1: {:?} -> ウェイト2: {:?}）",
            saturation1,
            saturation2,
            font_config1.weight,
            font_config2.weight
        );
    }

    /// Property 7.10: 推奨フォントリストが空でないこと
    ///
    /// 任意の色情報に対して、推奨フォントリストが
    /// 少なくとも1つのフォントを含むことを検証します。
    #[test]
    fn prop_recommended_fonts_not_empty(color_info in color_info_strategy()) {
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        prop_assert!(
            !font_config.recommended_fonts.is_empty(),
            "推奨フォントリストが空です"
        );
        
        // 各フォント名が空でないことを確認
        for font in &font_config.recommended_fonts {
            prop_assert!(
                !font.is_empty(),
                "推奨フォントリストに空の文字列が含まれています"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// 具体的な例: 暗い背景、低彩度
    #[test]
    fn test_dark_low_saturation() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 20.0, a: 0.0, b: 0.0 }],
            average_lightness: 20.0,
            saturation: 30.0,
            hue: 0.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        assert_eq!(font_config.weight, FontWeight::Normal);
    }

    /// 具体的な例: 明るい背景、低彩度
    #[test]
    fn test_light_low_saturation() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 80.0, a: 0.0, b: 0.0 }],
            average_lightness: 80.0,
            saturation: 30.0,
            hue: 0.0,
            is_dark: false,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        assert_eq!(font_config.weight, FontWeight::Medium);
    }

    /// 具体的な例: 暗い背景、高彩度
    #[test]
    fn test_dark_high_saturation() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 20.0, a: 50.0, b: 50.0 }],
            average_lightness: 20.0,
            saturation: 70.0,
            hue: 45.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        assert_eq!(font_config.weight, FontWeight::Medium);
    }

    /// 具体的な例: 明るい背景、高彩度
    #[test]
    fn test_light_high_saturation() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 80.0, a: 50.0, b: 50.0 }],
            average_lightness: 80.0,
            saturation: 70.0,
            hue: 45.0,
            is_dark: false,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        assert_eq!(font_config.weight, FontWeight::Bold);
    }

    /// エッジケース: 彩度が正確に50.0
    #[test]
    fn test_saturation_exactly_50() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 20.0, a: 0.0, b: 0.0 }],
            average_lightness: 20.0,
            saturation: 50.0,
            hue: 0.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 50.0は増加しない（> 50.0のみ増加）
        assert_eq!(font_config.weight, FontWeight::Normal);
    }

    /// エッジケース: 彩度が50.1
    #[test]
    fn test_saturation_just_above_50() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 20.0, a: 0.0, b: 0.0 }],
            average_lightness: 20.0,
            saturation: 50.1,
            hue: 0.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 50.0を超えるので増加
        assert_eq!(font_config.weight, FontWeight::Medium);
    }

    /// エッジケース: 明度が正確に50.0
    #[test]
    fn test_lightness_exactly_50() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 50.0, a: 0.0, b: 0.0 }],
            average_lightness: 50.0,
            saturation: 30.0,
            hue: 0.0,
            is_dark: false,  // 50.0は明るい背景として扱われる
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        assert_eq!(font_config.weight, FontWeight::Medium);
    }

    /// エッジケース: 最小値（明度0.0、彩度0.0）
    #[test]
    fn test_minimum_values() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 0.0, a: 0.0, b: 0.0 }],
            average_lightness: 0.0,
            saturation: 0.0,
            hue: 0.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        assert_eq!(font_config.weight, FontWeight::Normal);
    }

    /// エッジケース: 最大値（明度100.0、彩度100.0）
    #[test]
    fn test_maximum_values() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 100.0, a: 0.0, b: 0.0 }],
            average_lightness: 100.0,
            saturation: 100.0,
            hue: 360.0,
            is_dark: false,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        assert_eq!(font_config.weight, FontWeight::Bold);
    }
}
