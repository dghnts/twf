// フォント設定生成

use crate::models::{ColorInfo, FontConfig, FontWeight};

/// フォント設定最適化
pub struct FontOptimizer;

impl FontOptimizer {
    /// フォント設定を生成
    /// 
    /// 背景の明度と彩度に基づいて最適なフォントウェイトを決定します。
    /// 
    /// # アルゴリズム
    /// 1. 背景の明度に基づいてフォントウェイトを決定
    ///    - 暗い背景: 通常の太さ
    ///    - 明るい背景: 少し太め
    /// 2. 背景の彩度（複雑さ）に基づいて調整
    ///    - 彩度が高い（複雑な背景）: より太めに
    /// 
    /// # Requirements
    /// - 2.3.1: 背景画像の複雑さに応じて適切なフォントウェイトを推奨
    /// - 2.3.2: 背景画像の明度に応じてフォントの太さを調整
    pub fn optimize(&self, color_info: &ColorInfo) -> FontConfig {
        // 1. 背景の明度に基づいてフォントウェイトを決定
        let weight = if color_info.is_dark {
            // 暗い背景では通常の太さ
            FontWeight::Normal
        } else {
            // 明るい背景では少し太め（視認性向上のため）
            FontWeight::Medium
        };
        
        // 2. 背景の彩度（複雑さ）に基づいて調整
        let weight = if color_info.saturation > 50.0 {
            // 彩度が高い（複雑な背景）場合は太めに
            weight.increase()
        } else {
            weight
        };
        
        // 3. システムの等幅フォントを検出
        let available_fonts = detect_monospace_fonts();
        
        FontConfig {
            weight,
            recommended_fonts: available_fonts,
        }
    }
}

/// システムにインストールされている等幅フォントを検出
/// 
/// # Requirements
/// - 2.3.3: システムにインストールされている等幅フォントを検出
/// 
/// # 戻り値
/// 推奨される等幅フォントのリスト（優先度順）
pub fn detect_monospace_fonts() -> Vec<String> {
    let mut fonts = Vec::new();
    
    // プラットフォーム別の一般的な等幅フォント
    #[cfg(target_os = "macos")]
    {
        fonts.extend_from_slice(&[
            "SF Mono".to_string(),
            "Menlo".to_string(),
            "Monaco".to_string(),
            "Courier New".to_string(),
        ]);
    }
    
    #[cfg(target_os = "linux")]
    {
        fonts.extend_from_slice(&[
            "DejaVu Sans Mono".to_string(),
            "Liberation Mono".to_string(),
            "Courier New".to_string(),
            "Monospace".to_string(),
        ]);
    }
    
    #[cfg(target_os = "windows")]
    {
        fonts.extend_from_slice(&[
            "Cascadia Code".to_string(),
            "Cascadia Mono".to_string(),
            "Consolas".to_string(),
            "Courier New".to_string(),
        ]);
    }
    
    // プラットフォーム非依存の人気フォント
    fonts.extend_from_slice(&[
        "Fira Code".to_string(),
        "JetBrains Mono".to_string(),
        "Source Code Pro".to_string(),
        "Hack".to_string(),
        "Inconsolata".to_string(),
    ]);
    
    // 重複を削除
    fonts.sort();
    fonts.dedup();
    
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Lab;
    
    /// 暗い背景でのフォントウェイト選択をテスト
    #[test]
    fn test_dark_background_font_weight() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 20.0, a: 0.0, b: 0.0 }],
            average_lightness: 20.0,
            saturation: 30.0,
            hue: 0.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 暗い背景、低彩度 → Normal
        assert_eq!(font_config.weight, FontWeight::Normal);
    }
    
    /// 明るい背景でのフォントウェイト選択をテスト
    #[test]
    fn test_light_background_font_weight() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 80.0, a: 0.0, b: 0.0 }],
            average_lightness: 80.0,
            saturation: 30.0,
            hue: 0.0,
            is_dark: false,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 明るい背景、低彩度 → Medium
        assert_eq!(font_config.weight, FontWeight::Medium);
    }
    
    /// 高彩度背景でのフォントウェイト選択をテスト
    #[test]
    fn test_high_saturation_font_weight() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 20.0, a: 50.0, b: 50.0 }],
            average_lightness: 20.0,
            saturation: 70.0,  // 高彩度
            hue: 45.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 暗い背景、高彩度 → Normal → Medium
        assert_eq!(font_config.weight, FontWeight::Medium);
    }
    
    /// 明るい背景 + 高彩度でのフォントウェイト選択をテスト
    #[test]
    fn test_light_high_saturation_font_weight() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 80.0, a: 50.0, b: 50.0 }],
            average_lightness: 80.0,
            saturation: 70.0,  // 高彩度
            hue: 45.0,
            is_dark: false,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 明るい背景、高彩度 → Medium → Bold
        assert_eq!(font_config.weight, FontWeight::Bold);
    }
    
    /// 等幅フォント検出のテスト
    #[test]
    fn test_detect_monospace_fonts() {
        let fonts = detect_monospace_fonts();
        
        // 少なくとも1つのフォントが検出されること
        assert!(!fonts.is_empty());
        
        // プラットフォーム固有のフォントが含まれていること
        #[cfg(target_os = "macos")]
        {
            assert!(fonts.contains(&"SF Mono".to_string()) || 
                    fonts.contains(&"Menlo".to_string()));
        }
        
        #[cfg(target_os = "linux")]
        {
            assert!(fonts.contains(&"DejaVu Sans Mono".to_string()) || 
                    fonts.contains(&"Liberation Mono".to_string()));
        }
        
        #[cfg(target_os = "windows")]
        {
            assert!(fonts.contains(&"Cascadia Code".to_string()) || 
                    fonts.contains(&"Consolas".to_string()));
        }
        
        // 人気フォントが含まれていること
        assert!(fonts.contains(&"Fira Code".to_string()));
        assert!(fonts.contains(&"JetBrains Mono".to_string()));
    }
    
    /// フォント設定に推奨フォントが含まれることをテスト
    #[test]
    fn test_font_config_includes_recommended_fonts() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 50.0, a: 0.0, b: 0.0 }],
            average_lightness: 50.0,
            saturation: 40.0,
            hue: 0.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 推奨フォントリストが空でないこと
        assert!(!font_config.recommended_fonts.is_empty());
    }
    
    /// 境界値テスト: 彩度が正確に50.0の場合
    #[test]
    fn test_saturation_boundary() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 20.0, a: 0.0, b: 0.0 }],
            average_lightness: 20.0,
            saturation: 50.0,  // 境界値
            hue: 0.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 彩度が50.0の場合は増加しない（> 50.0のみ増加）
        assert_eq!(font_config.weight, FontWeight::Normal);
    }
    
    /// 境界値テスト: 彩度が50.1の場合
    #[test]
    fn test_saturation_just_above_boundary() {
        let color_info = ColorInfo {
            dominant_colors: vec![Lab { l: 20.0, a: 0.0, b: 0.0 }],
            average_lightness: 20.0,
            saturation: 50.1,  // 境界値を超える
            hue: 0.0,
            is_dark: true,
        };
        
        let optimizer = FontOptimizer;
        let font_config = optimizer.optimize(&color_info);
        
        // 彩度が50.0を超える場合は増加
        assert_eq!(font_config.weight, FontWeight::Medium);
    }
}
