// 画像解析

use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageReader};
use std::path::Path;
use crate::models::{ColorInfo, Lab, Rgb, TwfError};
use crate::utils::color_space::{rgb_to_lab, calculate_saturation, calculate_hue};
use rand::seq::SliceRandom;

pub struct ImageAnalyzer {
    sample_size: usize,
}

impl ImageAnalyzer {
    /// 新しいImageAnalyzerインスタンスを作成
    pub fn new(sample_size: usize) -> Self {
        Self { sample_size }
    }
    
    /// 画像を解析して色情報を抽出
    /// 
    /// 設計書のP-07に従って以下の処理を実行します：
    /// 1. 画像を読み込み
    /// 2. リサイズ（800x600、パフォーマンス最適化）
    /// 3. ピクセルをサンプリング（グリッドサンプリング、最大10000ピクセル）
    /// 4. RGB → Lab変換
    /// 5. 主要色の抽出（K-meansは次のタスクで実装）
    /// 6. 明度、彩度、色相を計算
    pub async fn analyze(&self, image_path: &Path) -> Result<ColorInfo> {
        // 1. 画像を読み込み
        let img = ImageReader::open(image_path)
            .map_err(|e| TwfError::ImageLoadError(format!("{}: {}", image_path.display(), e)))?
            .decode()
            .map_err(|e| TwfError::ImageLoadError(format!("デコードエラー: {}", e)))?;
        
        // 2. リサイズ（パフォーマンス最適化のため800x600に）
        let img = img.resize(800, 600, image::imageops::FilterType::Lanczos3);
        
        // 3. ピクセルをサンプリング
        let pixels = self.sample_pixels(&img);
        
        // 4. RGB → Lab変換
        let lab_colors: Vec<Lab> = pixels.iter()
            .map(|rgb| rgb_to_lab(*rgb))
            .collect();
        
        // 5. K-meansクラスタリングで主要色を抽出（k=5）
        let dominant_colors = if lab_colors.len() > 5 {
            self.kmeans_clustering(&lab_colors, 5, 100)
        } else {
            // サンプル数が少ない場合はそのまま使用
            lab_colors.clone()
        };
        
        // 6. 平均明度を計算
        let average_lightness = if !lab_colors.is_empty() {
            lab_colors.iter().map(|c| c.l).sum::<f64>() / lab_colors.len() as f64
        } else {
            50.0 // デフォルト値
        };
        
        // 7. 彩度と色相を計算
        let saturation = calculate_saturation(&lab_colors);
        let hue = calculate_hue(&lab_colors);
        
        // 8. 暗い背景かどうかを判定
        let is_dark = average_lightness < 50.0;
        
        Ok(ColorInfo {
            dominant_colors,
            average_lightness,
            saturation,
            hue,
            is_dark,
        })
    }
    
    /// ピクセルをサンプリング（グリッドサンプリング）
    /// 
    /// 全ピクセルを処理するとパフォーマンスが低下するため、
    /// グリッドサンプリングを使用して処理するピクセル数を制限します。
    /// 
    /// # 引数
    /// * `img` - サンプリング対象の画像
    /// 
    /// # 戻り値
    /// サンプリングされたRGB色のベクター
    fn sample_pixels(&self, img: &DynamicImage) -> Vec<Rgb> {
        let (width, height) = img.dimensions();
        let total_pixels = (width * height) as usize;
        
        // 全ピクセル数がサンプルサイズ以下の場合は全ピクセルを使用
        if total_pixels <= self.sample_size {
            return self.get_all_pixels(img);
        }
        
        // グリッドサンプリング
        // サンプルサイズから適切なステップサイズを計算
        let step = ((total_pixels as f64 / self.sample_size as f64).sqrt().ceil()) as u32;
        let mut samples = Vec::new();
        
        for y in (0..height).step_by(step as usize) {
            for x in (0..width).step_by(step as usize) {
                let pixel = img.get_pixel(x, y);
                samples.push(Rgb::new(pixel[0], pixel[1], pixel[2]));
            }
        }
        
        samples
    }
    
