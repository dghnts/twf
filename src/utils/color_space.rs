// 色空間変換

use crate::models::{Lab, Rgb, Xyz};

/// sRGB → 線形RGB変換
/// 
/// sRGB色空間の値（0.0-1.0）を線形RGB色空間に変換します。
/// ガンマ補正を解除する処理です。
pub fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// 線形RGB → sRGB変換
/// 
/// 線形RGB色空間の値を、sRGB色空間（0.0-1.0）に変換します。
/// ガンマ補正を適用する処理です。
pub fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// RGB → XYZ変換
/// 
/// RGB色空間（0-255）をCIE XYZ色空間に変換します。
/// D65白色点を使用します。
pub fn rgb_to_xyz(rgb: Rgb) -> Xyz {
    // sRGB → 線形RGB
    let r = srgb_to_linear(rgb.r as f64 / 255.0);
    let g = srgb_to_linear(rgb.g as f64 / 255.0);
    let b = srgb_to_linear(rgb.b as f64 / 255.0);
    
    // 線形RGB → XYZ（D65白色点）
    let x = r * 0.4124564 + g * 0.3575761 + b * 0.1804375;
    let y = r * 0.2126729 + g * 0.7151522 + b * 0.0721750;
    let z = r * 0.0193339 + g * 0.1191920 + b * 0.9503041;
    
    Xyz { x, y, z }
}

/// XYZ → RGB変換
/// 
/// CIE XYZ色空間をRGB色空間（0-255）に変換します。
/// D65白色点を使用します。
pub fn xyz_to_rgb(xyz: Xyz) -> Rgb {
    // XYZ → 線形RGB
    let r = xyz.x *  3.2404542 + xyz.y * -1.5371385 + xyz.z * -0.4985314;
    let g = xyz.x * -0.9692660 + xyz.y *  1.8760108 + xyz.z *  0.0415560;
    let b = xyz.x *  0.0556434 + xyz.y * -0.2040259 + xyz.z *  1.0572252;
    
    // 線形RGB → sRGB
    let r = linear_to_srgb(r) * 255.0;
    let g = linear_to_srgb(g) * 255.0;
    let b = linear_to_srgb(b) * 255.0;
    
    // クランプ（0-255の範囲に収める）
    let r = r.clamp(0.0, 255.0) as u8;
    let g = g.clamp(0.0, 255.0) as u8;
    let b = b.clamp(0.0, 255.0) as u8;
    
    Rgb { r, g, b }
}

/// Lab色空間の変換関数（XYZ → Lab用）
/// 
/// CIE Lab色空間の変換に使用される非線形関数です。
fn lab_f(t: f64) -> f64 {
    let delta: f64 = 6.0 / 29.0;
    if t > delta.powi(3) {
        t.powf(1.0 / 3.0)
    } else {
        t / (3.0 * delta.powi(2)) + 4.0 / 29.0
    }
}

/// Lab色空間の逆変換関数（Lab → XYZ用）
/// 
/// CIE Lab色空間からXYZ色空間への変換に使用される逆関数です。
fn lab_f_inv(t: f64) -> f64 {
    let delta: f64 = 6.0 / 29.0;
    if t > delta {
        t.powi(3)
    } else {
        3.0 * delta.powi(2) * (t - 4.0 / 29.0)
    }
}

/// XYZ → Lab変換
/// 
/// CIE XYZ色空間をCIE Lab色空間に変換します。
/// D65白色点を使用します。
pub fn xyz_to_lab(xyz: Xyz) -> Lab {
    // D65白色点で正規化
    let xn = 0.95047;
    let yn = 1.00000;
    let zn = 1.08883;
    
    let fx = lab_f(xyz.x / xn);
    let fy = lab_f(xyz.y / yn);
    let fz = lab_f(xyz.z / zn);
    
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    
    Lab { l, a, b }
}

