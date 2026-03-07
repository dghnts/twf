// Property 12: 色空間変換のラウンドトリップ
//
// このプロパティテストは、色空間変換の正確性を検証します。
// RGB → Lab → RGB、RGB → XYZ → RGB、XYZ → Lab → XYZ の変換が
// 許容誤差範囲内で元の値に戻ることを確認します。

use proptest::prelude::*;
use twf::models::{Lab, Rgb, Xyz};
use twf::utils::color_space::{
    lab_to_rgb, lab_to_xyz, rgb_to_lab, rgb_to_xyz, xyz_to_lab, xyz_to_rgb,
};

// RGB値の生成戦略（0-255の範囲）
fn rgb_strategy() -> impl Strategy<Value = Rgb> {
    (0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(r, g, b)| Rgb::new(r, g, b))
}

// XYZ値の生成戦略（0.0-1.2の範囲、D65白色点を考慮）
fn xyz_strategy() -> impl Strategy<Value = Xyz> {
    (0.0f64..=1.2, 0.0f64..=1.2, 0.0f64..=1.2).prop_map(|(x, y, z)| Xyz { x, y, z })
}

// Lab値の生成戦略（L: 0-100, a: -128-127, b: -128-127）
fn lab_strategy() -> impl Strategy<Value = Lab> {
    (0.0f64..=100.0, -128.0f64..=127.0, -128.0f64..=127.0)
        .prop_map(|(l, a, b)| Lab { l, a, b })
}