    /// 全ピクセルを取得
    /// 
    /// 画像のすべてのピクセルをRGB色として取得します。
    /// 
    /// # 引数
    /// * `img` - 対象の画像
    /// 
    /// # 戻り値
    /// すべてのピクセルのRGB色のベクター
    fn get_all_pixels(&self, img: &DynamicImage) -> Vec<Rgb> {
        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                pixels.push(Rgb::new(pixel[0], pixel[1], pixel[2]));
            }
        }
        
        pixels
    }
    
    /// K-meansクラスタリングで主要色を抽出
    /// 
    /// 設計書のP-07に従って、K-meansアルゴリズムを実装します：
    /// 1. ランダムに初期中心を選択
    /// 2. 各色を最も近い中心に割り当て
    /// 3. 新しい中心を計算
    /// 4. 収束判定（最大100回の反復）
    /// 5. 最終的な中心（主要色）を返す
    /// 
    /// # 引数
    /// * `colors` - クラスタリング対象の色（Lab色空間）
    /// * `k` - クラスタ数（主要色の数）
    /// * `max_iterations` - 最大反復回数
    /// 
    /// # 戻り値
    /// k個の主要色（Lab色空間）
    fn kmeans_clustering(&self, colors: &[Lab], k: usize, max_iterations: usize) -> Vec<Lab> {
        if colors.is_empty() || k == 0 {
            return Vec::new();
        }
        
        if colors.len() <= k {
            return colors.to_vec();
        }
        
        // 1. ランダムに初期中心を選択
        let mut rng = rand::thread_rng();
        let mut centroids: Vec<Lab> = colors
            .choose_multiple(&mut rng, k)
            .copied()
            .collect();
        
        for _ in 0..max_iterations {
            // 2. 各色を最も近い中心に割り当て
            let clusters = self.assign_to_nearest_centroid(colors, &centroids);
            
            // 3. 新しい中心を計算
            let new_centroids = self.calculate_new_centroids(&clusters);
            
            // 4. 収束判定
            if self.centroids_converged(&centroids, &new_centroids) {
                break;
            }
            
            centroids = new_centroids;
        }
        
        centroids
    }
    
    /// 各色を最も近い中心に割り当て
    /// 
    /// # 引数
    /// * `colors` - 割り当て対象の色
    /// * `centroids` - 現在の中心
    /// 
    /// # 戻り値
    /// クラスタごとの色のベクター
    fn assign_to_nearest_centroid(&self, colors: &[Lab], centroids: &[Lab]) -> Vec<Vec<Lab>> {
        let mut clusters: Vec<Vec<Lab>> = vec![Vec::new(); centroids.len()];
        
        for color in colors {
            // 最も近い中心を見つける
            let nearest_idx = centroids
                .iter()
                .enumerate()
                .min_by(|(_, c1), (_, c2)| {
                    let dist1 = self.color_distance(color, c1);
                    let dist2 = self.color_distance(color, c2);
                    dist1.partial_cmp(&dist2).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            
            clusters[nearest_idx].push(*color);
        }
        
        clusters
    }
    
    /// 新しい中心を計算
    /// 
    /// 各クラスタの平均を新しい中心とします。
    /// 
    /// # 引数
    /// * `clusters` - クラスタごとの色のベクター
    /// 
    /// # 戻り値
    /// 新しい中心のベクター
    fn calculate_new_centroids(&self, clusters: &[Vec<Lab>]) -> Vec<Lab> {
        clusters
            .iter()
            .map(|cluster| {
                if cluster.is_empty() {
                    // 空のクラスタの場合はデフォルト値を返す
                    Lab { l: 50.0, a: 0.0, b: 0.0 }
                } else {
                    // クラスタ内の色の平均を計算
                    let sum_l: f64 = cluster.iter().map(|c| c.l).sum();
                    let sum_a: f64 = cluster.iter().map(|c| c.a).sum();
                    let sum_b: f64 = cluster.iter().map(|c| c.b).sum();
                    let count = cluster.len() as f64;
                    
                    Lab {
                        l: sum_l / count,
                        a: sum_a / count,
                        b: sum_b / count,
                    }
                }
            })
            .collect()
    }
    
    /// 中心が収束したかどうかを判定
    /// 
    /// すべての中心の移動距離が閾値以下の場合、収束したと判定します。
    /// 
    /// # 引数
    /// * `old_centroids` - 古い中心
    /// * `new_centroids` - 新しい中心
    /// 
    /// # 戻り値
    /// 収束した場合はtrue
    fn centroids_converged(&self, old_centroids: &[Lab], new_centroids: &[Lab]) -> bool {
        const CONVERGENCE_THRESHOLD: f64 = 0.1;
        
        if old_centroids.len() != new_centroids.len() {
            return false;
        }
        
        old_centroids
            .iter()
            .zip(new_centroids.iter())
            .all(|(old, new)| self.color_distance(old, new) < CONVERGENCE_THRESHOLD)
    }
    
    /// Lab色空間での色の距離を計算（ユークリッド距離）
    /// 
    /// # 引数
    /// * `c1` - 色1
    /// * `c2` - 色2
    /// 
    /// # 戻り値
    /// 2色間の距離
    fn color_distance(&self, c1: &Lab, c2: &Lab) -> f64 {
        let dl = c1.l - c2.l;
        let da = c1.a - c2.a;
        let db = c1.b - c2.b;
        (dl * dl + da * da + db * db).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbImage, Rgb as ImageRgb};
    
    /// テスト用の単色画像を作成
    fn create_test_image(width: u32, height: u32, color: Rgb) -> DynamicImage {
        let mut img = RgbImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = ImageRgb([color.r, color.g, color.b]);
        }
        DynamicImage::ImageRgb8(img)
    }
    
    #[test]
    fn test_sample_pixels_small_image() {
        let analyzer = ImageAnalyzer::new(10000);
        let img = create_test_image(10, 10, Rgb::new(255, 0, 0));
        
        let samples = analyzer.sample_pixels(&img);
        
        // 小さい画像（100ピクセル）は全ピクセルがサンプリングされるはず
        assert_eq!(samples.len(), 100);
        
        // すべて赤色のはず
        for sample in samples {
            assert_eq!(sample.r, 255);
            assert_eq!(sample.g, 0);
            assert_eq!(sample.b, 0);
        }
    }
    
    #[test]
    fn test_sample_pixels_large_image() {
        let analyzer = ImageAnalyzer::new(1000);
        let img = create_test_image(200, 200, Rgb::new(0, 255, 0));
        
        let samples = analyzer.sample_pixels(&img);
        
        // 大きい画像（40000ピクセル）はサンプリングされるはず
        assert!(samples.len() <= 1000);
        assert!(samples.len() > 0);
        
        // すべて緑色のはず
        for sample in samples {
            assert_eq!(sample.r, 0);
            assert_eq!(sample.g, 255);
            assert_eq!(sample.b, 0);
        }
    }
    
    #[test]
    fn test_get_all_pixels() {
        let analyzer = ImageAnalyzer::new(10000);
        let img = create_test_image(5, 5, Rgb::new(0, 0, 255));
        
        let pixels = analyzer.get_all_pixels(&img);
        
        // 5x5 = 25ピクセル
        assert_eq!(pixels.len(), 25);
        
        // すべて青色のはず
        for pixel in pixels {
            assert_eq!(pixel.r, 0);
            assert_eq!(pixel.g, 0);
            assert_eq!(pixel.b, 255);
        }
    }
    
    #[tokio::test]
    async fn test_analyze_uniform_color() {
        // 一時ファイルに画像を保存
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("test.png");
        
        let img = create_test_image(100, 100, Rgb::new(128, 128, 128));
        img.save(&image_path).unwrap();
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // グレー（128, 128, 128）の明度は約53程度
        assert!(color_info.average_lightness > 40.0 && color_info.average_lightness < 60.0);
        
        // グレーは彩度が低いはず
        assert!(color_info.saturation < 20.0);
        
        // 明度が50以上なので暗くない
        assert!(!color_info.is_dark);
    }
    
    #[tokio::test]
    async fn test_analyze_dark_image() {
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("dark.png");
        
        // 暗い色の画像
        let img = create_test_image(100, 100, Rgb::new(30, 30, 30));
        img.save(&image_path).unwrap();
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // 暗い色なので明度が低いはず
        assert!(color_info.average_lightness < 50.0);
        
        // is_darkがtrueのはず
        assert!(color_info.is_dark);
    }
    
    #[tokio::test]
    async fn test_analyze_bright_image() {
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("bright.png");
        
        // 明るい色の画像
        let img = create_test_image(100, 100, Rgb::new(220, 220, 220));
        img.save(&image_path).unwrap();
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // 明るい色なので明度が高いはず
        assert!(color_info.average_lightness > 50.0);
        
        // is_darkがfalseのはず
        assert!(!color_info.is_dark);
    }
    
    #[tokio::test]
    async fn test_analyze_saturated_color() {
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("red.png");
        
        // 鮮やかな赤の画像
        let img = create_test_image(100, 100, Rgb::new(255, 0, 0));
        img.save(&image_path).unwrap();
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // 鮮やかな色なので彩度が高いはず
        assert!(color_info.saturation > 50.0);
    }
    
    #[tokio::test]
    async fn test_analyze_nonexistent_file() {
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(Path::new("/nonexistent/image.png")).await;
        
        // ファイルが存在しないのでエラーになるはず
        assert!(result.is_err());
    }
    
    #[test]
    fn test_kmeans_clustering_basic() {
        let analyzer = ImageAnalyzer::new(10000);
        
        // 3つの明確に異なる色を用意
        let colors = vec![
            Lab { l: 20.0, a: 0.0, b: 0.0 },  // 暗い色
            Lab { l: 20.0, a: 0.0, b: 0.0 },
            Lab { l: 20.0, a: 0.0, b: 0.0 },
            Lab { l: 50.0, a: 0.0, b: 0.0 },  // 中間の色
            Lab { l: 50.0, a: 0.0, b: 0.0 },
            Lab { l: 50.0, a: 0.0, b: 0.0 },
            Lab { l: 80.0, a: 0.0, b: 0.0 },  // 明るい色
            Lab { l: 80.0, a: 0.0, b: 0.0 },
            Lab { l: 80.0, a: 0.0, b: 0.0 },
        ];
        
        // k=3でクラスタリング
        let centroids = analyzer.kmeans_clustering(&colors, 3, 100);
        
        // 3つの中心が返されるはず
        assert_eq!(centroids.len(), 3);
        
        // 中心の明度が異なるはず（順序は不定）
        let mut lightnesses: Vec<f64> = centroids.iter().map(|c| c.l).collect();
        lightnesses.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // 暗い、中間、明るいの3つのグループに分かれているはず
        assert!(lightnesses[0] < 30.0);  // 暗い
        assert!(lightnesses[1] > 40.0 && lightnesses[1] < 60.0);  // 中間
        assert!(lightnesses[2] > 70.0);  // 明るい
    }
    
    #[test]
    fn test_kmeans_clustering_empty() {
        let analyzer = ImageAnalyzer::new(10000);
        let colors: Vec<Lab> = vec![];
        
        let centroids = analyzer.kmeans_clustering(&colors, 3, 100);
        
        // 空の入力には空の結果
        assert_eq!(centroids.len(), 0);
    }
    
    #[test]
    fn test_kmeans_clustering_fewer_colors_than_k() {
        let analyzer = ImageAnalyzer::new(10000);
        let colors = vec![
            Lab { l: 50.0, a: 0.0, b: 0.0 },
            Lab { l: 60.0, a: 0.0, b: 0.0 },
        ];
        
        // k=5だが色は2つしかない
        let centroids = analyzer.kmeans_clustering(&colors, 5, 100);
        
        // 入力の色がそのまま返されるはず
        assert_eq!(centroids.len(), 2);
    }
    
    #[test]
    fn test_color_distance() {
        let analyzer = ImageAnalyzer::new(10000);
        
        let c1 = Lab { l: 50.0, a: 0.0, b: 0.0 };
        let c2 = Lab { l: 50.0, a: 0.0, b: 0.0 };
        
        // 同じ色の距離は0
        let dist = analyzer.color_distance(&c1, &c2);
        assert!(dist < 0.001);
        
        let c3 = Lab { l: 60.0, a: 0.0, b: 0.0 };
        
        // 異なる色の距離は0より大きい
        let dist2 = analyzer.color_distance(&c1, &c3);
        assert!(dist2 > 0.0);
        assert!((dist2 - 10.0).abs() < 0.001);  // 明度の差が10なので距離も約10
    }
    
    #[test]
    fn test_centroids_converged() {
        let analyzer = ImageAnalyzer::new(10000);
        
        let centroids1 = vec![
            Lab { l: 50.0, a: 0.0, b: 0.0 },
            Lab { l: 60.0, a: 0.0, b: 0.0 },
        ];
        
        let centroids2 = vec![
            Lab { l: 50.05, a: 0.0, b: 0.0 },  // わずかな変化
            Lab { l: 60.05, a: 0.0, b: 0.0 },
        ];
        
        // わずかな変化なので収束したと判定されるはず
        assert!(analyzer.centroids_converged(&centroids1, &centroids2));
        
        let centroids3 = vec![
            Lab { l: 55.0, a: 0.0, b: 0.0 },  // 大きな変化
            Lab { l: 65.0, a: 0.0, b: 0.0 },
        ];
        
        // 大きな変化なので収束していないと判定されるはず
        assert!(!analyzer.centroids_converged(&centroids1, &centroids3));
    }
    
    #[tokio::test]
    async fn test_analyze_with_kmeans() {
        // K-meansクラスタリングが実際に使用されることを確認
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("multicolor.png");
        
        // 複数の色を含む画像を作成
        let mut img = RgbImage::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                let color = if x < 33 {
                    ImageRgb([255, 0, 0])  // 赤
                } else if x < 66 {
                    ImageRgb([0, 255, 0])  // 緑
                } else {
                    ImageRgb([0, 0, 255])  // 青
                };
                img.put_pixel(x, y, color);
            }
        }
        DynamicImage::ImageRgb8(img).save(&image_path).unwrap();
        
        let analyzer = ImageAnalyzer::new(10000);
        let result = analyzer.analyze(&image_path).await;
        
        assert!(result.is_ok());
        let color_info = result.unwrap();
        
        // K-meansで5つの主要色が抽出されるはず
        assert_eq!(color_info.dominant_colors.len(), 5);
        
        // 彩度が高いはず（鮮やかな色を含むため）
        assert!(color_info.saturation > 30.0);
    }
}