/// Lab → XYZ変換
/// 
/// CIE Lab色空間をCIE XYZ色空間に変換します。
/// D65白色点を使用します。
pub fn lab_to_xyz(lab: Lab) -> Xyz {
    let fy = (lab.l + 16.0) / 116.0;
    let fx = lab.a / 500.0 + fy;
    let fz = fy - lab.b / 200.0;
    
    // D65白色点
    let xn = 0.95047;
    let yn = 1.00000;
    let zn = 1.08883;
    
    let x = xn * lab_f_inv(fx);
    let y = yn * lab_f_inv(fy);
    let z = zn * lab_f_inv(fz);
    
    Xyz { x, y, z }
}

/// RGB → Lab変換
/// 
/// RGB色空間（0-255）をCIE Lab色空間に変換します。
/// RGB → XYZ → Labの変換チェーンを使用します。
pub fn rgb_to_lab(rgb: Rgb) -> Lab {
    let xyz = rgb_to_xyz(rgb);
    xyz_to_lab(xyz)
}

/// Lab → RGB変換
/// 
/// CIE Lab色空間をRGB色空間（0-255）に変換します。
/// Lab → XYZ → RGBの変換チェーンを使用します。
pub fn lab_to_rgb(lab: Lab) -> Rgb {
    let xyz = lab_to_xyz(lab);
    xyz_to_rgb(xyz)
}

/// 彩度を計算
/// 
/// Lab色空間のa, b成分から彩度を計算します。
/// 彩度は色の鮮やかさを表し、0-100の範囲で返されます。
/// 
/// # 引数
/// * `colors` - Lab色空間の色のスライス
/// 
/// # 戻り値
/// 平均彩度（0-100）
pub fn calculate_saturation(colors: &[Lab]) -> f64 {
    if colors.is_empty() {
        return 0.0;
    }
    
    // 各色の彩度を計算（a, b成分のユークリッド距離）
    let total_saturation: f64 = colors.iter()
        .map(|lab| (lab.a.powi(2) + lab.b.powi(2)).sqrt())
        .sum();
    
    // 平均彩度を計算し、0-100の範囲に正規化
    // Lab色空間のa, bの最大値は約128なので、最大彩度は約181
    let avg_saturation = total_saturation / colors.len() as f64;
    (avg_saturation / 181.0 * 100.0).min(100.0)
}

/// 色相を計算
/// 
/// Lab色空間のa, b成分から色相を計算します。
/// 色相は色の種類を表し、0-360度の範囲で返されます。
/// 
/// # 引数
/// * `colors` - Lab色空間の色のスライス
/// 
/// # 戻り値
/// 平均色相（0-360度）
pub fn calculate_hue(colors: &[Lab]) -> f64 {
    if colors.is_empty() {
        return 0.0;
    }
    
    // 各色の色相を計算（a, b成分からatan2を使用）
    let hues: Vec<f64> = colors.iter()
        .filter(|lab| lab.a.abs() > 0.01 || lab.b.abs() > 0.01) // 無彩色を除外
        .map(|lab| {
            let hue_rad = lab.b.atan2(lab.a);
            // ラジアンから度に変換し、0-360の範囲に正規化
            let hue_deg = hue_rad.to_degrees();
            if hue_deg < 0.0 {
                hue_deg + 360.0
            } else {
                hue_deg
            }
        })
        .collect();
    
    if hues.is_empty() {
        return 0.0;
    }
    
    // 平均色相を計算（円周上の平均）
    let sum_sin: f64 = hues.iter().map(|h| h.to_radians().sin()).sum();
    let sum_cos: f64 = hues.iter().map(|h| h.to_radians().cos()).sum();
    let avg_hue_rad = sum_sin.atan2(sum_cos);
    let avg_hue_deg = avg_hue_rad.to_degrees();
    
    if avg_hue_deg < 0.0 {
        avg_hue_deg + 360.0
    } else {
        avg_hue_deg
    }
}

