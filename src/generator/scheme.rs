// カラースキーム生成

use crate::analyzer::contrast::calculate_contrast_ratio;
use crate::models::{AnsiColors, ColorInfo, ColorScheme, Lab, Rgb, Result, TwfError};
use crate::utils::color_space::{generate_color_from_hue, lab_to_rgb, lighten, rgb_to_lab};

/// カラースキーム生成器
pub struct SchemeGenerator {
    /// 最小コントラスト比（WCAG基準）
    contrast_ratio: f64,
}

impl SchemeGenerator {
    /// 新しいSchemeGeneratorを作成
    /// 
    /// # Arguments
    /// 
    /// * `contrast_ratio` - 最小コントラスト比（デフォルト: 4.5 = WCAG AA基準）
    pub fn new(contrast_ratio: f64) -> Self {
        Self { contrast_ratio }
    }

    /// デフォルトのSchemeGeneratorを作成（WCAG AA基準: 4.5:1）
    pub fn default() -> Self {
        Self::new(4.5)
    }

    /// カラースキームを生成
    /// 
    /// ColorInfoからANSIカラースキームを生成します。
    /// 
    /// # Arguments
    /// 
    /// * `color_info` - 色情報
    /// 
    /// # Returns
    /// 
    /// 生成されたカラースキーム
    pub fn generate(&self, color_info: &ColorInfo) -> Result<ColorScheme> {
        // 1. 背景色を取得（最も支配的な色）
        let background = color_info.dominant_colors[0];
        let bg_rgb = lab_to_rgb(background);

        // 2. フォアグラウンドカラーを計算
        let foreground = self.calculate_foreground_color(color_info)?;

        // 3. ANSI 16色を生成
        let ansi_colors = self.generate_ansi_colors(color_info, foreground, bg_rgb);

        // 4. 256色パレットを生成（オプション）
        let palette_256 = if supports_256_colors() {
            Some(self.generate_256_palette(color_info))
        } else {
            None
        };

        // 5. True Colorサポートを検出
        let supports_true_color = detect_true_color_support();

        Ok(ColorScheme {
            foreground,
            background,
            ansi_colors,
            palette_256,
            supports_true_color,
        })
    }

    /// フォアグラウンドカラーを計算（コントラスト比を考慮）
    /// 
    /// 背景色に対して十分なコントラスト比を持つフォアグラウンドカラーを計算します。
    /// WCAG AA基準（4.5:1以上）を満たすように調整します。
    /// 
    /// # Arguments
    /// 
    /// * `color_info` - 色情報
    /// 
    /// # Returns
    /// 
    /// フォアグラウンドカラー（RGB）
    pub fn calculate_foreground_color(&self, color_info: &ColorInfo) -> Result<Rgb> {
        let bg = lab_to_rgb(color_info.dominant_colors[0]);

        // 明るい背景 → 暗いフォアグラウンド
        // 暗い背景 → 明るいフォアグラウンド
        let candidate = if color_info.is_dark {
            Rgb::new(240, 240, 240) // ほぼ白
        } else {
            Rgb::new(30, 30, 30) // ほぼ黒
        };

        // コントラスト比をチェック
        let contrast = calculate_contrast_ratio(bg, candidate);
        if contrast < self.contrast_ratio {
            // コントラスト比が不足している場合は調整
            self.adjust_for_contrast(bg, candidate, self.contrast_ratio)
        } else {
            Ok(candidate)
        }
    }

    /// コントラスト比を満たすように色を調整
    /// 
    /// 二分探索でコントラスト比を満たす色を見つけます。
    /// 
    /// # Arguments
    /// 
    /// * `bg` - 背景色
    /// * `fg` - フォアグラウンドカラーの候補
    /// * `target_contrast` - 目標コントラスト比
    /// 
    /// # Returns
    /// 
    /// 調整されたフォアグラウンドカラー
    fn adjust_for_contrast(&self, bg: Rgb, fg: Rgb, target_contrast: f64) -> Result<Rgb> {
        let bg_lab = rgb_to_lab(bg);
        let fg_lab = rgb_to_lab(fg);

        // 明度を調整する方向を決定
        let direction = if fg_lab.l > bg_lab.l { 1.0 } else { -1.0 };

        // 二分探索でコントラスト比を満たす明度を見つける
        for step in 0..100 {
            let adjusted_l = fg_lab.l + direction * step as f64;

            // 明度が範囲外の場合はスキップ
            if adjusted_l < 0.0 || adjusted_l > 100.0 {
                continue;
            }

            let adjusted_lab = Lab {
                l: adjusted_l,
                a: fg_lab.a,
                b: fg_lab.b,
            };
            let adjusted_rgb = lab_to_rgb(adjusted_lab);

            let contrast = calculate_contrast_ratio(bg, adjusted_rgb);
            if contrast >= target_contrast {
                return Ok(adjusted_rgb);
            }
        }

        // 最大限調整しても不足する場合は、白または黒を返す
        let fallback = if direction > 0.0 {
            Rgb::new(255, 255, 255)
        } else {
            Rgb::new(0, 0, 0)
        };

        // フォールバックでもコントラスト比が不足する場合はエラー
        let fallback_contrast = calculate_contrast_ratio(bg, fallback);
        if fallback_contrast < target_contrast {
            Err(TwfError::InsufficientContrast {
                actual: fallback_contrast,
                required: target_contrast,
            })
        } else {
            Ok(fallback)
        }
    }

