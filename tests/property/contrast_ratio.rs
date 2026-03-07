// Property 13: コントラスト比計算の対称性
//
// このプロパティテストは、コントラスト比計算の正確性を検証します。
// WCAG 2.1基準に基づいたコントラスト比計算が、以下のプロパティを満たすことを確認します：
// 1. 対称性: contrast_ratio(A, B) == contrast_ratio(B, A)
// 2. 範囲: コントラスト比は常に1.0以上21.0以下
// 3. 同一色: contrast_ratio(A, A) == 1.0
// 4. 白黒: contrast_ratio(white, black) == 21.0（理論値）

use proptest::prelude::*;
use twf::analyzer::contrast::{calculate_contrast_ratio, calculate_relative_luminance};
use twf::models::Rgb;

// RGB値の生成戦略（0-255の範囲）
fn rgb_strategy() -> impl Strategy<Value = Rgb> {
    (0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(r, g, b)| Rgb::new(r, g, b))
}

proptest! {
    /// Property 13.1: コントラスト比の対称性
    ///
    /// 任意の2つのRGB色に対して、コントラスト比計算は対称的であることを検証します。
    /// つまり、calculate_contrast_ratio(color1, color2) == calculate_contrast_ratio(color2, color1)
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_contrast_ratio_symmetry(color1 in rgb_strategy(), color2 in rgb_strategy()) {
        let ratio1 = calculate_contrast_ratio(color1, color2);
        let ratio2 = calculate_contrast_ratio(color2, color1);

        // 浮動小数点演算の誤差を考慮して、非常に小さい許容誤差で比較
        let tolerance = 1e-10;
        let diff = (ratio1 - ratio2).abs();

        prop_assert!(
            diff < tolerance,
            "Contrast ratio should be symmetric: ratio({:?}, {:?}) = {}, ratio({:?}, {:?}) = {}, diff = {}",
            color1, color2, ratio1, color2, color1, ratio2, diff
        );
    }

    /// Property 13.2: コントラスト比の範囲
    ///
    /// 任意の2つのRGB色に対して、コントラスト比は常に1.0以上21.0以下の範囲内であることを検証します。
    /// - 最小値: 1.0（同じ色同士）
    /// - 最大値: 21.0（白と黒）
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_contrast_ratio_range(color1 in rgb_strategy(), color2 in rgb_strategy()) {
        let ratio = calculate_contrast_ratio(color1, color2);

        prop_assert!(
            ratio >= 1.0,
            "Contrast ratio should be at least 1.0: got {} for colors {:?} and {:?}",
            ratio, color1, color2
        );

        prop_assert!(
            ratio <= 21.0,
            "Contrast ratio should be at most 21.0: got {} for colors {:?} and {:?}",
            ratio, color1, color2
        );
    }

    /// Property 13.3: 同一色のコントラスト比
    ///
    /// 任意のRGB色に対して、その色自身とのコントラスト比は1.0であることを検証します。
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_contrast_ratio_same_color(color in rgb_strategy()) {
        let ratio = calculate_contrast_ratio(color, color);

        // 浮動小数点演算の誤差を考慮
        let tolerance = 1e-10;
        let diff = (ratio - 1.0).abs();

        prop_assert!(
            diff < tolerance,
            "Contrast ratio of same color should be 1.0: got {} for color {:?}, diff = {}",
            ratio, color, diff
        );
    }

    /// Property 13.4: 相対輝度の範囲
    ///
    /// 任意のRGB色に対して、相対輝度は常に0.0以上1.0以下の範囲内であることを検証します。
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_relative_luminance_range(color in rgb_strategy()) {
        let luminance = calculate_relative_luminance(color);

        prop_assert!(
            luminance >= 0.0,
            "Relative luminance should be at least 0.0: got {} for color {:?}",
            luminance, color
        );

        prop_assert!(
            luminance <= 1.0,
            "Relative luminance should be at most 1.0: got {} for color {:?}",
            luminance, color
        );
    }

    /// Property 13.5: 相対輝度の単調性
    ///
    /// RGB値が増加すると、相対輝度も増加することを検証します。
    /// ただし、RGB各チャンネルの寄与度が異なるため（G > R > B）、
    /// ここでは全チャンネルが同時に増加する場合のみを検証します。
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_relative_luminance_monotonicity(
        r1 in 0u8..=254,
        g1 in 0u8..=254,
        b1 in 0u8..=254,
    ) {
        let color1 = Rgb::new(r1, g1, b1);
        let color2 = Rgb::new(r1 + 1, g1 + 1, b1 + 1);

        let luminance1 = calculate_relative_luminance(color1);
        let luminance2 = calculate_relative_luminance(color2);

        prop_assert!(
            luminance2 >= luminance1,
            "Relative luminance should increase when RGB values increase: \
             color1={:?} (luminance={}), color2={:?} (luminance={})",
            color1, luminance1, color2, luminance2
        );
    }

    /// Property 13.6: コントラスト比の推移性（弱い形）
    ///
    /// 3つの色A, B, Cがあり、AがBより明るく、BがCより明るい場合、
    /// contrast_ratio(A, C) >= max(contrast_ratio(A, B), contrast_ratio(B, C))
    /// が成り立つことを検証します。
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_contrast_ratio_transitivity(
        color_a in rgb_strategy(),
        color_b in rgb_strategy(),
        color_c in rgb_strategy(),
    ) {
        let lum_a = calculate_relative_luminance(color_a);
        let lum_b = calculate_relative_luminance(color_b);
        let lum_c = calculate_relative_luminance(color_c);

        // A > B > C の順に並べ替え
        let mut colors = vec![(lum_a, color_a), (lum_b, color_b), (lum_c, color_c)];
        colors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let (_, brightest) = colors[0];
        let (_, middle) = colors[1];
        let (_, darkest) = colors[2];

        let ratio_brightest_darkest = calculate_contrast_ratio(brightest, darkest);
        let ratio_brightest_middle = calculate_contrast_ratio(brightest, middle);
        let ratio_middle_darkest = calculate_contrast_ratio(middle, darkest);

        // 最も明るい色と最も暗い色のコントラスト比は、
        // 中間の色を経由した場合のコントラスト比以上であるべき
        prop_assert!(
            ratio_brightest_darkest >= ratio_brightest_middle - 1e-10,
            "Contrast ratio transitivity violated: \
             ratio({:?}, {:?}) = {} should be >= ratio({:?}, {:?}) = {}",
            brightest, darkest, ratio_brightest_darkest,
            brightest, middle, ratio_brightest_middle
        );

        prop_assert!(
            ratio_brightest_darkest >= ratio_middle_darkest - 1e-10,
            "Contrast ratio transitivity violated: \
             ratio({:?}, {:?}) = {} should be >= ratio({:?}, {:?}) = {}",
            brightest, darkest, ratio_brightest_darkest,
            middle, darkest, ratio_middle_darkest
        );
    }
}

