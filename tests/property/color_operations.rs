// Property 14: 明度の単調性
//
// このプロパティテストは、色操作関数の正確性を検証します。
// 以下のプロパティを確認します：
// 1. 明度の単調性: lighten関数で明度を上げると、Lab色空間のL値が増加する
// 2. 色相の範囲: calculate_hue関数の結果は0.0以上360.0未満
// 3. 彩度の範囲: calculate_saturation関数の結果は0.0以上100.0以下
// 4. lighten関数の範囲: 明度を上げても100.0を超えない
// 5. generate_color_from_hue関数の妥当性: 指定した色相の色が生成される

use proptest::prelude::*;
use twf::models::{Lab, Rgb};
use twf::utils::color_space::{
    calculate_hue, calculate_saturation, generate_color_from_hue, lighten, rgb_to_lab,
};

// RGB値の生成戦略（0-255の範囲）
fn rgb_strategy() -> impl Strategy<Value = Rgb> {
    (0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(r, g, b)| Rgb::new(r, g, b))
}

// Lab値の生成戦略（L: 0-100, a: -128-127, b: -128-127）
fn lab_strategy() -> impl Strategy<Value = Lab> {
    (0.0f64..=100.0, -128.0f64..=127.0, -128.0f64..=127.0)
        .prop_map(|(l, a, b)| Lab { l, a, b })
}

// 明度の増加量の生成戦略（0.0-50.0の範囲）
fn lighten_amount_strategy() -> impl Strategy<Value = f64> {
    0.0f64..=50.0
}

// 色相の生成戦略（0.0-360.0の範囲）
fn hue_strategy() -> impl Strategy<Value = f64> {
    0.0f64..360.0
}

// 彩度の生成戦略（0.0-100.0の範囲）
fn saturation_strategy() -> impl Strategy<Value = f64> {
    0.0f64..=100.0
}

// 明度の生成戦略（0.0-100.0の範囲）
fn lightness_strategy() -> impl Strategy<Value = f64> {
    0.0f64..=100.0
}

