# 実装計画: TWF（Terminal Wallpaper Fit）

## 概要

TWFは、ターミナルの背景画像または背景色を解析し、視認性の高いカラースキームとフォント設定を自動生成するRust製CLIツールです。本実装計画では、requirements.mdとdesign.mdに基づいて、段階的に機能を実装していきます。

実装は以下の順序で進めます：
1. プロジェクト構造とコアデータ型の定義
2. ユーティリティ機能（色空間変換、コントラスト計算）
3. 画像解析と色解析
4. 背景画像/背景色の検出機能
5. カラースキームとフォント設定の生成
6. 設定適用とバックアップ機能
7. CLIインターフェースとプレビュー機能
8. 統合とエラーハンドリング

## タスク

- [ ] 1. プロジェクト構造とコアデータ型の設定
  - [x] 1.1 Cargoプロジェクトの初期化とディレクトリ構造の作成
    - `src/`配下にモジュール構造を作成（detector、analyzer、generator、applier、preview、models、utils）
    - Cargo.tomlに必要な依存関係を追加（image、clap、tokio、serde、thiserror等）
    - _Requirements: 4.1.1, 4.1.2, 4.1.3, 4.1.4_
  
  - [x] 1.2 コアデータ型の定義（models/mod.rs）
    - Rgb、Lab、Xyz構造体を定義
    - ColorInfo、AnsiColors、ColorScheme、FontConfig構造体を定義
    - TerminalType、ShellType、InputSource列挙型を定義
    - BackupInfo、AppConfig構造体を定義
    - _Requirements: 2.1, 2.2, 2.3, 2.4_
  
  - [x] 1.3 エラー型の定義
    - TwfError列挙型を定義（thiserrorを使用）
    - 各エラーケースに適切なエラーメッセージを設定
    - _Requirements: 2.7.7_

- [ ] 2. ユーティリティ機能の実装
  - [x] 2.1 色空間変換関数の実装（utils/color_space.rs）
    - rgb_to_lab、lab_to_rgb関数を実装
    - rgb_to_xyz、xyz_to_rgb、xyz_to_lab、lab_to_xyz関数を実装
    - srgb_to_linear、linear_to_srgb関数を実装
    - _Requirements: 2.1.2_
  
  - [x] 2.2 色空間変換のプロパティテストを作成
    - **Property 12: 色空間変換のラウンドトリップ**
    - **Validates: Requirements（色空間変換の正確性）**
  
  - [x] 2.3 コントラスト比計算関数の実装（analyzer/contrast.rs）
    - calculate_contrast_ratio関数を実装（WCAG 2.1基準）
    - calculate_relative_luminance関数を実装
    - _Requirements: 2.2.5_
  
  - [ ]* 2.4 コントラスト比計算のプロパティテストを作成
    - **Property 13: コントラスト比計算の対称性**
    - **Validates: Requirements（コントラスト比計算の正確性）**
  
  - [x] 2.5 色操作関数の実装（utils/color_space.rs）
    - calculate_saturation、calculate_hue関数を実装
    - lighten関数を実装
    - generate_color_from_hue関数を実装
    - _Requirements: 2.1.3_
  
  - [ ]* 2.6 色操作のプロパティテストを作成
    - **Property 14: 明度の単調性**
    - **Validates: Requirements（色操作の正確性）**

- [x] 3. チェックポイント - 基礎機能の確認
  - すべてのテストが通ることを確認し、質問があればユーザーに確認してください

- [ ] 4. 画像解析と色解析の実装
  - [x] 4.1 画像解析機能の実装（analyzer/image.rs）
    - ImageAnalyzer構造体を実装
    - analyze関数を実装（画像読み込み、リサイズ、ピクセルサンプリング）
    - sample_pixels関数を実装（グリッドサンプリング）
    - _Requirements: 2.1.1, 2.1.4, 2.1.5_
  
  - [x] 4.2 K-meansクラスタリングの実装（analyzer/image.rs）
    - kmeans_clustering関数を実装
    - 主要色の抽出ロジックを実装
    - _Requirements: 2.1.1_
  
  - [ ]* 4.3 画像解析のプロパティテストを作成
    - **Property 1: 画像解析の一貫性**
    - **Validates: Requirements 2.1.1, 2.1.2, 2.1.3**
  
  - [x] 4.4 色解析機能の実装（analyzer/color.rs）
    - ColorAnalyzer構造体を実装
    - analyze関数を実装（単一色から色情報を抽出）
    - _Requirements: 2.1.8_
  
  - [ ]* 4.5 背景色からカラースキーム生成のプロパティテストを作成
    - **Property 3: 背景色からカラースキーム生成**
    - **Validates: Requirements 2.1.8**