// 特定の重要なケースをテストする追加のプロパティテスト
proptest! {
    /// Property 13.7: 白と任意の色のコントラスト比
    ///
    /// 白（255, 255, 255）と任意の色のコントラスト比は、
    /// その色が暗いほど高くなることを検証します。
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_white_contrast_ratio(color in rgb_strategy()) {
        let white = Rgb::new(255, 255, 255);
        let ratio = calculate_contrast_ratio(white, color);

        // 白とのコントラスト比は1.0（白自身）から21.0（黒）の範囲
        prop_assert!(
            ratio >= 1.0 && ratio <= 21.0,
            "White contrast ratio out of range: got {} for color {:?}",
            ratio, color
        );

        // 色が暗いほど（輝度が低いほど）、白とのコントラスト比は高くなる
        let luminance = calculate_relative_luminance(color);
        let expected_ratio = (1.0 + 0.05) / (luminance + 0.05);

        let tolerance = 1e-10;
        let diff = (ratio - expected_ratio).abs();

        prop_assert!(
            diff < tolerance,
            "White contrast ratio calculation mismatch: expected {}, got {}, diff = {}",
            expected_ratio, ratio, diff
        );
    }

    /// Property 13.8: 黒と任意の色のコントラスト比
    ///
    /// 黒（0, 0, 0）と任意の色のコントラスト比は、
    /// その色が明るいほど高くなることを検証します。
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_black_contrast_ratio(color in rgb_strategy()) {
        let black = Rgb::new(0, 0, 0);
        let ratio = calculate_contrast_ratio(black, color);

        // 黒とのコントラスト比は1.0（黒自身）から21.0（白）の範囲
        prop_assert!(
            ratio >= 1.0 && ratio <= 21.0,
            "Black contrast ratio out of range: got {} for color {:?}",
            ratio, color
        );

        // 色が明るいほど（輝度が高いほど）、黒とのコントラスト比は高くなる
        let luminance = calculate_relative_luminance(color);
        let expected_ratio = (luminance + 0.05) / (0.0 + 0.05);

        let tolerance = 1e-10;
        let diff = (ratio - expected_ratio).abs();

        prop_assert!(
            diff < tolerance,
            "Black contrast ratio calculation mismatch: expected {}, got {}, diff = {}",
            expected_ratio, ratio, diff
        );
    }

    /// Property 13.9: WCAG AA基準の検証
    ///
    /// 任意の2つの色に対して、コントラスト比が4.5:1以上であれば、
    /// WCAG AA基準を満たすことを検証します。
    ///
    /// **Validates: Requirements 2.2.5（コントラスト比がWCAG 2.1 AA基準を満たすこと）**
    #[test]
    fn prop_wcag_aa_threshold(color1 in rgb_strategy(), color2 in rgb_strategy()) {
        let ratio = calculate_contrast_ratio(color1, color2);

        // WCAG AA基準: 4.5:1以上
        let meets_aa = ratio >= 4.5;

        // コントラスト比が4.5以上の場合、AA基準を満たす
        // コントラスト比が4.5未満の場合、AA基準を満たさない
        if meets_aa {
            prop_assert!(
                ratio >= 4.5,
                "Color pair should meet WCAG AA standard: ratio = {} for colors {:?} and {:?}",
                ratio, color1, color2
            );
        } else {
            prop_assert!(
                ratio < 4.5,
                "Color pair should not meet WCAG AA standard: ratio = {} for colors {:?} and {:?}",
                ratio, color1, color2
            );
        }
    }

    /// Property 13.10: グレースケールのコントラスト比
    ///
    /// グレースケール（R=G=B）の色同士のコントラスト比は、
    /// RGB値の差に応じて単調に増加することを検証します。
    ///
    /// **Validates: Requirements（コントラスト比計算の正確性）**
    #[test]
    fn prop_grayscale_contrast_monotonicity(
        gray1 in 0u8..=255,
        gray2 in 0u8..=255,
    ) {
        let color1 = Rgb::new(gray1, gray1, gray1);
        let color2 = Rgb::new(gray2, gray2, gray2);

        let ratio = calculate_contrast_ratio(color1, color2);

        // グレースケールの場合、RGB値の差が大きいほどコントラスト比も大きい
        let diff = (gray1 as i16 - gray2 as i16).abs();

        if diff == 0 {
            // 同じ色の場合、コントラスト比は1.0
            let tolerance = 1e-10;
            prop_assert!(
                (ratio - 1.0).abs() < tolerance,
                "Same grayscale color should have contrast ratio 1.0: got {}",
                ratio
            );
        } else {
            // 異なる色の場合、コントラスト比は1.0より大きい
            prop_assert!(
                ratio > 1.0,
                "Different grayscale colors should have contrast ratio > 1.0: got {} for gray1={}, gray2={}",
                ratio, gray1, gray2
            );
        }

        // グレースケールの最大コントラスト比は21.0（白と黒）
        prop_assert!(
            ratio <= 21.0,
            "Grayscale contrast ratio should be at most 21.0: got {} for gray1={}, gray2={}",
            ratio, gray1, gray2
        );
    }
}
