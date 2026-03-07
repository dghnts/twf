// コントラスト比計算

use crate::models::Rgb;

/// コントラスト比を計算（WCAG 2.1基準）
/// 
/// 2色間のコントラスト比を計算します。
/// WCAG 2.1基準では、通常テキストに対してAA基準で4.5:1以上、AAA基準で7:1以上が推奨されます。
/// 
/// # Arguments
/// 
/// * `color1` - 1つ目の色（RGB）
/// * `color2` - 2つ目の色（RGB）
/// 
/// # Returns
/// 
/// コントラスト比（1.0〜21.0の範囲）
/// 
/// # Example
/// 
/// ```
/// use twf::models::Rgb;
/// use twf::analyzer::contrast::calculate_contrast_ratio;
/// 
/// let white = Rgb::new(255, 255, 255);
/// let black = Rgb::new(0, 0, 0);
/// let ratio = calculate_contrast_ratio(white, black);
/// assert!(ratio >= 21.0); // 白と黒の最大コントラスト比
/// ```
pub fn calculate_contrast_ratio(color1: Rgb, color2: Rgb) -> f64 {
    // 1. 相対輝度を計算
    let l1 = calculate_relative_luminance(color1);
    let l2 = calculate_relative_luminance(color2);
    
    // 2. コントラスト比を計算
    // 明るい方を分子、暗い方を分母にする
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    
    // コントラスト比の式: (lighter + 0.05) / (darker + 0.05)
    (lighter + 0.05) / (darker + 0.05)
}

/// 相対輝度を計算（ITU-R BT.709基準）
/// 
/// RGB色の相対輝度を計算します。
/// 相対輝度は、色の明るさを表す値で、0.0（黒）から1.0（白）の範囲です。
/// 
/// # Arguments
/// 
/// * `color` - RGB色
/// 
/// # Returns
/// 
/// 相対輝度（0.0〜1.0の範囲）
/// 
/// # Example
/// 
/// ```
/// use twf::models::Rgb;
/// use twf::analyzer::contrast::calculate_relative_luminance;
/// 
/// let white = Rgb::new(255, 255, 255);
/// let luminance = calculate_relative_luminance(white);
/// assert!((luminance - 1.0).abs() < 0.01); // 白の相対輝度は約1.0
/// ```
pub fn calculate_relative_luminance(color: Rgb) -> f64 {
    // sRGB → 線形RGB変換
    let r = srgb_to_linear(color.r as f64 / 255.0);
    let g = srgb_to_linear(color.g as f64 / 255.0);
    let b = srgb_to_linear(color.b as f64 / 255.0);
    
    // 相対輝度の計算（ITU-R BT.709）
    // Y = 0.2126 * R + 0.7152 * G + 0.0722 * B
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// sRGB値を線形RGB値に変換
/// 
/// sRGB色空間はガンマ補正が適用されているため、
/// 線形RGB色空間に変換する必要があります。
/// 
/// # Arguments
/// 
/// * `c` - sRGB値（0.0〜1.0の範囲）
/// 
/// # Returns
/// 
/// 線形RGB値（0.0〜1.0の範囲）
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.03928 {
        // 暗い色の場合は線形変換
        c / 12.92
    } else {
        // 明るい色の場合はガンマ補正を解除
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrast_ratio_white_black() {
        // 白と黒の最大コントラスト比は21:1
        let white = Rgb::new(255, 255, 255);
        let black = Rgb::new(0, 0, 0);
        let ratio = calculate_contrast_ratio(white, black);
        
        // 21.0に非常に近い値であることを確認
        assert!((ratio - 21.0).abs() < 0.01, "Expected ~21.0, got {}", ratio);
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        // 同じ色のコントラスト比は1:1
        let gray = Rgb::new(128, 128, 128);
        let ratio = calculate_contrast_ratio(gray, gray);
        
        assert!((ratio - 1.0).abs() < 0.01, "Expected ~1.0, got {}", ratio);
    }

    #[test]
    fn test_contrast_ratio_symmetry() {
        // コントラスト比は対称的（順序を入れ替えても同じ）
        let color1 = Rgb::new(100, 150, 200);
        let color2 = Rgb::new(50, 75, 100);
        
        let ratio1 = calculate_contrast_ratio(color1, color2);
        let ratio2 = calculate_contrast_ratio(color2, color1);
        
        assert!((ratio1 - ratio2).abs() < 0.01, "Ratios should be equal");
    }

    #[test]
    fn test_contrast_ratio_wcag_aa() {
        // WCAG AA基準（4.5:1以上）を満たす例
        let white = Rgb::new(255, 255, 255);
        let dark_gray = Rgb::new(100, 100, 100);
        let ratio = calculate_contrast_ratio(white, dark_gray);
        
        assert!(ratio >= 4.5, "Should meet WCAG AA standard (4.5:1)");
    }

    #[test]
    fn test_relative_luminance_white() {
        // 白の相対輝度は1.0
        let white = Rgb::new(255, 255, 255);
        let luminance = calculate_relative_luminance(white);
        
        assert!((luminance - 1.0).abs() < 0.01, "Expected ~1.0, got {}", luminance);
    }

    #[test]
    fn test_relative_luminance_black() {
        // 黒の相対輝度は0.0
        let black = Rgb::new(0, 0, 0);
        let luminance = calculate_relative_luminance(black);
        
        assert!(luminance.abs() < 0.01, "Expected ~0.0, got {}", luminance);
    }

    #[test]
    fn test_relative_luminance_range() {
        // 相対輝度は0.0〜1.0の範囲内
        let colors = vec![
            Rgb::new(255, 0, 0),    // 赤
            Rgb::new(0, 255, 0),    // 緑
            Rgb::new(0, 0, 255),    // 青
            Rgb::new(128, 128, 128), // グレー
        ];
        
        for color in colors {
            let luminance = calculate_relative_luminance(color);
            assert!(luminance >= 0.0 && luminance <= 1.0, 
                "Luminance {} out of range for color {:?}", luminance, color);
        }
    }

    #[test]
    fn test_srgb_to_linear_dark() {
        // 暗い色の線形変換
        let result = srgb_to_linear(0.03);
        let expected = 0.03 / 12.92;
        
        assert!((result - expected).abs() < 0.0001, "Expected {}, got {}", expected, result);
    }

    #[test]
    fn test_srgb_to_linear_bright() {
        // 明るい色のガンマ補正解除
        let result = srgb_to_linear(0.5);
        let expected = ((0.5 + 0.055) / 1.055_f64).powf(2.4);
        
        assert!((result - expected).abs() < 0.0001, "Expected {}, got {}", expected, result);
    }
}