/// 色を明るくする
/// 
/// RGB色の明度を指定した量だけ上げます。
/// Lab色空間で明度を調整してからRGBに戻します。
/// 
/// # 引数
/// * `color` - 元のRGB色
/// * `amount` - 明度を上げる量（0-100）
/// 
/// # 戻り値
/// 明るくされたRGB色
pub fn lighten(color: Rgb, amount: f64) -> Rgb {
    let mut lab = rgb_to_lab(color);
    
    // 明度を上げる（最大100まで）
    lab.l = (lab.l + amount).min(100.0);
    
    lab_to_rgb(lab)
}

/// 色相から色を生成
/// 
/// 指定された色相、彩度、明度からRGB色を生成します。
/// 
/// # 引数
/// * `hue` - 色相（0-360度）
/// * `saturation` - 彩度（0-100）
/// * `lightness` - 明度（0-100）
/// 
/// # 戻り値
/// 生成されたRGB色
pub fn generate_color_from_hue(hue: f64, saturation: f64, lightness: f64) -> Rgb {
    // 色相を0-360の範囲に正規化
    let hue = hue % 360.0;
    let hue_rad = hue.to_radians();
    
    // 彩度を0-181の範囲に変換（Lab色空間の最大彩度）
    let chroma = saturation / 100.0 * 181.0;
    
    // Lab色空間のa, b成分を計算
    let a = chroma * hue_rad.cos();
    let b = chroma * hue_rad.sin();
    
    // Lab色を作成
    let lab = Lab {
        l: lightness,
        a,
        b,
    };
    
    lab_to_rgb(lab)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_to_linear() {
        // 暗い値のテスト（線形領域）
        let result = srgb_to_linear(0.03);
        assert!((result - 0.03 / 12.92).abs() < 1e-6);
        
        // 明るい値のテスト（ガンマ補正領域）
        let result = srgb_to_linear(0.5);
        let expected = ((0.5_f64 + 0.055) / 1.055).powf(2.4);
        assert!((result - expected).abs() < 1e-6);
    }

    #[test]
    fn test_linear_to_srgb() {
        // 暗い値のテスト（線形領域）
        let result = linear_to_srgb(0.002);
        assert!((result - 12.92 * 0.002).abs() < 1e-6);
        
        // 明るい値のテスト（ガンマ補正領域）
        let result = linear_to_srgb(0.5);
        let expected = 1.055 * 0.5_f64.powf(1.0 / 2.4) - 0.055;
        assert!((result - expected).abs() < 1e-6);
    }

    #[test]
    fn test_rgb_to_xyz_black() {
        let black = Rgb::new(0, 0, 0);
        let xyz = rgb_to_xyz(black);
        assert!((xyz.x - 0.0).abs() < 1e-6);
        assert!((xyz.y - 0.0).abs() < 1e-6);
        assert!((xyz.z - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_rgb_to_xyz_white() {
        let white = Rgb::new(255, 255, 255);
        let xyz = rgb_to_xyz(white);
        // 白色はD65白色点に近い値になるはず
        assert!((xyz.x - 0.95047).abs() < 0.01);
        assert!((xyz.y - 1.00000).abs() < 0.01);
        assert!((xyz.z - 1.08883).abs() < 0.01);
    }

    #[test]
    fn test_xyz_to_rgb_black() {
        let xyz = Xyz { x: 0.0, y: 0.0, z: 0.0 };
        let rgb = xyz_to_rgb(xyz);
        assert_eq!(rgb.r, 0);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_xyz_to_rgb_white() {
        let xyz = Xyz { x: 0.95047, y: 1.00000, z: 1.08883 };
        let rgb = xyz_to_rgb(xyz);
        // 白色に近い値になるはず（誤差を考慮）
        assert!(rgb.r >= 250);
        assert!(rgb.g >= 250);
        assert!(rgb.b >= 250);
    }

    #[test]
    fn test_rgb_to_lab_black() {
        let black = Rgb::new(0, 0, 0);
        let lab = rgb_to_lab(black);
        // 黒のL値は0に近いはず
        assert!(lab.l < 1.0);
    }

    #[test]
    fn test_rgb_to_lab_white() {
        let white = Rgb::new(255, 255, 255);
        let lab = rgb_to_lab(white);
        // 白のL値は100に近いはず
        assert!(lab.l > 99.0);
        // a, b値は0に近いはず（無彩色）
        assert!(lab.a.abs() < 1.0);
        assert!(lab.b.abs() < 1.0);
    }

    #[test]
    fn test_lab_to_rgb_black() {
        let lab = Lab { l: 0.0, a: 0.0, b: 0.0 };
        let rgb = lab_to_rgb(lab);
        // 黒に近い値になるはず
        assert!(rgb.r < 5);
        assert!(rgb.g < 5);
        assert!(rgb.b < 5);
    }

    #[test]
    fn test_lab_to_rgb_white() {
        let lab = Lab { l: 100.0, a: 0.0, b: 0.0 };
        let rgb = lab_to_rgb(lab);
        // 白に近い値になるはず
        assert!(rgb.r >= 250);
        assert!(rgb.g >= 250);
        assert!(rgb.b >= 250);
    }

    #[test]
    fn test_rgb_lab_roundtrip() {
        // 様々な色でラウンドトリップテスト
        let test_colors = vec![
            Rgb::new(255, 0, 0),    // 赤
            Rgb::new(0, 255, 0),    // 緑
            Rgb::new(0, 0, 255),    // 青
            Rgb::new(128, 128, 128), // グレー
            Rgb::new(255, 255, 0),  // 黄色
        ];

        for original in test_colors {
            let lab = rgb_to_lab(original);
            let converted = lab_to_rgb(lab);
            
            // 許容誤差範囲内で等しいことを確認（浮動小数点演算の誤差を考慮）
            let tolerance = 2;
            assert!(
                (original.r as i16 - converted.r as i16).abs() <= tolerance,
                "Red channel mismatch: {} vs {} for color {:?}",
                original.r, converted.r, original
            );
            assert!(
                (original.g as i16 - converted.g as i16).abs() <= tolerance,
                "Green channel mismatch: {} vs {} for color {:?}",
                original.g, converted.g, original
            );
            assert!(
                (original.b as i16 - converted.b as i16).abs() <= tolerance,
                "Blue channel mismatch: {} vs {} for color {:?}",
                original.b, converted.b, original
            );
        }
    }

    #[test]
    fn test_xyz_lab_roundtrip() {
        let test_xyz = vec![
            Xyz { x: 0.0, y: 0.0, z: 0.0 },
            Xyz { x: 0.5, y: 0.5, z: 0.5 },
            Xyz { x: 0.95047, y: 1.00000, z: 1.08883 },
        ];

        for original in test_xyz {
            let lab = xyz_to_lab(original);
            let converted = lab_to_xyz(lab);
            
            // 許容誤差範囲内で等しいことを確認
            let tolerance = 1e-6;
            assert!(
                (original.x - converted.x).abs() < tolerance,
                "X mismatch: {} vs {}",
                original.x, converted.x
            );
            assert!(
                (original.y - converted.y).abs() < tolerance,
                "Y mismatch: {} vs {}",
                original.y, converted.y
            );
            assert!(
                (original.z - converted.z).abs() < tolerance,
                "Z mismatch: {} vs {}",
                original.z, converted.z
            );
        }
    }

    #[test]
    fn test_calculate_saturation() {
        // 無彩色（グレー）の彩度は0に近いはず
        let gray = Rgb::new(128, 128, 128);
        let lab_gray = rgb_to_lab(gray);
        let saturation = calculate_saturation(&[lab_gray]);
        assert!(saturation < 10.0, "Gray saturation should be low, got {}", saturation);
        
        // 鮮やかな赤の彩度は高いはず
        let red = Rgb::new(255, 0, 0);
        let lab_red = rgb_to_lab(red);
        let saturation = calculate_saturation(&[lab_red]);
        assert!(saturation > 50.0, "Red saturation should be high, got {}", saturation);
        
        // 空のスライスは0を返すはず
        let saturation = calculate_saturation(&[]);
        assert_eq!(saturation, 0.0);
    }

    #[test]
    fn test_calculate_hue() {
        // 赤の色相は約0度
        let red = Rgb::new(255, 0, 0);
        let lab_red = rgb_to_lab(red);
        let hue = calculate_hue(&[lab_red]);
        // 赤は0度付近または360度付近
        assert!(hue < 45.0 || hue > 315.0, "Red hue should be near 0/360 degrees, got {}", hue);
        
        // 緑の色相は約120度
        let green = Rgb::new(0, 255, 0);
        let lab_green = rgb_to_lab(green);
        let hue = calculate_hue(&[lab_green]);
        assert!(hue > 90.0 && hue < 180.0, "Green hue should be around 120 degrees, got {}", hue);
        
        // 青の色相（Lab色空間では約306度）
        let blue = Rgb::new(0, 0, 255);
        let lab_blue = rgb_to_lab(blue);
        let hue = calculate_hue(&[lab_blue]);
        // Lab色空間での青は約270-330度の範囲
        assert!(hue > 270.0 && hue < 330.0, "Blue hue should be around 306 degrees in Lab space, got {}", hue);
        
        // 空のスライスは0を返すはず
        let hue = calculate_hue(&[]);
        assert_eq!(hue, 0.0);
    }

    #[test]
    fn test_lighten() {
        // 黒を明るくする
        let black = Rgb::new(0, 0, 0);
        let lightened = lighten(black, 50.0);
        let lab_lightened = rgb_to_lab(lightened);
        
        // 明度が上がっていることを確認
        assert!(lab_lightened.l > 40.0, "Lightened black should have L > 40, got {}", lab_lightened.l);
        
        // 白を明るくしても変わらない（すでに最大明度）
        let white = Rgb::new(255, 255, 255);
        let lightened_white = lighten(white, 50.0);
        let lab_white = rgb_to_lab(white);
        let lab_lightened_white = rgb_to_lab(lightened_white);
        
        // 明度はほぼ同じはず
        assert!((lab_white.l - lab_lightened_white.l).abs() < 5.0);
    }

    #[test]
    fn test_generate_color_from_hue() {
        // 赤を生成（色相0度）
        let red = generate_color_from_hue(0.0, 80.0, 50.0);
        assert!(red.r > 100, "Generated red should have high R component");
        
        // 緑を生成（色相120度）
        let green = generate_color_from_hue(120.0, 80.0, 50.0);
        assert!(green.g > 100, "Generated green should have high G component");
        
        // 青を生成（色相240度）
        let blue = generate_color_from_hue(240.0, 80.0, 50.0);
        assert!(blue.b > 100, "Generated blue should have high B component");
        
        // 彩度0の色は無彩色になるはず
        let gray = generate_color_from_hue(0.0, 0.0, 50.0);
        let diff = (gray.r as i16 - gray.g as i16).abs().max((gray.g as i16 - gray.b as i16).abs());
        assert!(diff < 20, "Low saturation color should be grayish, got RGB({}, {}, {})", gray.r, gray.g, gray.b);
    }

    #[test]
    fn test_lighten_monotonicity() {
        // 明度の単調性をテスト（Property 14）
        let test_colors = vec![
            Rgb::new(100, 50, 50),
            Rgb::new(50, 100, 50),
            Rgb::new(50, 50, 100),
        ];
        
        for color in test_colors {
            let original_lab = rgb_to_lab(color);
            let lightened = lighten(color, 20.0);
            let lightened_lab = rgb_to_lab(lightened);
            
            assert!(
                lightened_lab.l >= original_lab.l - 0.01, // 浮動小数点誤差を考慮
                "Lightened color should have L >= original L, original: {}, lightened: {}",
                original_lab.l, lightened_lab.l
            );
        }
    }
}