    /// ANSI 16色を生成
    /// 
    /// 背景の色相に基づいて調和する色を生成します。
    /// 
    /// # Arguments
    /// 
    /// * `color_info` - 色情報
    /// * `fg` - フォアグラウンドカラー
    /// * `bg` - 背景色
    /// 
    /// # Returns
    /// 
    /// ANSI 16色
    fn generate_ansi_colors(&self, color_info: &ColorInfo, _fg: Rgb, _bg: Rgb) -> AnsiColors {
        let base_hue = color_info.hue;

        // 基本8色を生成（色相環を使用）
        let black = if color_info.is_dark {
            Rgb::new(40, 40, 40)
        } else {
            Rgb::new(0, 0, 0)
        };

        let white = if color_info.is_dark {
            Rgb::new(220, 220, 220)
        } else {
            Rgb::new(255, 255, 255)
        };

        // 色相環から調和する色を選択
        // 明度は背景に応じて調整
        let lightness = if color_info.is_dark { 60.0 } else { 45.0 };

        let red = generate_color_from_hue(base_hue + 0.0, 70.0, lightness);
        let green = generate_color_from_hue(base_hue + 120.0, 70.0, lightness);
        let yellow = generate_color_from_hue(base_hue + 60.0, 70.0, lightness + 10.0);
        let blue = generate_color_from_hue(base_hue + 240.0, 70.0, lightness);
        let magenta = generate_color_from_hue(base_hue + 300.0, 70.0, lightness);
        let cyan = generate_color_from_hue(base_hue + 180.0, 70.0, lightness + 10.0);

        // 明るいバリアントを生成（明度を上げる）
        let bright_black = lighten(black, 30.0);
        let bright_red = lighten(red, 20.0);
        let bright_green = lighten(green, 20.0);
        let bright_yellow = lighten(yellow, 20.0);
        let bright_blue = lighten(blue, 20.0);
        let bright_magenta = lighten(magenta, 20.0);
        let bright_cyan = lighten(cyan, 20.0);
        let bright_white = white;

        AnsiColors {
            black,
            red,
            green,
            yellow,
            blue,
            magenta,
            cyan,
            white,
            bright_black,
            bright_red,
            bright_green,
            bright_yellow,
            bright_blue,
            bright_magenta,
            bright_cyan,
            bright_white,
        }
    }

    /// 256色パレットを生成
    /// 
    /// 256色対応ターミナル向けの詳細な色調整を提供します。
    /// 
    /// # Arguments
    /// 
    /// * `color_info` - 色情報
    /// 
    /// # Returns
    /// 
    /// 256色パレット
    fn generate_256_palette(&self, color_info: &ColorInfo) -> Vec<Rgb> {
        let mut palette = Vec::with_capacity(256);

        // 0-15: ANSI 16色（既に生成済みなので、ここでは簡易版を生成）
        for i in 0..16 {
            let lightness = (i as f64 / 15.0) * 100.0;
            let color = generate_color_from_hue(color_info.hue, 50.0, lightness);
            palette.push(color);
        }

        // 16-231: 6x6x6色立方体
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let r_val = if r == 0 { 0 } else { 55 + r * 40 };
                    let g_val = if g == 0 { 0 } else { 55 + g * 40 };
                    let b_val = if b == 0 { 0 } else { 55 + b * 40 };
                    palette.push(Rgb::new(r_val as u8, g_val as u8, b_val as u8));
                }
            }
        }

        // 232-255: グレースケール
        for i in 0..24 {
            let gray = 8 + i * 10;
            palette.push(Rgb::new(gray as u8, gray as u8, gray as u8));
        }

        palette
    }
}

/// True Colorサポートを検出
/// 
/// 環境変数をチェックして、ターミナルがTrue Color（24bit色）に対応しているかを判定します。
/// 
/// # Returns
/// 
/// True Colorに対応している場合はtrue
pub fn detect_true_color_support() -> bool {
    // COLORTERM環境変数をチェック
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        if colorterm == "truecolor" || colorterm == "24bit" {
            return true;
        }
    }

    // TERM環境変数をチェック
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("24bit") || term.contains("truecolor") {
            return true;
        }
    }

    false
}