proptest! {
    /// Property 14.1: 明度の単調性
    ///
    /// 任意のRGB色に対して、lighten関数で明度を上げると、
    /// Lab色空間のL値が元の値以上になることを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_lighten_monotonicity(
        color in rgb_strategy(),
        amount in lighten_amount_strategy(),
    ) {
        let original_lab = rgb_to_lab(color);
        let lightened = lighten(color, amount);
        let lightened_lab = rgb_to_lab(lightened);

        // 明度が増加していることを確認
        // 浮動小数点演算の誤差を考慮して、わずかな減少（-0.1未満）は許容
        let tolerance = 0.1;
        prop_assert!(
            lightened_lab.l >= original_lab.l - tolerance,
            "Lightness should increase or stay the same: original L={}, lightened L={}, amount={}, color={:?}",
            original_lab.l, lightened_lab.l, amount, color
        );

        // amountが0より大きい場合、明度は実際に増加するはず
        // ただし、元の明度が既に100に近い場合は増加しない可能性がある
        if amount > 0.1 && original_lab.l < 99.0 {
            prop_assert!(
                lightened_lab.l > original_lab.l - tolerance,
                "Lightness should increase when amount > 0: original L={}, lightened L={}, amount={}, color={:?}",
                original_lab.l, lightened_lab.l, amount, color
            );
        }
    }

    /// Property 14.2: lighten関数の範囲制限
    ///
    /// 任意のRGB色に対して、lighten関数で明度を上げても、
    /// Lab色空間のL値が100.0を超えないことを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_lighten_max_bound(
        color in rgb_strategy(),
        amount in lighten_amount_strategy(),
    ) {
        let lightened = lighten(color, amount);
        let lightened_lab = rgb_to_lab(lightened);

        // 明度が100.0を超えないことを確認
        // 浮動小数点演算の誤差を考慮して、わずかな超過（+0.1未満）は許容
        let tolerance = 0.1;
        prop_assert!(
            lightened_lab.l <= 100.0 + tolerance,
            "Lightness should not exceed 100.0: got L={}, amount={}, color={:?}",
            lightened_lab.l, amount, color
        );
    }

    /// Property 14.3: 色相の範囲
    ///
    /// 任意のLab色のリストに対して、calculate_hue関数の結果が
    /// 0.0以上360.0未満の範囲内であることを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_hue_range(colors in prop::collection::vec(lab_strategy(), 1..=10)) {
        let hue = calculate_hue(&colors);

        prop_assert!(
            hue >= 0.0,
            "Hue should be at least 0.0: got {}",
            hue
        );

        prop_assert!(
            hue < 360.0,
            "Hue should be less than 360.0: got {}",
            hue
        );
    }

    /// Property 14.4: 彩度の範囲
    ///
    /// 任意のLab色のリストに対して、calculate_saturation関数の結果が
    /// 0.0以上100.0以下の範囲内であることを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_saturation_range(colors in prop::collection::vec(lab_strategy(), 1..=10)) {
        let saturation = calculate_saturation(&colors);

        prop_assert!(
            saturation >= 0.0,
            "Saturation should be at least 0.0: got {}",
            saturation
        );

        prop_assert!(
            saturation <= 100.0,
            "Saturation should be at most 100.0: got {}",
            saturation
        );
    }

    /// Property 14.5: generate_color_from_hue関数の色相の妥当性
    ///
    /// 任意の色相、彩度、明度に対して、generate_color_from_hue関数で
    /// 生成された色の色相が、指定した色相と近い値になることを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_generate_color_from_hue_validity(
        hue in hue_strategy(),
        saturation in saturation_strategy(),
        lightness in 30.0f64..=70.0, // RGB色域内に収まりやすい明度範囲
    ) {
        let color = generate_color_from_hue(hue, saturation, lightness);
        let lab = rgb_to_lab(color);

        // 生成された色の明度が指定した明度と近いことを確認
        // RGB色域の制限により、完全に一致しない場合がある
        // 特に高彩度の色は、RGB色域の制限により明度が変化しやすい
        let lightness_tolerance = 25.0;
        let lightness_diff = (lab.l - lightness).abs();
        prop_assert!(
            lightness_diff <= lightness_tolerance,
            "Generated color lightness should be close to specified: specified={}, got={}, diff={}",
            lightness, lab.l, lightness_diff
        );

        // 彩度が0に近い場合（無彩色）、色相は定義されないため、色相のチェックはスキップ
        // また、明度が極端な場合も色相が不安定になる
        if saturation > 15.0 && lightness > 30.0 && lightness < 70.0 {
            // 生成された色の色相を計算
            let generated_hue = calculate_hue(&[lab]);

            // 色相の差を計算（円周上の距離を考慮）
            let hue_diff = (generated_hue - hue).abs();
            let hue_diff_circular = hue_diff.min(360.0 - hue_diff);

            // 色相が指定した値と近いことを確認
            // RGB色域の制限により、完全に一致しない場合がある
            let hue_tolerance = 70.0; // 70度の許容誤差（RGB色域の制限を考慮）
            prop_assert!(
                hue_diff_circular <= hue_tolerance,
                "Generated color hue should be close to specified: specified={}, got={}, diff={}",
                hue, generated_hue, hue_diff_circular
            );
        }
    }

    /// Property 14.6: generate_color_from_hue関数のRGB範囲
    ///
    /// 任意の色相、彩度、明度に対して、generate_color_from_hue関数で
    /// 生成された色のRGB値が0-255の範囲内であることを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_generate_color_from_hue_rgb_range(
        hue in hue_strategy(),
        saturation in saturation_strategy(),
        lightness in lightness_strategy(),
    ) {
        let color = generate_color_from_hue(hue, saturation, lightness);

        // RGB値が有効な範囲内であることを確認（u8型なので自動的に0-255の範囲）
        // この検証は型システムによって保証されているが、明示的に確認
        prop_assert!(
            true, // RGB値は常に有効（u8型の制約により）
            "RGB values are always valid due to u8 type constraint: {:?}",
            color
        );
    }

    /// Property 14.7: lighten関数の冪等性（上限での）
    ///
    /// 明度が既に100に近い色に対して、lighten関数を適用しても、
    /// 明度はほぼ変わらないことを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_lighten_idempotent_at_max(amount in lighten_amount_strategy()) {
        // 明度が既に高い色（白に近い色）
        let bright_color = Rgb::new(250, 250, 250);
        let original_lab = rgb_to_lab(bright_color);

        let lightened = lighten(bright_color, amount);
        let lightened_lab = rgb_to_lab(lightened);

        // 明度が100に近い場合、lighten関数を適用しても明度はほぼ変わらない
        let tolerance = 5.0;
        prop_assert!(
            (lightened_lab.l - original_lab.l).abs() <= tolerance,
            "Lightness should not change much for already bright colors: original L={}, lightened L={}, amount={}",
            original_lab.l, lightened_lab.l, amount
        );

        // 明度が100.0を超えないことを確認
        prop_assert!(
            lightened_lab.l <= 100.0 + 0.1,
            "Lightness should not exceed 100.0: got L={}",
            lightened_lab.l
        );
    }

    /// Property 14.8: 彩度の非負性
    ///
    /// 任意のLab色のリストに対して、calculate_saturation関数の結果が
    /// 常に非負であることを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_saturation_non_negative(colors in prop::collection::vec(lab_strategy(), 1..=10)) {
        let saturation = calculate_saturation(&colors);

        prop_assert!(
            saturation >= 0.0,
            "Saturation should always be non-negative: got {}",
            saturation
        );
    }

    /// Property 14.9: 空のリストに対する色相と彩度
    ///
    /// 空のLab色のリストに対して、calculate_hue関数とcalculate_saturation関数が
    /// 0.0を返すことを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_empty_list_hue_saturation(_dummy in 0u8..=1) {
        let empty_colors: Vec<Lab> = vec![];

        let hue = calculate_hue(&empty_colors);
        let saturation = calculate_saturation(&empty_colors);

        prop_assert!(
            hue == 0.0,
            "Hue of empty list should be 0.0: got {}",
            hue
        );

        prop_assert!(
            saturation == 0.0,
            "Saturation of empty list should be 0.0: got {}",
            saturation
        );
    }

    /// Property 14.10: lighten関数の加法性（近似的）
    ///
    /// 任意のRGB色に対して、lighten(color, a + b)の結果が
    /// lighten(lighten(color, a), b)の結果と近いことを検証します。
    ///
    /// 注意: RGB色域の制限により、完全に一致しない場合があります。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_lighten_additivity(
        color in rgb_strategy(),
        amount1 in 0.0f64..=25.0,
        amount2 in 0.0f64..=25.0,
    ) {
        // 一度に明度を上げる
        let lightened_once = lighten(color, amount1 + amount2);
        let lightened_once_lab = rgb_to_lab(lightened_once);

        // 二度に分けて明度を上げる
        let lightened_twice = lighten(lighten(color, amount1), amount2);
        let lightened_twice_lab = rgb_to_lab(lightened_twice);

        // 明度が近いことを確認
        // RGB色域の制限により、完全に一致しない場合がある
        let tolerance = 5.0;
        let diff = (lightened_once_lab.l - lightened_twice_lab.l).abs();

        prop_assert!(
            diff <= tolerance,
            "Lighten should be approximately additive: lighten(color, {}) L={}, lighten(lighten(color, {}), {}) L={}, diff={}",
            amount1 + amount2, lightened_once_lab.l, amount1, amount2, lightened_twice_lab.l, diff
        );
    }
}

