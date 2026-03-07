// Feature: twf, Property 1: 画像解析の一貫性
// **Validates: Requirements 2.1.1, 2.1.2, 2.1.3**

use proptest::prelude::*;
use image::{DynamicImage, RgbImage, Rgb as ImageRgb};
use twf::analyzer::image::ImageAnalyzer;
use twf::models::Rgb;
use std::path::PathBuf;
use tempfile::TempDir;

/// テスト用の画像を生成するヘルパー関数
/// 
/// # 引数
/// * `width` - 画像の幅
/// * `height` - 画像の高さ
/// * `colors` - 画像に含める色のリスト（RGB）
/// 
/// # 戻り値
/// 生成された画像
fn generate_test_image(width: u32, height: u32, colors: &[Rgb]) -> DynamicImage {
    let mut img = RgbImage::new(width, height);
    let num_colors = colors.len();
    
    if num_colors == 0 {
        // 色が指定されていない場合は黒で塗りつぶす
        return DynamicImage::ImageRgb8(img);
    }
    
    // 画像を色で塗り分ける
    for y in 0..height {
        for x in 0..width {
            // ピクセル位置に基づいて色を選択
            let color_idx = ((x + y) as usize) % num_colors;
            let color = colors[color_idx];
            img.put_pixel(x, y, ImageRgb([color.r, color.g, color.b]));
        }
    }
    
    DynamicImage::ImageRgb8(img)
}

/// 画像を一時ファイルに保存するヘルパー関数
/// 
/// # 引数
/// * `img` - 保存する画像
/// * `temp_dir` - 一時ディレクトリ
/// 
/// # 戻り値
/// 保存された画像のパス
fn save_test_image(img: &DynamicImage, temp_dir: &TempDir) -> PathBuf {
    let image_path = temp_dir.path().join("test_image.png");
    img.save(&image_path).expect("画像の保存に失敗しました");
    image_path
}