/// 256色サポートを検出
/// 
/// 環境変数をチェックして、ターミナルが256色に対応しているかを判定します。
/// 
/// # Returns
/// 
/// 256色に対応している場合はtrue
pub fn supports_256_colors() -> bool {
    // True Colorをサポートしていれば256色もサポート
    if detect_true_color_support() {
        return true;
    }

    // TERM環境変数をチェック
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") || term.contains("256") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_color_info(is_dark: bool) -> ColorInfo {
        ColorInfo {
            dominant_colors: vec![Lab {
                l: if is_dark { 20.0 } else { 80.0 },
                a: 0.0,
                b: 0.0,
            }],
            average_lightness: if is_dark { 20.0 } else { 80.0 },
            saturation: 50.0,
            hue: 0.0,
            is_dark,
        }
    }

    #[test]
    fn test_scheme_generator_new() {
        let generator = SchemeGenerator::new(4.5);
        assert_eq!(generator.contrast_ratio, 4.5);
    }

    #[test]
    fn test_scheme_generator_default() {
        let generator = SchemeGenerator::default();
        assert_eq!(generator.contrast_ratio, 4.5);
    }

    #[test]
    fn test_generate_dark_background() {
        let generator = SchemeGenerator::default();
        let color_info = create_test_color_info(true);

        let scheme = generator.generate(&color_info).unwrap();

        // 暗い背景には明るいフォアグラウンド
        assert!(scheme.foreground.r > 200);
        assert!(scheme.foreground.g > 200);
        assert!(scheme.foreground.b > 200);

        // コントラスト比をチェック
        let bg_rgb = lab_to_rgb(scheme.background);
        let contrast = calculate_contrast_ratio(bg_rgb, scheme.foreground);
        assert!(
            contrast >= 4.5,
            "Contrast ratio {} should be >= 4.5",
            contrast
        );
    }

    #[test]
    fn test_generate_light_background() {
        let generator = SchemeGenerator::default();
        let color_info = create_test_color_info(false);

        let scheme = generator.generate(&color_info).unwrap();

        // 明るい背景には暗いフォアグラウンド
        assert!(scheme.foreground.r < 100);
        assert!(scheme.foreground.g < 100);
        assert!(scheme.foreground.b < 100);

        // コントラスト比をチェック
        let bg_rgb = lab_to_rgb(scheme.background);
        let contrast = calculate_contrast_ratio(bg_rgb, scheme.foreground);
        assert!(
            contrast >= 4.5,
            "Contrast ratio {} should be >= 4.5",
            contrast
        );
    }

    #[test]
    fn test_calculate_foreground_color_dark() {
        let generator = SchemeGenerator::default();
        let color_info = create_test_color_info(true);

        let fg = generator.calculate_foreground_color(&color_info).unwrap();

        // 暗い背景には明るいフォアグラウンド
        assert!(fg.r > 200);
        assert!(fg.g > 200);
        assert!(fg.b > 200);
    }

    #[test]
    fn test_calculate_foreground_color_light() {
        let generator = SchemeGenerator::default();
        let color_info = create_test_color_info(false);

        let fg = generator.calculate_foreground_color(&color_info).unwrap();

        // 明るい背景には暗いフォアグラウンド
        assert!(fg.r < 100);
        assert!(fg.g < 100);
        assert!(fg.b < 100);
    }

    #[test]
    fn test_adjust_for_contrast() {
        let generator = SchemeGenerator::default();
        let bg = Rgb::new(50, 50, 50); // 暗いグレー
        let fg = Rgb::new(70, 70, 70); // 少し明るいグレー（コントラスト不足）

        let adjusted = generator.adjust_for_contrast(bg, fg, 4.5).unwrap();

        // 調整後のコントラスト比をチェック
        let contrast = calculate_contrast_ratio(bg, adjusted);
        assert!(
            contrast >= 4.5,
            "Adjusted contrast ratio {} should be >= 4.5",
            contrast
        );
    }

    #[test]
    fn test_generate_ansi_colors() {
        let generator = SchemeGenerator::default();
        let color_info = create_test_color_info(true);
        let fg = Rgb::new(240, 240, 240);
        let bg = Rgb::new(20, 20, 20);

        let ansi = generator.generate_ansi_colors(&color_info, fg, bg);

        // 16色すべてが生成されていることを確認
        assert!(ansi.black.r <= ansi.white.r);
        assert!(ansi.bright_black.r > ansi.black.r);
    }

    #[test]
    fn test_generate_256_palette() {
        let generator = SchemeGenerator::default();
        let color_info = create_test_color_info(true);

        let palette = generator.generate_256_palette(&color_info);

        // 256色が生成されていることを確認
        assert_eq!(palette.len(), 256);
    }

    #[test]
    fn test_detect_true_color_support() {
        // 環境変数に依存するため、実際の値をテストするのは難しい
        // 関数が呼び出せることだけを確認
        let _ = detect_true_color_support();
    }

    #[test]
    fn test_supports_256_colors() {
        // 環境変数に依存するため、実際の値をテストするのは難しい
        // 関数が呼び出せることだけを確認
        let _ = supports_256_colors();
    }
}