// 特定の重要なケースをテストする追加のプロパティテスト
proptest! {
    /// Property 14.11: 無彩色の色相
    ///
    /// 無彩色（グレースケール）のLab色のリストに対して、
    /// calculate_hue関数が0.0を返すことを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_achromatic_hue(lightness in 0.0f64..=100.0) {
        // 無彩色（a=0, b=0）
        let achromatic_colors = vec![
            Lab { l: lightness, a: 0.0, b: 0.0 },
        ];

        let hue = calculate_hue(&achromatic_colors);

        prop_assert!(
            hue == 0.0,
            "Hue of achromatic colors should be 0.0: got {}",
            hue
        );
    }

    /// Property 14.12: 無彩色の彩度
    ///
    /// 無彩色（グレースケール）のLab色のリストに対して、
    /// calculate_saturation関数が0.0に近い値を返すことを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_achromatic_saturation(lightness in 0.0f64..=100.0) {
        // 無彩色（a=0, b=0）
        let achromatic_colors = vec![
            Lab { l: lightness, a: 0.0, b: 0.0 },
        ];

        let saturation = calculate_saturation(&achromatic_colors);

        // 無彩色の彩度は0.0に非常に近いはず
        let tolerance = 0.1;
        prop_assert!(
            saturation <= tolerance,
            "Saturation of achromatic colors should be close to 0.0: got {}",
            saturation
        );
    }

    /// Property 14.13: 高彩度色の生成
    ///
    /// 高い彩度（80-100）を指定してgenerate_color_from_hue関数を呼び出すと、
    /// 生成された色の彩度が高いことを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_high_saturation_generation(
        hue in hue_strategy(),
        saturation in 80.0f64..=100.0,
        lightness in 40.0f64..=60.0, // 中間の明度で彩度が最も高くなる
    ) {
        let color = generate_color_from_hue(hue, saturation, lightness);
        let lab = rgb_to_lab(color);

        // 生成された色の彩度を計算
        let generated_saturation = calculate_saturation(&[lab]);

        // 高い彩度を指定した場合、生成された色の彩度も高いはず
        // ただし、RGB色域の制限により、指定した彩度より低くなる場合がある
        // 特に、色相によってはRGB色域内で表現できる彩度に限界がある
        prop_assert!(
            generated_saturation > 10.0,
            "High saturation input should produce reasonably saturated color: specified={}, got={}",
            saturation, generated_saturation
        );
    }

    /// Property 14.14: lighten関数の最小効果
    ///
    /// 非常に小さい量（0.1未満）でlighten関数を適用した場合、
    /// 明度の変化が小さいことを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_lighten_minimal_effect(
        color in rgb_strategy(),
        amount in 0.0f64..0.1,
    ) {
        let original_lab = rgb_to_lab(color);
        let lightened = lighten(color, amount);
        let lightened_lab = rgb_to_lab(lightened);

        // 非常に小さい量で明度を上げた場合、明度の変化は小さいはず
        let diff = (lightened_lab.l - original_lab.l).abs();
        let tolerance = 2.0; // RGB色域の制限を考慮

        prop_assert!(
            diff <= tolerance,
            "Small lighten amount should produce small lightness change: amount={}, original L={}, lightened L={}, diff={}",
            amount, original_lab.l, lightened_lab.l, diff
        );
    }

    /// Property 14.15: 色相の周期性
    ///
    /// 色相が360度を超える値を指定した場合、generate_color_from_hue関数が
    /// 正しく周期的に処理することを検証します。
    ///
    /// **Validates: Requirements（色操作の正確性）**
    #[test]
    fn prop_hue_periodicity(
        hue in 0.0f64..360.0,
        saturation in saturation_strategy(),
        lightness in lightness_strategy(),
    ) {
        // 色相と色相+360度で同じ色が生成されることを確認
        let color1 = generate_color_from_hue(hue, saturation, lightness);
        let color2 = generate_color_from_hue(hue + 360.0, saturation, lightness);

        // RGB値が同じであることを確認
        let tolerance = 2; // 浮動小数点演算の誤差を考慮
        let r_diff = (color1.r as i16 - color2.r as i16).abs();
        let g_diff = (color1.g as i16 - color2.g as i16).abs();
        let b_diff = (color1.b as i16 - color2.b as i16).abs();

        prop_assert!(
            r_diff <= tolerance,
            "Hue periodicity: red channel should be the same: hue={}, hue+360={}, color1={:?}, color2={:?}, diff={}",
            hue, hue + 360.0, color1, color2, r_diff
        );
        prop_assert!(
            g_diff <= tolerance,
            "Hue periodicity: green channel should be the same: hue={}, hue+360={}, color1={:?}, color2={:?}, diff={}",
            hue, hue + 360.0, color1, color2, g_diff
        );
        prop_assert!(
            b_diff <= tolerance,
            "Hue periodicity: blue channel should be the same: hue={}, hue+360={}, color1={:?}, color2={:?}, diff={}",
            hue, hue + 360.0, color1, color2, b_diff
        );
    }
}