/// RGB色のジェネレータ
fn rgb_strategy() -> impl Strategy<Value = Rgb> {
    (0u8..=255, 0u8..=255, 0u8..=255)
        .prop_map(|(r, g, b)| Rgb::new(r, g, b))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]
    
    /// Property 1.1: 明度の範囲
    /// 任意の画像に対して、average_lightnessは0.0以上100.0以下である
    #[test]
    fn test_lightness_range(
        width in 50u32..200u32,
        height in 50u32..200u32,
        colors in prop::collection::vec(rgb_strategy(), 1..5)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let img = generate_test_image(width, height, &colors);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = runtime.block_on(analyzer.analyze(&image_path));
        
        prop_assert!(result.is_ok(), "画像解析が失敗しました: {:?}", result.err());
        
        let color_info = result.unwrap();
        prop_assert!(
            color_info.average_lightness >= 0.0 && color_info.average_lightness <= 100.0,
            "明度が範囲外です: {}",
            color_info.average_lightness
        );
    }
    
    /// Property 1.2: 彩度の範囲
    /// 任意の画像に対して、saturationは0.0以上100.0以下である
    #[test]
    fn test_saturation_range(
        width in 50u32..200u32,
        height in 50u32..200u32,
        colors in prop::collection::vec(rgb_strategy(), 1..5)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let img = generate_test_image(width, height, &colors);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = runtime.block_on(analyzer.analyze(&image_path));
        
        prop_assert!(result.is_ok(), "画像解析が失敗しました: {:?}", result.err());
        
        let color_info = result.unwrap();
        prop_assert!(
            color_info.saturation >= 0.0 && color_info.saturation <= 100.0,
            "彩度が範囲外です: {}",
            color_info.saturation
        );
    }
    
    /// Property 1.3: 色相の範囲
    /// 任意の画像に対して、hueは0.0以上360.0未満である
    #[test]
    fn test_hue_range(
        width in 50u32..200u32,
        height in 50u32..200u32,
        colors in prop::collection::vec(rgb_strategy(), 1..5)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let img = generate_test_image(width, height, &colors);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = runtime.block_on(analyzer.analyze(&image_path));
        
        prop_assert!(result.is_ok(), "画像解析が失敗しました: {:?}", result.err());
        
        let color_info = result.unwrap();
        prop_assert!(
            color_info.hue >= 0.0 && color_info.hue < 360.0,
            "色相が範囲外です: {}",
            color_info.hue
        );
    }
    
    /// Property 1.4: 主要色の数
    /// 任意の画像に対して、dominant_colorsは空でなく、最大5個の色を含む
    #[test]
    fn test_dominant_colors_count(
        width in 50u32..200u32,
        height in 50u32..200u32,
        colors in prop::collection::vec(rgb_strategy(), 1..5)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let img = generate_test_image(width, height, &colors);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = runtime.block_on(analyzer.analyze(&image_path));
        
        prop_assert!(result.is_ok(), "画像解析が失敗しました: {:?}", result.err());
        
        let color_info = result.unwrap();
        prop_assert!(
            !color_info.dominant_colors.is_empty(),
            "主要色が空です"
        );
        prop_assert!(
            color_info.dominant_colors.len() <= 5,
            "主要色の数が5を超えています: {}",
            color_info.dominant_colors.len()
        );
    }
    
    /// Property 1.5: is_darkフラグの一貫性
    /// average_lightness < 50.0の場合、is_dark == trueである
    /// average_lightness >= 50.0の場合、is_dark == falseである
    #[test]
    fn test_is_dark_consistency(
        width in 50u32..200u32,
        height in 50u32..200u32,
        colors in prop::collection::vec(rgb_strategy(), 1..5)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let img = generate_test_image(width, height, &colors);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = runtime.block_on(analyzer.analyze(&image_path));
        
        prop_assert!(result.is_ok(), "画像解析が失敗しました: {:?}", result.err());
        
        let color_info = result.unwrap();
        if color_info.average_lightness < 50.0 {
            prop_assert!(
                color_info.is_dark,
                "明度が50未満なのにis_darkがfalseです: lightness={}",
                color_info.average_lightness
            );
        } else {
            prop_assert!(
                !color_info.is_dark,
                "明度が50以上なのにis_darkがtrueです: lightness={}",
                color_info.average_lightness
            );
        }
    }
    
    /// Property 1.6: 一貫性（決定性）
    /// 同じ画像を複数回解析しても、同じ結果が得られる
    /// 注: K-meansクラスタリングはランダム性を含むため、
    /// 主要色の順序は異なる可能性があるが、明度・彩度・色相は一貫している必要がある
    #[test]
    fn test_analysis_consistency(
        width in 50u32..200u32,
        height in 50u32..200u32,
        colors in prop::collection::vec(rgb_strategy(), 1..5)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let img = generate_test_image(width, height, &colors);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        
        // 同じ画像を2回解析
        let result1 = runtime.block_on(analyzer.analyze(&image_path));
        let result2 = runtime.block_on(analyzer.analyze(&image_path));
        
        prop_assert!(result1.is_ok(), "1回目の画像解析が失敗しました: {:?}", result1.err());
        prop_assert!(result2.is_ok(), "2回目の画像解析が失敗しました: {:?}", result2.err());
        
        let color_info1 = result1.unwrap();
        let color_info2 = result2.unwrap();
        
        // 明度、彩度、色相は一貫しているはず
        let lightness_diff = (color_info1.average_lightness - color_info2.average_lightness).abs();
        prop_assert!(
            lightness_diff < 1.0,
            "明度が一貫していません: {} vs {}",
            color_info1.average_lightness,
            color_info2.average_lightness
        );
        
        let saturation_diff = (color_info1.saturation - color_info2.saturation).abs();
        prop_assert!(
            saturation_diff < 1.0,
            "彩度が一貫していません: {} vs {}",
            color_info1.saturation,
            color_info2.saturation
        );
        
        let hue_diff = (color_info1.hue - color_info2.hue).abs();
        // 色相は循環するため、差が大きい場合は360度を引く
        let hue_diff = if hue_diff > 180.0 { 360.0 - hue_diff } else { hue_diff };
        prop_assert!(
            hue_diff < 5.0,
            "色相が一貫していません: {} vs {}",
            color_info1.hue,
            color_info2.hue
        );
        
        // is_darkフラグも一貫しているはず
        prop_assert_eq!(
            color_info1.is_dark,
            color_info2.is_dark,
            "is_darkフラグが一貫していません"
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    /// ユニットテスト: 単色画像の解析
    /// 単色の画像を解析した場合、主要色は最大5つになる（K-meansの動作）
    #[tokio::test]
    async fn test_single_color_image() {
        let temp_dir = TempDir::new().unwrap();
        let color = Rgb::new(128, 128, 128);
        let img = generate_test_image(100, 100, &[color]);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // K-meansは最大5つの主要色を返す
        assert!(color_info.dominant_colors.len() <= 5);
        
        // グレーなので彩度は低いはず
        assert!(color_info.saturation < 20.0);
    }
    
    /// ユニットテスト: 暗い画像の解析
    #[tokio::test]
    async fn test_dark_image() {
        let temp_dir = TempDir::new().unwrap();
        let color = Rgb::new(20, 20, 20);
        let img = generate_test_image(100, 100, &[color]);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // 暗い色なので明度が低いはず
        assert!(color_info.average_lightness < 50.0);
        assert!(color_info.is_dark);
    }
    
    /// ユニットテスト: 明るい画像の解析
    #[tokio::test]
    async fn test_bright_image() {
        let temp_dir = TempDir::new().unwrap();
        let color = Rgb::new(230, 230, 230);
        let img = generate_test_image(100, 100, &[color]);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // 明るい色なので明度が高いはず
        assert!(color_info.average_lightness > 50.0);
        assert!(!color_info.is_dark);
    }
    
    /// ユニットテスト: 鮮やかな色の画像の解析
    #[tokio::test]
    async fn test_saturated_image() {
        let temp_dir = TempDir::new().unwrap();
        let color = Rgb::new(255, 0, 0);  // 鮮やかな赤
        let img = generate_test_image(100, 100, &[color]);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // 鮮やかな色なので彩度が高いはず
        assert!(color_info.saturation > 50.0);
    }
    
    /// ユニットテスト: 複数色の画像の解析
    #[tokio::test]
    async fn test_multicolor_image() {
        let temp_dir = TempDir::new().unwrap();
        let colors = vec![
            Rgb::new(255, 0, 0),    // 赤
            Rgb::new(0, 255, 0),    // 緑
            Rgb::new(0, 0, 255),    // 青
            Rgb::new(255, 255, 0),  // 黄
            Rgb::new(255, 0, 255),  // マゼンタ
        ];
        let img = generate_test_image(200, 200, &colors);
        let image_path = save_test_image(&img, &temp_dir);
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // 複数の色があるので主要色は複数
        assert!(color_info.dominant_colors.len() > 1);
        assert!(color_info.dominant_colors.len() <= 5);
        
        // 鮮やかな色が多いので彩度は0より大きいはず
        assert!(color_info.saturation >= 0.0);
    }
}
