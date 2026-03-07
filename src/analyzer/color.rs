// 色解析

use crate::models::{ColorInfo, Rgb};
use crate::utils::color_space::{rgb_to_lab, calculate_saturation, calculate_hue};

/// 色解析器
/// 
/// 単一の背景色から色情報を抽出します。
pub struct ColorAnalyzer;

impl ColorAnalyzer {
    /// 背景色を解析
    /// 
    /// 単一のRGB色から色情報を抽出します。
    /// 
    /// # 引数
    /// * `bg_color` - 解析する背景色（RGB）
    /// 
    /// # 戻り値
    /// 抽出された色情報（ColorInfo）
    /// 
    /// # 処理フロー
    /// 1. RGB → Lab変換
    /// 2. 明度を取得
    /// 3. 彩度と色相を計算
    /// 4. ColorInfo構造体を返す
    pub fn analyze(bg_color: Rgb) -> ColorInfo {
        // 1. RGB → Lab変換
        let lab = rgb_to_lab(bg_color);
        
        // 2. 明度を取得
        let lightness = lab.l;
        
        // 3. 彩度と色相を計算
        // 単一色なので、その色のみを含む配列を渡す
        let saturation = calculate_saturation(&[lab]);
        let hue = calculate_hue(&[lab]);
        
        // 4. ColorInfo構造体を返す
        ColorInfo {
            dominant_colors: vec![lab],
            average_lightness: lightness,
            saturation,
            hue,
            is_dark: lightness < 50.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_black() {
        // 黒の解析
        let black = Rgb::new(0, 0, 0);
        let info = ColorAnalyzer::analyze(black);
        
        // 明度は非常に低いはず
        assert!(info.average_lightness < 5.0, "Black should have very low lightness");
        
        // 暗い背景と判定されるはず
        assert!(info.is_dark, "Black should be classified as dark");
        
        // 彩度は低いはず（無彩色）
        assert!(info.saturation < 10.0, "Black should have low saturation");
    }

    #[test]
    fn test_analyze_white() {
        // 白の解析
        let white = Rgb::new(255, 255, 255);
        let info = ColorAnalyzer::analyze(white);
        
        // 明度は非常に高いはず
        assert!(info.average_lightness > 95.0, "White should have very high lightness");
        
        // 明るい背景と判定されるはず
        assert!(!info.is_dark, "White should not be classified as dark");
        
        // 彩度は低いはず（無彩色）
        assert!(info.saturation < 10.0, "White should have low saturation");
    }

    #[test]
    fn test_analyze_red() {
        // 赤の解析
        let red = Rgb::new(255, 0, 0);
        let info = ColorAnalyzer::analyze(red);
        
        // 彩度は高いはず
        assert!(info.saturation > 50.0, "Red should have high saturation, got {}", info.saturation);
        
        // 色相は赤付近（0度または360度付近）
        assert!(info.hue < 45.0 || info.hue > 315.0, "Red hue should be near 0/360 degrees, got {}", info.hue);
    }

    #[test]
    fn test_analyze_green() {
        // 緑の解析
        let green = Rgb::new(0, 255, 0);
        let info = ColorAnalyzer::analyze(green);
        
        // 彩度は高いはず
        assert!(info.saturation > 50.0, "Green should have high saturation, got {}", info.saturation);
        
        // 色相は緑付近（120度付近）
        assert!(info.hue > 90.0 && info.hue < 180.0, "Green hue should be around 120 degrees, got {}", info.hue);
    }

    #[test]
    fn test_analyze_blue() {
        // 青の解析
        let blue = Rgb::new(0, 0, 255);
        let info = ColorAnalyzer::analyze(blue);
        
        // 彩度は高いはず
        assert!(info.saturation > 50.0, "Blue should have high saturation, got {}", info.saturation);
        
        // 色相は青付近（Lab色空間では約270-330度）
        assert!(info.hue > 270.0 && info.hue < 330.0, "Blue hue should be around 306 degrees in Lab space, got {}", info.hue);
    }

    #[test]
    fn test_analyze_gray() {
        // グレーの解析
        let gray = Rgb::new(128, 128, 128);
        let info = ColorAnalyzer::analyze(gray);
        
        // 明度は中間付近
        assert!(info.average_lightness > 40.0 && info.average_lightness < 60.0, 
                "Gray should have medium lightness, got {}", info.average_lightness);
        
        // 彩度は低いはず（無彩色）
        assert!(info.saturation < 10.0, "Gray should have low saturation, got {}", info.saturation);
    }

    #[test]
    fn test_analyze_dark_threshold() {
        // 明度50を境界として暗い/明るいが判定されることを確認
        
        // 明度が50未満の色（暗い）
        let dark_color = Rgb::new(50, 50, 50);
        let info = ColorAnalyzer::analyze(dark_color);
        assert!(info.is_dark, "Dark color should be classified as dark");
        
        // 明度が50以上の色（明るい）
        let light_color = Rgb::new(200, 200, 200);
        let info = ColorAnalyzer::analyze(light_color);
        assert!(!info.is_dark, "Light color should not be classified as dark");
    }

    #[test]
    fn test_analyze_dominant_colors() {
        // dominant_colorsに1つの色が含まれることを確認
        let color = Rgb::new(100, 150, 200);
        let info = ColorAnalyzer::analyze(color);
        
        assert_eq!(info.dominant_colors.len(), 1, "Should have exactly one dominant color");
        
        // dominant_colorsの色が入力色のLab変換と一致することを確認
        let expected_lab = rgb_to_lab(color);
        let actual_lab = info.dominant_colors[0];
        
        assert!((expected_lab.l - actual_lab.l).abs() < 0.01);
        assert!((expected_lab.a - actual_lab.a).abs() < 0.01);
        assert!((expected_lab.b - actual_lab.b).abs() < 0.01);
    }

    #[test]
    fn test_analyze_various_colors() {
        // 様々な色で解析が正常に動作することを確認
        let test_colors = vec![
            Rgb::new(255, 255, 0),   // 黄色
            Rgb::new(255, 0, 255),   // マゼンタ
            Rgb::new(0, 255, 255),   // シアン
            Rgb::new(128, 0, 0),     // 暗い赤
            Rgb::new(0, 128, 0),     // 暗い緑
            Rgb::new(0, 0, 128),     // 暗い青
        ];

        for color in test_colors {
            let info = ColorAnalyzer::analyze(color);
            
            // 基本的な妥当性チェック
            assert!(info.average_lightness >= 0.0 && info.average_lightness <= 100.0,
                    "Lightness should be in range 0-100, got {}", info.average_lightness);
            assert!(info.saturation >= 0.0 && info.saturation <= 100.0,
                    "Saturation should be in range 0-100, got {}", info.saturation);
            assert!(info.hue >= 0.0 && info.hue < 360.0,
                    "Hue should be in range 0-360, got {}", info.hue);
            assert_eq!(info.dominant_colors.len(), 1,
                      "Should have exactly one dominant color");
        }
    }
}