- [ ] 5. 背景画像/背景色の検出機能の実装
  - [x] 5.1 ターミナル種別判定の実装（detector/terminal.rs）
    - detect_terminal関数を実装（環境変数をチェック）
    - TerminalType列挙型の各ケースに対応
    - _Requirements: 2.8.3_
  
  - [ ]* 5.2 ターミナルタイプ判定のプロパティテストを作成
    - **Property 10: ターミナルタイプの判定**
    - **Validates: Requirements 2.8.3**
  
  - [x] 5.3 iTerm2背景画像検出の実装（detector/iterm2.rs）
    - detect_iterm2_background関数を実装
    - plistファイルのパース処理を実装
    - _Requirements: 2.8.1, 2.8.2_
  
  - [x] 5.4 Alacritty背景画像検出の実装（detector/alacritty.rs）
    - detect_alacritty_background関数を実装
    - YAML/TOML設定ファイルのパース処理を実装
    - _Requirements: 2.8.1, 2.8.2_
  
  - [x] 5.5 Windows Terminal背景画像検出の実装（detector/windows_terminal.rs）
    - detect_windows_terminal_background関数を実装
    - JSON設定ファイルのパース処理を実装
    - _Requirements: 2.8.1, 2.8.2_
  
  - [x] 5.6 GNOME Terminal背景画像検出の実装（detector/gnome_terminal.rs）
    - detect_gnome_terminal_background関数を実装
    - dconf/gsettingsからの設定取得を実装
    - _Requirements: 2.8.1, 2.8.2_
  
  - [x] 5.7 Kitty背景画像検出の実装（detector/kitty.rs）
    - detect_kitty_background関数を実装
    - Kitty設定ファイルのパース処理を実装
    - _Requirements: 2.8.1, 2.8.2_
  
  - [x] 5.8 WezTerm背景画像検出の実装（detector/wezterm.rs）
    - detect_wezterm_background関数を実装
    - WezTerm設定ファイルのパース処理を実装
    - _Requirements: 2.8.1, 2.8.2_
  
  - [x] 5.9 自動検出ロジックの実装（detector/auto.rs）
    - AutoDetector構造体を実装
    - detect_background_image関数を実装（ターミナルタイプに応じて適切な検出関数を呼び出し）
    - _Requirements: 2.8.1, 2.8.4_
  
  - [ ]* 5.10 背景画像パス検出のプロパティテストを作成
    - **Property 11: 背景画像パス検出の妥当性**
    - **Validates: Requirements 2.8.1**
  
  - [x] 5.11 背景色検出の実装（detector/bg_color.rs）
    - BgColorDetector構造体を実装
    - detect_background_color関数を実装（OSC 11エスケープシーケンス）
    - parse_osc11_response関数を実装
    - _Requirements: 2.1.7_
  
  - [ ]* 5.12 OSC 11レスポンスパースのプロパティテストを作成
    - **Property 2: OSC 11レスポンスのパース**
    - **Validates: Requirements 2.1.7**

- [~] 6. チェックポイント - 検出機能の確認
  - すべてのテストが通ることを確認し、質問があればユーザーに確認してください