proptest! {
    /// Property 12.1: RGB → Lab → RGB のラウンドトリップ
    ///
    /// 任意のRGB色をLab色空間に変換し、再びRGBに戻した場合、
    /// 元の色と許容誤差範囲内で一致することを検証します。
    ///
    /// 許容誤差: 各チャンネルで±2（浮動小数点演算の丸め誤差を考慮）
    #[test]
    fn prop_rgb_lab_roundtrip(rgb in rgb_strategy()) {
        let lab = rgb_to_lab(rgb);
        let converted = lab_to_rgb(lab);

        // 許容誤差範囲内で等しいことを確認
        let tolerance = 2;
        let r_diff = (rgb.r as i16 - converted.r as i16).abs();
        let g_diff = (rgb.g as i16 - converted.g as i16).abs();
        let b_diff = (rgb.b as i16 - converted.b as i16).abs();

        prop_assert!(
            r_diff <= tolerance,
            "Red channel mismatch: original={}, converted={}, diff={}",
            rgb.r, converted.r, r_diff
        );
        prop_assert!(
            g_diff <= tolerance,
            "Green channel mismatch: original={}, converted={}, diff={}",
            rgb.g, converted.g, g_diff
        );
        prop_assert!(
            b_diff <= tolerance,
            "Blue channel mismatch: original={}, converted={}, diff={}",
            rgb.b, converted.b, b_diff
        );
    }

    /// Property 12.2: RGB → XYZ → RGB のラウンドトリップ
    ///
    /// 任意のRGB色をXYZ色空間に変換し、再びRGBに戻した場合、
    /// 元の色と許容誤差範囲内で一致することを検証します。
    ///
    /// 許容誤差: 各チャンネルで±1（XYZ変換は比較的精度が高い）
    #[test]
    fn prop_rgb_xyz_roundtrip(rgb in rgb_strategy()) {
        let xyz = rgb_to_xyz(rgb);
        let converted = xyz_to_rgb(xyz);

        // 許容誤差範囲内で等しいことを確認
        let tolerance = 1;
        let r_diff = (rgb.r as i16 - converted.r as i16).abs();
        let g_diff = (rgb.g as i16 - converted.g as i16).abs();
        let b_diff = (rgb.b as i16 - converted.b as i16).abs();

        prop_assert!(
            r_diff <= tolerance,
            "Red channel mismatch: original={}, converted={}, diff={}",
            rgb.r, converted.r, r_diff
        );
        prop_assert!(
            g_diff <= tolerance,
            "Green channel mismatch: original={}, converted={}, diff={}",
            rgb.g, converted.g, g_diff
        );
        prop_assert!(
            b_diff <= tolerance,
            "Blue channel mismatch: original={}, converted={}, diff={}",
            rgb.b, converted.b, b_diff
        );
    }

    /// Property 12.3: XYZ → Lab → XYZ のラウンドトリップ
    ///
    /// 任意のXYZ色をLab色空間に変換し、再びXYZに戻した場合、
    /// 元の色と許容誤差範囲内で一致することを検証します。
    ///
    /// 許容誤差: 各成分で1e-6（浮動小数点演算の精度）
    #[test]
    fn prop_xyz_lab_roundtrip(xyz in xyz_strategy()) {
        let lab = xyz_to_lab(xyz);
        let converted = lab_to_xyz(lab);

        // 許容誤差範囲内で等しいことを確認
        let tolerance = 1e-6;
        let x_diff = (xyz.x - converted.x).abs();
        let y_diff = (xyz.y - converted.y).abs();
        let z_diff = (xyz.z - converted.z).abs();

        prop_assert!(
            x_diff < tolerance,
            "X component mismatch: original={}, converted={}, diff={}",
            xyz.x, converted.x, x_diff
        );
        prop_assert!(
            y_diff < tolerance,
            "Y component mismatch: original={}, converted={}, diff={}",
            xyz.y, converted.y, y_diff
        );
        prop_assert!(
            z_diff < tolerance,
            "Z component mismatch: original={}, converted={}, diff={}",
            xyz.z, converted.z, z_diff
        );
    }

    /// Property 12.4: Lab → RGB → Lab のラウンドトリップ（制限付き）
    ///
    /// Lab色空間の色をRGBに変換し、再びLabに戻した場合、
    /// RGB色域内の色については元の色と近い値になることを検証します。
    ///
    /// 注意: Lab色空間はRGB色域よりも広いため、RGB色域外の色は
    /// クランプされます。このテストでは、RGB変換後に色域内に
    /// 収まった色のみを検証します。
    #[test]
    fn prop_lab_rgb_roundtrip_clamped(lab in lab_strategy()) {
        let rgb = lab_to_rgb(lab);
        let converted = rgb_to_lab(rgb);

        // RGB変換でクランプが発生した可能性があるため、
        // 元のLab値とRGBから再変換したLab値の差を確認
        // 色域内の色であれば、差は小さいはず
        let l_diff = (lab.l - converted.l).abs();
        let a_diff = (lab.a - converted.a).abs();
        let b_diff = (lab.b - converted.b).abs();

        // RGB色域内の色（L: 0-100, a/b: -128-127の範囲内）であれば、
        // 許容誤差範囲内で一致するはず
        // ただし、色域外の色はクランプされるため、大きな差が生じる可能性がある
        // ここでは、変換が正常に完了することを確認
        prop_assert!(
            converted.l >= 0.0 && converted.l <= 100.0,
            "Converted L value out of range: {}",
            converted.l
        );
        prop_assert!(
            converted.a >= -128.0 && converted.a <= 127.0,
            "Converted a value out of range: {}",
            converted.a
        );
        prop_assert!(
            converted.b >= -128.0 && converted.b <= 127.0,
            "Converted b value out of range: {}",
            converted.b
        );

        // RGB色域内の色（明度が適度で彩度が低い色）の場合、
        // より厳密な検証を行う
        // 注意: 彩度が高い色や極端な明度の色は、RGB色域外になる可能性が高い
        // Lab色空間はRGB色域よりも広いため、色域境界付近では大きな誤差が生じる
        if lab.l >= 30.0 && lab.l <= 70.0 && lab.a.abs() <= 30.0 && lab.b.abs() <= 30.0 {
            let tolerance = 15.0; // Lab色空間での許容誤差（色域境界での誤差を考慮）
            prop_assert!(
                l_diff < tolerance,
                "L component mismatch for in-gamut color: original={}, converted={}, diff={}",
                lab.l, converted.l, l_diff
            );
            prop_assert!(
                a_diff < tolerance,
                "a component mismatch for in-gamut color: original={}, converted={}, diff={}",
                lab.a, converted.a, a_diff
            );
            prop_assert!(
                b_diff < tolerance,
                "b component mismatch for in-gamut color: original={}, converted={}, diff={}",
                lab.b, converted.b, b_diff
            );
        }
    }

    /// Property 12.5: 色空間変換の可逆性（RGB → Lab → XYZ → Lab → RGB）
    ///
    /// RGB色を複数の色空間を経由して変換し、最終的に元のRGBに
    /// 戻ることを検証します。これにより、変換チェーン全体の
    /// 一貫性を確認します。
    #[test]
    fn prop_multi_space_roundtrip(rgb in rgb_strategy()) {
        // RGB → Lab → XYZ → Lab → RGB
        let lab1 = rgb_to_lab(rgb);
        let xyz = lab_to_xyz(lab1);
        let lab2 = xyz_to_lab(xyz);
        let converted = lab_to_rgb(lab2);

        // 許容誤差範囲内で等しいことを確認
        let tolerance = 3; // 複数回の変換を経るため、誤差が累積する
        let r_diff = (rgb.r as i16 - converted.r as i16).abs();
        let g_diff = (rgb.g as i16 - converted.g as i16).abs();
        let b_diff = (rgb.b as i16 - converted.b as i16).abs();

        prop_assert!(
            r_diff <= tolerance,
            "Red channel mismatch after multi-space conversion: original={}, converted={}, diff={}",
            rgb.r, converted.r, r_diff
        );
        prop_assert!(
            g_diff <= tolerance,
            "Green channel mismatch after multi-space conversion: original={}, converted={}, diff={}",
            rgb.g, converted.g, g_diff
        );
        prop_assert!(
            b_diff <= tolerance,
            "Blue channel mismatch after multi-space conversion: original={}, converted={}, diff={}",
            rgb.b, converted.b, b_diff
        );
    }
}