- [ ] 7. カラースキームとフォント設定の生成
  - [x] 7.1 カラースキーム生成の実装（generator/scheme.rs）
    - SchemeGenerator構造体を実装
    - generate関数を実装（ColorInfoからColorSchemeを生成）
    - calculate_foreground_color関数を実装
    - _Requirements: 2.2.1_
  
  - [x] 7.2 コントラスト比調整の実装（generator/scheme.rs）
    - adjust_for_contrast関数を実装
    - WCAG AA基準（4.5:1）を満たすように色を調整
    - _Requirements: 2.2.5_
  
  - [ ]* 7.3 コントラスト比保証のプロパティテストを作成
    - **Property 4: コントラスト比の保証**
    - **Validates: Requirements 2.2.5**
  
  - [x] 7.4 ANSI 16色生成の実装（generator/scheme.rs）
    - generate_ansi_colors関数を実装
    - 色相環を使用して調和する色を生成
    - _Requirements: 2.2.2_
  
  - [ ]* 7.5 ANSI 16色完全性のプロパティテストを作成
    - **Property 5: ANSI 16色の完全性**
    - **Validates: Requirements 2.2.2**
  
  - [x] 7.6 256色パレット生成の実装（generator/scheme.rs）
    - generate_256_palette関数を実装
    - 256色対応ターミナル向けの詳細な色調整を提供
    - _Requirements: 2.2.3_
  
  - [ ]* 7.7 256色パレット完全性のプロパティテストを作成
    - **Property 6: 256色パレットの完全性**
    - **Validates: Requirements 2.2.3**
  
  - [x] 7.8 True Colorサポート検出の実装（utils/mod.rs）
    - detect_true_color_support関数を実装
    - supports_256_colors関数を実装
    - _Requirements: 2.2.4_
  
  - [~] 7.9 フォント設定生成の実装（generator/font.rs）
    - FontOptimizer構造体を実装
    - optimize関数を実装（明度と彩度に基づいてフォントウェイトを決定）
    - detect_monospace_fonts関数を実装
    - _Requirements: 2.3.1, 2.3.2, 2.3.3_
  
  - [ ]* 7.10 フォントウェイト選択のプロパティテストを作成
    - **Property 7: フォントウェイト選択の一貫性**
    - **Validates: Requirements 2.3.1, 2.3.2**

- [ ] 8. 設定適用とバックアップ機能の実装
  - [~] 8.1 シェル種別検出の実装（applier/shell.rs）
    - detect_shell_type関数を実装
    - get_shell_config_path関数を実装
    - _Requirements: 2.4.1, 2.4.3_
  
  - [ ]* 8.2 シェル設定ファイルパスのプロパティテストを作成
    - **Property 8: シェル設定ファイルパスの正確性**
    - **Validates: Requirements 2.4.3**
  
  - [~] 8.3 Bash/Zsh設定生成の実装（applier/shell.rs）
    - generate_bash_config関数を実装
    - ANSIエスケープシーケンスを含む設定ブロックを生成
    - _Requirements: 2.4.2, 2.4.4_
  
  - [~] 8.4 Fish設定生成の実装（applier/shell.rs）
    - generate_fish_config関数を実装
    - Fish形式の設定ブロックを生成
    - _Requirements: 2.4.2, 2.4.4_
  
  - [~] 8.5 PowerShell設定生成の実装（applier/shell.rs）
    - generate_powershell_config関数を実装
    - PowerShell形式の設定ブロックを生成
    - _Requirements: 2.4.2, 2.4.4_
  
  - [~] 8.6 バックアップ管理の実装（applier/backup.rs）
    - BackupManager構造体を実装
    - create_backup関数を実装（タイムスタンプ付きバックアップファイルを作成）
    - get_latest_backup関数を実装
    - _Requirements: 2.5.3, 3.6.2_
  
  - [~] 8.7 ロールバック機能の実装（applier/rollback.rs）
    - rollback関数を実装
    - バックアップから設定を復元
    - _Requirements: 2.5.4_
  
  - [ ]* 8.8 設定のラウンドトリップのプロパティテストを作成
    - **Property 9: 設定のラウンドトリップ（バックアップとロールバック）**
    - **Validates: Requirements 2.5.4**
  
  - [~] 8.9 設定適用の実装（applier/shell.rs）
    - ConfigApplier構造体を実装
    - apply関数を実装（バックアップ作成、ユーザー確認、設定書き込み）
    - write_config関数を実装
    - apply_to_current_session関数を実装
    - _Requirements: 2.5.1, 2.5.2, 3.6.1_
  
  - [~] 8.10 ファイル操作ユーティリティの実装（utils/file.rs）
    - append_to_file関数を実装
    - remove_existing_twf_block関数を実装
    - confirm_apply関数を実装（ユーザー確認）
    - _Requirements: 2.5.2, 3.6.1_

- [~] 9. チェックポイント - 設定適用機能の確認
  - すべてのテストが通ることを確認し、質問があればユーザーに確認してください

- [ ] 10. CLIインターフェースとプレビュー機能の実装
  - [~] 10.1 CLI引数パースの実装（cli.rs）
    - CliArgs構造体を定義（clapを使用）
    - すべてのオプション（--image、--detect、--color、--preview、--apply、--rollback、--verbose）を定義
    - ヘルプメッセージとバージョン情報を設定
    - _Requirements: 2.7.1, 2.7.2, 2.7.3, 2.7.4, 2.7.5, 2.7.6, 2.7.7, 2.7.8_
  
  - [~] 10.2 入力ソース決定の実装（main.rs）
    - determine_input_source関数を実装
    - 画像パス、自動検出、背景色の優先順位を処理
    - _Requirements: 2.7.2, 2.7.3, 2.7.4_
  
  - [~] 10.3 メイン処理フローの実装（main.rs）
    - run関数を実装
    - 入力ソースの決定 → 色情報の取得 → カラースキーム生成 → モード別処理
    - フォールバック戦略を実装（画像検出 → 背景色検出 → デフォルト色）
    - _Requirements: 2.1.6, 2.1.9, 2.8.4_
  
  - [~] 10.4 プレビュー表示の実装（preview/mod.rs）
    - PreviewRenderer構造体を実装
    - render関数を実装
    - render_ansi_colors関数を実装（16色のサンプル表示）
    - render_command_examples関数を実装（ls、git status等の出力例）
    - render_font_config関数を実装
    - render_contrast_info関数を実装
    - _Requirements: 2.6.1, 2.6.2, 2.6.3_
  
  - [~] 10.5 進行状況表示の実装（main.rs）
    - プログレスバーまたはスピナーを実装
    - 各処理ステップの進行状況を表示
    - _Requirements: 3.4.3_

- [ ] 11. エラーハンドリングとロギングの実装
  - [~] 11.1 エラーメッセージの改善
    - 各エラーケースに対して、原因と解決方法を含む詳細なメッセージを実装
    - 日本語と英語の両方のメッセージを提供
    - _Requirements: 2.7.7, 3.4.2_
  
  - [~] 11.2 ロギング機能の実装（utils/mod.rs）
    - log_debug、log_info、log_warning、log_errorマクロを実装
    - --verboseフラグに応じて詳細出力を制御
    - _Requirements: 2.7.6_
  
  - [~] 11.3 フォールバック戦略のテスト
    - 画像検出失敗時のフォールバックをテスト
    - 背景色検出失敗時のフォールバックをテスト
    - デフォルト背景色の使用をテスト
    - _Requirements: 2.1.6, 2.1.9, 2.8.4_

- [ ] 12. 統合とエンドツーエンドテスト
  - [~] 12.1 統合テストの作成（tests/integration/）
    - 画像パス指定モードのエンドツーエンドテスト
    - 自動検出モードのエンドツーエンドテスト
    - 背景色指定モードのエンドツーエンドテスト
    - プレビューモードのテスト
    - 適用モードのテスト
    - ロールバックモードのテスト
    - _Requirements: すべての機能要件_
  
  - [~] 12.2 クロスプラットフォームテストの実施
    - Linux、macOS、Windowsでの動作確認
    - 各OSでのターミナル検出とシェル検出のテスト
    - _Requirements: 2.4.1, 3.5.1_
  
  - [~] 12.3 パフォーマンステストの実施
    - 画像解析が3秒以内に完了することを確認
    - 全体の処理が5秒以内に完了することを確認
    - メモリ使用量が100MB以下であることを確認
    - 4K解像度の画像でも適切に処理できることを確認
    - _Requirements: 2.1.5, 3.1.1, 3.1.2, 3.1.3_

- [ ] 13. ドキュメントとパッケージング
  - [x] 13.1 README.mdの作成
    - インストール方法、使用方法、オプションの説明を記載
    - 日本語と英語の両方で作成
    - _Requirements: 3.3.4_
  
  - [~] 13.2 LICENSEファイルの作成
    - MIT または Apache 2.0ライセンスを選択
    - _Requirements: 4.4.3_
  
  - [~] 13.3 リリースビルドの作成
    - 各プラットフォーム向けのバイナリをビルド
    - 単一のバイナリファイルとして配布できることを確認
    - _Requirements: 3.2.3_

- [~] 14. 最終チェックポイント
  - すべてのテストが通ることを確認し、ユーザーに最終確認を求めてください

## 注意事項

- `*`マークが付いたタスクはオプションであり、より迅速なMVPのためにスキップ可能です
- 各タスクは特定の要件を参照しており、トレーサビリティを確保しています
- チェックポイントタスクでは、段階的な検証を行い、問題があれば早期に発見します
- プロパティテストは、普遍的な正確性プロパティを検証し、ユニットテストは具体的な例とエッジケースを検証します
