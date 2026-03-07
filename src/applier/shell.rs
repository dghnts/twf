// シェル設定適用

use crate::models::{ColorScheme, FontConfig, Result, ShellType, TwfError};
use crate::applier::backup::BackupManager;
use crate::utils::file::{append_to_file, remove_existing_twf_block, confirm_apply};
use chrono::Local;
use std::env;
use std::path::{Path, PathBuf};

/// 設定適用マネージャー
pub struct ConfigApplier {
    backup_manager: BackupManager,
}

impl ConfigApplier {
    /// 新しいConfigApplierを作成
    /// 
    /// # Arguments
    /// * `backup_dir` - バックアップディレクトリのパス
    /// 
    /// # Returns
    /// * `ConfigApplier` - 新しいConfigApplierインスタンス
    pub fn new(backup_dir: PathBuf) -> Self {
        Self {
            backup_manager: BackupManager::new(backup_dir),
        }
    }

    /// 設定を適用
    /// 
    /// カラースキームとフォント設定をシェル設定ファイルに適用します。
    /// 適用前にバックアップを作成し、ユーザーに確認を求めます。
    /// 
    /// # Arguments
    /// * `scheme` - カラースキーム
    /// * `font` - フォント設定
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時はOk(())、失敗時はエラー
    pub async fn apply(&self, scheme: &ColorScheme, font: &FontConfig) -> Result<()> {
        // 1. シェルタイプを検出
        let shell_type = detect_shell_type()?;
        println!("検出されたシェル: {:?}", shell_type);

        // 2. 設定ファイルパスを取得
        let config_path = get_shell_config_path(&shell_type)?;
        println!("設定ファイル: {}", config_path.display());

        // 3. バックアップを作成
        println!("バックアップを作成中...");
        let backup_info = self.backup_manager.create_backup(&config_path).await?;
        println!("バックアップを作成しました: {}", backup_info.backup_path.display());

        // 4. ユーザーに確認
        if !confirm_apply()? {
            println!("設定の適用をキャンセルしました。");
            return Ok(());
        }

        // 5. 設定を書き込み
        println!("設定を書き込み中...");
        self.write_config(&config_path, &shell_type, scheme, font).await?;
        println!("設定を書き込みました。");

        // 6. 現在のセッションに適用
        println!("現在のセッションに適用中...");
        self.apply_to_current_session(scheme).await?;
        println!("設定を適用しました。");

        println!("\n設定が正常に適用されました！");
        println!("変更を有効にするには、シェルを再起動するか、以下のコマンドを実行してください:");
        match shell_type {
            ShellType::Bash => println!("  source ~/.bashrc"),
            ShellType::Zsh => println!("  source ~/.zshrc"),
            ShellType::Fish => println!("  source ~/.config/fish/config.fish"),
            ShellType::PowerShell => println!("  . $PROFILE"),
        }

        Ok(())
    }

    /// シェル設定ファイルに書き込み
    /// 
    /// 既存のTWFブロックを削除してから、新しい設定を追記します。
    /// 
    /// # Arguments
    /// * `path` - 設定ファイルのパス
    /// * `shell_type` - シェルタイプ
    /// * `scheme` - カラースキーム
    /// * `font` - フォント設定
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時はOk(())、失敗時はエラー
    async fn write_config(
        &self,
        path: &Path,
        shell_type: &ShellType,
        scheme: &ColorScheme,
        font: &FontConfig,
    ) -> Result<()> {
        // 既存のTWFブロックを削除
        remove_existing_twf_block(path).await?;

        // シェルタイプに応じた設定を生成
        let config_lines = match shell_type {
            ShellType::Bash | ShellType::Zsh => generate_bash_config(scheme, font),
            ShellType::Fish => generate_fish_config(scheme, font),
            ShellType::PowerShell => generate_powershell_config(scheme, font),
        };

        // 設定を追記
        append_to_file(path, &config_lines).await?;

        Ok(())
    }

    /// 現在のセッションに適用
    /// 
    /// ANSIエスケープシーケンスを使用して、現在のターミナルセッションに
    /// カラースキームを適用します。
    /// 
    /// # Arguments
    /// * `scheme` - カラースキーム
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時はOk(())、失敗時はエラー
    async fn apply_to_current_session(&self, scheme: &ColorScheme) -> Result<()> {
        // フォアグラウンドカラーを設定
        print!(
            "\x1b]10;rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}\x07",
            scheme.foreground.r,
            scheme.foreground.r,
            scheme.foreground.g,
            scheme.foreground.g,
            scheme.foreground.b,
            scheme.foreground.b
        );

        // ANSI色を環境変数として設定（現在のプロセスのみ）
        env::set_var(
            "TWF_ANSI_RED",
            format!(
                "\x1b[38;2;{};{};{}m",
                scheme.ansi_colors.red.r, scheme.ansi_colors.red.g, scheme.ansi_colors.red.b
            ),
        );
        env::set_var(
            "TWF_ANSI_GREEN",
            format!(
                "\x1b[38;2;{};{};{}m",
                scheme.ansi_colors.green.r, scheme.ansi_colors.green.g, scheme.ansi_colors.green.b
            ),
        );
        env::set_var(
            "TWF_ANSI_BLUE",
            format!(
                "\x1b[38;2;{};{};{}m",
                scheme.ansi_colors.blue.r, scheme.ansi_colors.blue.g, scheme.ansi_colors.blue.b
            ),
        );

        Ok(())
    }
}

/// 現在のシェルタイプを検出
/// 
/// 環境変数SHELLをチェックして、現在使用中のシェルを判定します。
/// 
/// # 戻り値
/// 
/// - `Ok(ShellType)` - 検出されたシェルタイプ
/// - `Err(TwfError::ShellDetectionError)` - シェルを判定できなかった場合
/// 
/// # 例
/// 
/// ```
/// let shell_type = detect_shell_type()?;
/// println!("検出されたシェル: {:?}", shell_type);
/// ```
pub fn detect_shell_type() -> Result<ShellType> {
    // 環境変数SHELLをチェック
    let shell_path = env::var("SHELL")
        .or_else(|_| env::var("COMSPEC")) // Windows用のフォールバック
        .map_err(|_| TwfError::ShellDetectionError)?;
    
    // シェルパスからシェル名を抽出
    let shell_name = shell_path
        .split(['/', '\\'])
        .last()
        .ok_or(TwfError::ShellDetectionError)?
        .to_lowercase();
    
    // シェル名からShellTypeを判定
    if shell_name.contains("bash") {
        Ok(ShellType::Bash)
    } else if shell_name.contains("zsh") {
        Ok(ShellType::Zsh)
    } else if shell_name.contains("fish") {
        Ok(ShellType::Fish)
    } else if shell_name.contains("pwsh") || shell_name.contains("powershell") {
        Ok(ShellType::PowerShell)
    } else {
        Err(TwfError::ShellDetectionError)
    }
}

/// シェル設定ファイルのパスを取得
/// 
/// 指定されたシェルタイプに対応する設定ファイルのパスを返します。
/// 
/// # 引数
/// 
/// - `shell_type` - シェルタイプ
/// 
/// # 戻り値
/// 
/// - `Ok(PathBuf)` - 設定ファイルのパス
/// - `Err(TwfError::ConfigFileNotFound)` - 設定ファイルが見つからない場合
/// 
/// # 例
/// 
/// ```
/// let shell_type = ShellType::Bash;
/// let config_path = get_shell_config_path(&shell_type)?;
/// println!("設定ファイル: {:?}", config_path);
/// ```
pub fn get_shell_config_path(shell_type: &ShellType) -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| TwfError::ConfigFileNotFound(PathBuf::from("ホームディレクトリ")))?;
    
    let config_path = match shell_type {
        ShellType::Bash => {
            // Bashの設定ファイルを優先順位順にチェック
            let candidates = vec![
                home_dir.join(".bashrc"),
                home_dir.join(".bash_profile"),
                home_dir.join(".profile"),
            ];
            
            candidates.into_iter()
                .find(|path| path.exists())
                .unwrap_or_else(|| home_dir.join(".bashrc")) // デフォルトは.bashrc
        }
        ShellType::Zsh => {
            // Zshの設定ファイルを優先順位順にチェック
            let candidates = vec![
                home_dir.join(".zshrc"),
                home_dir.join(".zprofile"),
            ];
            
            candidates.into_iter()
                .find(|path| path.exists())
                .unwrap_or_else(|| home_dir.join(".zshrc")) // デフォルトは.zshrc
        }
        ShellType::Fish => {
            // Fishの設定ファイルパス
            let config_dir = dirs::config_dir()
                .unwrap_or_else(|| home_dir.join(".config"));
            
            config_dir.join("fish").join("config.fish")
        }
        ShellType::PowerShell => {
            // PowerShellの設定ファイルパス
            // Windows: ~\Documents\PowerShell\Microsoft.PowerShell_profile.ps1
            // Unix: ~/.config/powershell/Microsoft.PowerShell_profile.ps1
            
            #[cfg(target_os = "windows")]
            {
                let documents_dir = home_dir.join("Documents");
                documents_dir.join("PowerShell").join("Microsoft.PowerShell_profile.ps1")
            }
            
            #[cfg(not(target_os = "windows"))]
            {
                let config_dir = dirs::config_dir()
                    .unwrap_or_else(|| home_dir.join(".config"));
                config_dir.join("powershell").join("Microsoft.PowerShell_profile.ps1")
            }
        }
    };
    
    Ok(config_path)
}

/// Bash/Zsh用の設定ブロックを生成
/// 
/// ANSIエスケープシーケンスを含む設定ブロックを生成します。
/// 生成される設定には、ANSI 16色の環境変数とフォント推奨設定が含まれます。
/// 
/// # 引数
/// 
/// - `scheme` - カラースキーム
/// - `font` - フォント設定
/// 
/// # 戻り値
/// 
/// 生成された設定ブロック（文字列）
/// 
/// # 例
/// 
/// ```
/// let config = generate_bash_config(&scheme, &font);
/// println!("{}", config);
/// ```
pub fn generate_bash_config(scheme: &ColorScheme, font: &FontConfig) -> String {
    let mut lines = Vec::new();
    
    // ヘッダー
    lines.push("# === TWF Generated Config ===".to_string());
    lines.push(format!("# Generated at: {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
    lines.push("# このセクションはTWFによって自動生成されました".to_string());
    lines.push(String::new());
    
    // ANSI色の設定
    lines.push("# ANSI色の設定".to_string());
    lines.push("# これらの環境変数は、ターミナルの色を定義します".to_string());
    
    // 基本8色
    lines.push(format!("export TWF_ANSI_BLACK='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.black.r, scheme.ansi_colors.black.g, scheme.ansi_colors.black.b));
    lines.push(format!("export TWF_ANSI_RED='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.red.r, scheme.ansi_colors.red.g, scheme.ansi_colors.red.b));
    lines.push(format!("export TWF_ANSI_GREEN='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.green.r, scheme.ansi_colors.green.g, scheme.ansi_colors.green.b));
    lines.push(format!("export TWF_ANSI_YELLOW='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.yellow.r, scheme.ansi_colors.yellow.g, scheme.ansi_colors.yellow.b));
    lines.push(format!("export TWF_ANSI_BLUE='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.blue.r, scheme.ansi_colors.blue.g, scheme.ansi_colors.blue.b));
    lines.push(format!("export TWF_ANSI_MAGENTA='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.magenta.r, scheme.ansi_colors.magenta.g, scheme.ansi_colors.magenta.b));
    lines.push(format!("export TWF_ANSI_CYAN='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.cyan.r, scheme.ansi_colors.cyan.g, scheme.ansi_colors.cyan.b));
    lines.push(format!("export TWF_ANSI_WHITE='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.white.r, scheme.ansi_colors.white.g, scheme.ansi_colors.white.b));
    
    lines.push(String::new());
    
    // 明るい8色
    lines.push("# 明るいANSI色".to_string());
    lines.push(format!("export TWF_ANSI_BRIGHT_BLACK='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_black.r, scheme.ansi_colors.bright_black.g, scheme.ansi_colors.bright_black.b));
    lines.push(format!("export TWF_ANSI_BRIGHT_RED='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_red.r, scheme.ansi_colors.bright_red.g, scheme.ansi_colors.bright_red.b));
    lines.push(format!("export TWF_ANSI_BRIGHT_GREEN='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_green.r, scheme.ansi_colors.bright_green.g, scheme.ansi_colors.bright_green.b));
    lines.push(format!("export TWF_ANSI_BRIGHT_YELLOW='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_yellow.r, scheme.ansi_colors.bright_yellow.g, scheme.ansi_colors.bright_yellow.b));
    lines.push(format!("export TWF_ANSI_BRIGHT_BLUE='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_blue.r, scheme.ansi_colors.bright_blue.g, scheme.ansi_colors.bright_blue.b));
    lines.push(format!("export TWF_ANSI_BRIGHT_MAGENTA='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_magenta.r, scheme.ansi_colors.bright_magenta.g, scheme.ansi_colors.bright_magenta.b));
    lines.push(format!("export TWF_ANSI_BRIGHT_CYAN='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_cyan.r, scheme.ansi_colors.bright_cyan.g, scheme.ansi_colors.bright_cyan.b));
    lines.push(format!("export TWF_ANSI_BRIGHT_WHITE='\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_white.r, scheme.ansi_colors.bright_white.g, scheme.ansi_colors.bright_white.b));
    
    lines.push(String::new());
    
    // フォアグラウンドとバックグラウンドの色
    lines.push("# フォアグラウンドとバックグラウンドの色".to_string());
    lines.push(format!("export TWF_FOREGROUND='\\e[38;2;{};{};{}m'", 
        scheme.foreground.r, scheme.foreground.g, scheme.foreground.b));
    
    lines.push(String::new());
    
    // リセットシーケンス
    lines.push("# リセットシーケンス".to_string());
    lines.push("export TWF_RESET='\\e[0m'".to_string());
    
    lines.push(String::new());
    
    // フォント設定の推奨事項
    lines.push("# フォント設定の推奨事項".to_string());
    lines.push("# 以下の設定は、ターミナルエミュレータの設定で手動で適用してください".to_string());
    lines.push(format!("# 推奨フォントウェイト: {:?}", font.weight));
    
    if !font.recommended_fonts.is_empty() {
        lines.push(format!("# 推奨フォント: {}", font.recommended_fonts.join(", ")));
    }
    
    lines.push(String::new());
    
    // 使用例
    lines.push("# 使用例:".to_string());
    lines.push("# echo -e \"${TWF_ANSI_RED}赤色のテキスト${TWF_RESET}\"".to_string());
    lines.push("# echo -e \"${TWF_ANSI_GREEN}緑色のテキスト${TWF_RESET}\"".to_string());
    
    lines.push(String::new());
    lines.push("# === End TWF Config ===".to_string());
    
    lines.join("\n")
}

/// Fish用の設定ブロックを生成
/// 
/// Fish形式（set -x）の設定ブロックを生成します。
/// 生成される設定には、ANSI 16色の環境変数とフォント推奨設定が含まれます。
/// 
/// # 引数
/// 
/// - `scheme` - カラースキーム
/// - `font` - フォント設定
/// 
/// # 戻り値
/// 
/// 生成された設定ブロック（文字列）
/// 
/// # 例
/// 
/// ```
/// let config = generate_fish_config(&scheme, &font);
/// println!("{}", config);
/// ```
pub fn generate_fish_config(scheme: &ColorScheme, font: &FontConfig) -> String {
    let mut lines = Vec::new();
    
    // ヘッダー
    lines.push("# === TWF Generated Config ===".to_string());
    lines.push(format!("# Generated at: {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
    lines.push("# このセクションはTWFによって自動生成されました".to_string());
    lines.push(String::new());
    
    // ANSI色の設定
    lines.push("# ANSI色の設定".to_string());
    lines.push("# これらの環境変数は、ターミナルの色を定義します".to_string());
    
    // 基本8色
    lines.push(format!("set -x TWF_ANSI_BLACK '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.black.r, scheme.ansi_colors.black.g, scheme.ansi_colors.black.b));
    lines.push(format!("set -x TWF_ANSI_RED '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.red.r, scheme.ansi_colors.red.g, scheme.ansi_colors.red.b));
    lines.push(format!("set -x TWF_ANSI_GREEN '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.green.r, scheme.ansi_colors.green.g, scheme.ansi_colors.green.b));
    lines.push(format!("set -x TWF_ANSI_YELLOW '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.yellow.r, scheme.ansi_colors.yellow.g, scheme.ansi_colors.yellow.b));
    lines.push(format!("set -x TWF_ANSI_BLUE '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.blue.r, scheme.ansi_colors.blue.g, scheme.ansi_colors.blue.b));
    lines.push(format!("set -x TWF_ANSI_MAGENTA '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.magenta.r, scheme.ansi_colors.magenta.g, scheme.ansi_colors.magenta.b));
    lines.push(format!("set -x TWF_ANSI_CYAN '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.cyan.r, scheme.ansi_colors.cyan.g, scheme.ansi_colors.cyan.b));
    lines.push(format!("set -x TWF_ANSI_WHITE '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.white.r, scheme.ansi_colors.white.g, scheme.ansi_colors.white.b));
    
    lines.push(String::new());
    
    // 明るい8色
    lines.push("# 明るいANSI色".to_string());
    lines.push(format!("set -x TWF_ANSI_BRIGHT_BLACK '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_black.r, scheme.ansi_colors.bright_black.g, scheme.ansi_colors.bright_black.b));
    lines.push(format!("set -x TWF_ANSI_BRIGHT_RED '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_red.r, scheme.ansi_colors.bright_red.g, scheme.ansi_colors.bright_red.b));
    lines.push(format!("set -x TWF_ANSI_BRIGHT_GREEN '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_green.r, scheme.ansi_colors.bright_green.g, scheme.ansi_colors.bright_green.b));
    lines.push(format!("set -x TWF_ANSI_BRIGHT_YELLOW '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_yellow.r, scheme.ansi_colors.bright_yellow.g, scheme.ansi_colors.bright_yellow.b));
    lines.push(format!("set -x TWF_ANSI_BRIGHT_BLUE '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_blue.r, scheme.ansi_colors.bright_blue.g, scheme.ansi_colors.bright_blue.b));
    lines.push(format!("set -x TWF_ANSI_BRIGHT_MAGENTA '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_magenta.r, scheme.ansi_colors.bright_magenta.g, scheme.ansi_colors.bright_magenta.b));
    lines.push(format!("set -x TWF_ANSI_BRIGHT_CYAN '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_cyan.r, scheme.ansi_colors.bright_cyan.g, scheme.ansi_colors.bright_cyan.b));
    lines.push(format!("set -x TWF_ANSI_BRIGHT_WHITE '\\e[38;2;{};{};{}m'", 
        scheme.ansi_colors.bright_white.r, scheme.ansi_colors.bright_white.g, scheme.ansi_colors.bright_white.b));
    
    lines.push(String::new());
    
    // フォアグラウンドとバックグラウンドの色
    lines.push("# フォアグラウンドとバックグラウンドの色".to_string());
    lines.push(format!("set -x TWF_FOREGROUND '\\e[38;2;{};{};{}m'", 
        scheme.foreground.r, scheme.foreground.g, scheme.foreground.b));
    
    lines.push(String::new());
    
    // リセットシーケンス
    lines.push("# リセットシーケンス".to_string());
    lines.push("set -x TWF_RESET '\\e[0m'".to_string());
    
    lines.push(String::new());
    
    // フォント設定の推奨事項
    lines.push("# フォント設定の推奨事項".to_string());
    lines.push("# 以下の設定は、ターミナルエミュレータの設定で手動で適用してください".to_string());
    lines.push(format!("# 推奨フォントウェイト: {:?}", font.weight));
    
    if !font.recommended_fonts.is_empty() {
        lines.push(format!("# 推奨フォント: {}", font.recommended_fonts.join(", ")));
    }
    
    lines.push(String::new());
    
    // 使用例
    lines.push("# 使用例:".to_string());
    lines.push("# echo -e \"$TWF_ANSI_RED\"赤色のテキスト\"$TWF_RESET\"".to_string());
    lines.push("# echo -e \"$TWF_ANSI_GREEN\"緑色のテキスト\"$TWF_RESET\"".to_string());
    
    lines.push(String::new());
    lines.push("# === End TWF Config ===".to_string());
    
    lines.join("\n")
}

/// PowerShell用の設定ブロックを生成
/// 
/// PowerShell形式（$env:変数名）の設定ブロックを生成します。
/// 生成される設定には、ANSI 16色の環境変数とフォント推奨設定が含まれます。
/// 
/// # 引数
/// 
/// - `scheme` - カラースキーム
/// - `font` - フォント設定
/// 
/// # 戻り値
/// 
/// 生成された設定ブロック（文字列）
/// 
/// # 例
/// 
/// ```
/// let config = generate_powershell_config(&scheme, &font);
/// println!("{}", config);
/// ```
pub fn generate_powershell_config(scheme: &ColorScheme, font: &FontConfig) -> String {
    let mut lines = Vec::new();
    
    // ヘッダー
    lines.push("# === TWF Generated Config ===".to_string());
    lines.push(format!("# Generated at: {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
    lines.push("# このセクションはTWFによって自動生成されました".to_string());
    lines.push(String::new());
    
    // ANSI色の設定
    lines.push("# ANSI色の設定".to_string());
    lines.push("# これらの環境変数は、ターミナルの色を定義します".to_string());
    
    // 基本8色
    lines.push(format!("$env:TWF_ANSI_BLACK = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.black.r, scheme.ansi_colors.black.g, scheme.ansi_colors.black.b));
    lines.push(format!("$env:TWF_ANSI_RED = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.red.r, scheme.ansi_colors.red.g, scheme.ansi_colors.red.b));
    lines.push(format!("$env:TWF_ANSI_GREEN = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.green.r, scheme.ansi_colors.green.g, scheme.ansi_colors.green.b));
    lines.push(format!("$env:TWF_ANSI_YELLOW = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.yellow.r, scheme.ansi_colors.yellow.g, scheme.ansi_colors.yellow.b));
    lines.push(format!("$env:TWF_ANSI_BLUE = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.blue.r, scheme.ansi_colors.blue.g, scheme.ansi_colors.blue.b));
    lines.push(format!("$env:TWF_ANSI_MAGENTA = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.magenta.r, scheme.ansi_colors.magenta.g, scheme.ansi_colors.magenta.b));
    lines.push(format!("$env:TWF_ANSI_CYAN = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.cyan.r, scheme.ansi_colors.cyan.g, scheme.ansi_colors.cyan.b));
    lines.push(format!("$env:TWF_ANSI_WHITE = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.white.r, scheme.ansi_colors.white.g, scheme.ansi_colors.white.b));
    
    lines.push(String::new());
    
    // 明るい8色
    lines.push("# 明るいANSI色".to_string());
    lines.push(format!("$env:TWF_ANSI_BRIGHT_BLACK = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.bright_black.r, scheme.ansi_colors.bright_black.g, scheme.ansi_colors.bright_black.b));
    lines.push(format!("$env:TWF_ANSI_BRIGHT_RED = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.bright_red.r, scheme.ansi_colors.bright_red.g, scheme.ansi_colors.bright_red.b));
    lines.push(format!("$env:TWF_ANSI_BRIGHT_GREEN = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.bright_green.r, scheme.ansi_colors.bright_green.g, scheme.ansi_colors.bright_green.b));
    lines.push(format!("$env:TWF_ANSI_BRIGHT_YELLOW = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.bright_yellow.r, scheme.ansi_colors.bright_yellow.g, scheme.ansi_colors.bright_yellow.b));
    lines.push(format!("$env:TWF_ANSI_BRIGHT_BLUE = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.bright_blue.r, scheme.ansi_colors.bright_blue.g, scheme.ansi_colors.bright_blue.b));
    lines.push(format!("$env:TWF_ANSI_BRIGHT_MAGENTA = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.bright_magenta.r, scheme.ansi_colors.bright_magenta.g, scheme.ansi_colors.bright_magenta.b));
    lines.push(format!("$env:TWF_ANSI_BRIGHT_CYAN = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.bright_cyan.r, scheme.ansi_colors.bright_cyan.g, scheme.ansi_colors.bright_cyan.b));
    lines.push(format!("$env:TWF_ANSI_BRIGHT_WHITE = \"`e[38;2;{};{};{}m\"", 
        scheme.ansi_colors.bright_white.r, scheme.ansi_colors.bright_white.g, scheme.ansi_colors.bright_white.b));
    
    lines.push(String::new());
    
    // フォアグラウンドとバックグラウンドの色
    lines.push("# フォアグラウンドとバックグラウンドの色".to_string());
    lines.push(format!("$env:TWF_FOREGROUND = \"`e[38;2;{};{};{}m\"", 
        scheme.foreground.r, scheme.foreground.g, scheme.foreground.b));
    
    lines.push(String::new());
    
    // リセットシーケンス
    lines.push("# リセットシーケンス".to_string());
    lines.push("$env:TWF_RESET = \"`e[0m\"".to_string());
    
    lines.push(String::new());
    
    // フォント設定の推奨事項
    lines.push("# フォント設定の推奨事項".to_string());
    lines.push("# 以下の設定は、ターミナルエミュレータの設定で手動で適用してください".to_string());
    lines.push(format!("# 推奨フォントウェイト: {:?}", font.weight));
    
    if !font.recommended_fonts.is_empty() {
        lines.push(format!("# 推奨フォント: {}", font.recommended_fonts.join(", ")));
    }
    
    lines.push(String::new());
    
    // 使用例
    lines.push("# 使用例:".to_string());
    lines.push("# Write-Host \"$env:TWF_ANSI_RED\"赤色のテキスト\"$env:TWF_RESET\"".to_string());
    lines.push("# Write-Host \"$env:TWF_ANSI_GREEN\"緑色のテキスト\"$env:TWF_RESET\"".to_string());
    
    lines.push(String::new());
    lines.push("# === End TWF Config ===".to_string());
    
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_shell_type() {
        // 環境変数SHELLが設定されている場合のテスト
        // 注: このテストは実行環境に依存します
        let result = detect_shell_type();
        
        // エラーでないことを確認（実際のシェルタイプは環境依存）
        if let Ok(shell_type) = result {
            println!("検出されたシェル: {:?}", shell_type);
        }
    }
    
    #[test]
    fn test_get_shell_config_path_bash() {
        let shell_type = ShellType::Bash;
        let result = get_shell_config_path(&shell_type);
        
        assert!(result.is_ok());
        let path = result.unwrap();
        
        // パスに.bashrcが含まれることを確認
        assert!(path.to_string_lossy().contains(".bashrc") 
            || path.to_string_lossy().contains(".bash_profile")
            || path.to_string_lossy().contains(".profile"));
    }
    
    #[test]
    fn test_get_shell_config_path_zsh() {
        let shell_type = ShellType::Zsh;
        let result = get_shell_config_path(&shell_type);
        
        assert!(result.is_ok());
        let path = result.unwrap();
        
        // パスに.zshrcが含まれることを確認
        assert!(path.to_string_lossy().contains(".zshrc")
            || path.to_string_lossy().contains(".zprofile"));
    }
    
    #[test]
    fn test_get_shell_config_path_fish() {
        let shell_type = ShellType::Fish;
        let result = get_shell_config_path(&shell_type);
        
        assert!(result.is_ok());
        let path = result.unwrap();
        
        // パスにconfig.fishが含まれることを確認
        assert!(path.to_string_lossy().contains("config.fish"));
    }
    
    #[test]
    fn test_get_shell_config_path_powershell() {
        let shell_type = ShellType::PowerShell;
        let result = get_shell_config_path(&shell_type);
        
        assert!(result.is_ok());
        let path = result.unwrap();
        
        // パスにMicrosoft.PowerShell_profile.ps1が含まれることを確認
        assert!(path.to_string_lossy().contains("Microsoft.PowerShell_profile.ps1"));
    }
    
    #[test]
    fn test_generate_bash_config() {
        use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
        
        // テスト用のカラースキームを作成
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };
        
        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };
        
        let font = FontConfig {
            weight: FontWeight::Normal,
            recommended_fonts: vec!["Fira Code".to_string(), "JetBrains Mono".to_string()],
        };
        
        // 設定を生成
        let config = generate_bash_config(&scheme, &font);
        
        // 設定ブロックのヘッダーとフッターが含まれることを確認
        assert!(config.contains("# === TWF Generated Config ==="));
        assert!(config.contains("# === End TWF Config ==="));
        
        // ANSI色の環境変数が含まれることを確認
        assert!(config.contains("export TWF_ANSI_BLACK="));
        assert!(config.contains("export TWF_ANSI_RED="));
        assert!(config.contains("export TWF_ANSI_GREEN="));
        assert!(config.contains("export TWF_ANSI_YELLOW="));
        assert!(config.contains("export TWF_ANSI_BLUE="));
        assert!(config.contains("export TWF_ANSI_MAGENTA="));
        assert!(config.contains("export TWF_ANSI_CYAN="));
        assert!(config.contains("export TWF_ANSI_WHITE="));
        
        // 明るいANSI色の環境変数が含まれることを確認
        assert!(config.contains("export TWF_ANSI_BRIGHT_BLACK="));
        assert!(config.contains("export TWF_ANSI_BRIGHT_RED="));
        assert!(config.contains("export TWF_ANSI_BRIGHT_GREEN="));
        assert!(config.contains("export TWF_ANSI_BRIGHT_YELLOW="));
        assert!(config.contains("export TWF_ANSI_BRIGHT_BLUE="));
        assert!(config.contains("export TWF_ANSI_BRIGHT_MAGENTA="));
        assert!(config.contains("export TWF_ANSI_BRIGHT_CYAN="));
        assert!(config.contains("export TWF_ANSI_BRIGHT_WHITE="));
        
        // フォアグラウンドとリセットシーケンスが含まれることを確認
        assert!(config.contains("export TWF_FOREGROUND="));
        assert!(config.contains("export TWF_RESET="));
        
        // フォント設定の推奨事項が含まれることを確認
        assert!(config.contains("推奨フォントウェイト: Normal"));
        assert!(config.contains("推奨フォント: Fira Code, JetBrains Mono"));
        
        // 使用例が含まれることを確認
        assert!(config.contains("使用例:"));
        
        // ANSIエスケープシーケンスの形式が正しいことを確認
        assert!(config.contains("\\e[38;2;"));
        assert!(config.contains("\\e[0m"));
    }
    
    #[test]
    fn test_generate_bash_config_with_empty_fonts() {
        use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
        
        // テスト用のカラースキーム（推奨フォントなし）
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };
        
        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };
        
        let font = FontConfig {
            weight: FontWeight::Bold,
            recommended_fonts: vec![],
        };
        
        // 設定を生成
        let config = generate_bash_config(&scheme, &font);
        
        // 基本的な構造が含まれることを確認
        assert!(config.contains("# === TWF Generated Config ==="));
        assert!(config.contains("# === End TWF Config ==="));
        
        // フォントウェイトは含まれるが、推奨フォントリストは含まれない
        assert!(config.contains("推奨フォントウェイト: Bold"));
        assert!(!config.contains("推奨フォント:"));
    }
    
    #[test]
    fn test_generate_fish_config() {
        use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
        
        // テスト用のカラースキームを作成
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };
        
        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };
        
        let font = FontConfig {
            weight: FontWeight::Normal,
            recommended_fonts: vec!["Fira Code".to_string(), "JetBrains Mono".to_string()],
        };
        
        // 設定を生成
        let config = generate_fish_config(&scheme, &font);
        
        // 設定ブロックのヘッダーとフッターが含まれることを確認
        assert!(config.contains("# === TWF Generated Config ==="));
        assert!(config.contains("# === End TWF Config ==="));
        
        // Fish形式の環境変数設定（set -x）が含まれることを確認
        assert!(config.contains("set -x TWF_ANSI_BLACK"));
        assert!(config.contains("set -x TWF_ANSI_RED"));
        assert!(config.contains("set -x TWF_ANSI_GREEN"));
        assert!(config.contains("set -x TWF_ANSI_YELLOW"));
        assert!(config.contains("set -x TWF_ANSI_BLUE"));
        assert!(config.contains("set -x TWF_ANSI_MAGENTA"));
        assert!(config.contains("set -x TWF_ANSI_CYAN"));
        assert!(config.contains("set -x TWF_ANSI_WHITE"));
        
        // 明るいANSI色の環境変数が含まれることを確認
        assert!(config.contains("set -x TWF_ANSI_BRIGHT_BLACK"));
        assert!(config.contains("set -x TWF_ANSI_BRIGHT_RED"));
        assert!(config.contains("set -x TWF_ANSI_BRIGHT_GREEN"));
        assert!(config.contains("set -x TWF_ANSI_BRIGHT_YELLOW"));
        assert!(config.contains("set -x TWF_ANSI_BRIGHT_BLUE"));
        assert!(config.contains("set -x TWF_ANSI_BRIGHT_MAGENTA"));
        assert!(config.contains("set -x TWF_ANSI_BRIGHT_CYAN"));
        assert!(config.contains("set -x TWF_ANSI_BRIGHT_WHITE"));
        
        // フォアグラウンドとリセットシーケンスが含まれることを確認
        assert!(config.contains("set -x TWF_FOREGROUND"));
        assert!(config.contains("set -x TWF_RESET"));
        
        // フォント設定の推奨事項が含まれることを確認
        assert!(config.contains("推奨フォントウェイト: Normal"));
        assert!(config.contains("推奨フォント: Fira Code, JetBrains Mono"));
        
        // 使用例が含まれることを確認
        assert!(config.contains("使用例:"));
        
        // ANSIエスケープシーケンスの形式が正しいことを確認
        assert!(config.contains("\\e[38;2;"));
        assert!(config.contains("\\e[0m"));
        
        // exportではなくset -xが使用されていることを確認
        assert!(!config.contains("export TWF_"));
    }
    
    #[test]
    fn test_generate_fish_config_with_empty_fonts() {
        use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
        
        // テスト用のカラースキーム（推奨フォントなし）
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };
        
        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };
        
        let font = FontConfig {
            weight: FontWeight::Medium,
            recommended_fonts: vec![],
        };
        
        // 設定を生成
        let config = generate_fish_config(&scheme, &font);
        
        // 基本的な構造が含まれることを確認
        assert!(config.contains("# === TWF Generated Config ==="));
        assert!(config.contains("# === End TWF Config ==="));
        
        // フォントウェイトは含まれるが、推奨フォントリストは含まれない
        assert!(config.contains("推奨フォントウェイト: Medium"));
        assert!(!config.contains("推奨フォント:"));
        
        // Fish形式の設定が含まれることを確認
        assert!(config.contains("set -x TWF_"));
    }
    
    #[test]
    fn test_fish_config_format_differs_from_bash() {
        use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
        
        // テスト用のカラースキームを作成
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };
        
        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };
        
        let font = FontConfig {
            weight: FontWeight::Normal,
            recommended_fonts: vec!["Fira Code".to_string()],
        };
        
        // Bash設定とFish設定を生成
        let bash_config = generate_bash_config(&scheme, &font);
        let fish_config = generate_fish_config(&scheme, &font);
        
        // Bash設定にはexportが含まれる
        assert!(bash_config.contains("export TWF_"));
        
        // Fish設定にはset -xが含まれる
        assert!(fish_config.contains("set -x TWF_"));
        
        // Fish設定にはexportが含まれない
        assert!(!fish_config.contains("export TWF_"));
        
        // Bash設定にはset -xが含まれない
        assert!(!bash_config.contains("set -x TWF_"));
    }
    
    #[test]
    fn test_generate_powershell_config() {
        use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
        
        // テスト用のカラースキームを作成
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };
        
        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };
        
        let font = FontConfig {
            weight: FontWeight::Normal,
            recommended_fonts: vec!["Cascadia Code".to_string(), "Consolas".to_string()],
        };
        
        // 設定を生成
        let config = generate_powershell_config(&scheme, &font);
        
        // 設定ブロックのヘッダーとフッターが含まれることを確認
        assert!(config.contains("# === TWF Generated Config ==="));
        assert!(config.contains("# === End TWF Config ==="));
        
        // PowerShell形式の環境変数設定（$env:）が含まれることを確認
        assert!(config.contains("$env:TWF_ANSI_BLACK"));
        assert!(config.contains("$env:TWF_ANSI_RED"));
        assert!(config.contains("$env:TWF_ANSI_GREEN"));
        assert!(config.contains("$env:TWF_ANSI_YELLOW"));
        assert!(config.contains("$env:TWF_ANSI_BLUE"));
        assert!(config.contains("$env:TWF_ANSI_MAGENTA"));
        assert!(config.contains("$env:TWF_ANSI_CYAN"));
        assert!(config.contains("$env:TWF_ANSI_WHITE"));
        
        // 明るいANSI色の環境変数が含まれることを確認
        assert!(config.contains("$env:TWF_ANSI_BRIGHT_BLACK"));
        assert!(config.contains("$env:TWF_ANSI_BRIGHT_RED"));
        assert!(config.contains("$env:TWF_ANSI_BRIGHT_GREEN"));
        assert!(config.contains("$env:TWF_ANSI_BRIGHT_YELLOW"));
        assert!(config.contains("$env:TWF_ANSI_BRIGHT_BLUE"));
        assert!(config.contains("$env:TWF_ANSI_BRIGHT_MAGENTA"));
        assert!(config.contains("$env:TWF_ANSI_BRIGHT_CYAN"));
        assert!(config.contains("$env:TWF_ANSI_BRIGHT_WHITE"));
        
        // フォアグラウンドとリセットシーケンスが含まれることを確認
        assert!(config.contains("$env:TWF_FOREGROUND"));
        assert!(config.contains("$env:TWF_RESET"));
        
        // フォント設定の推奨事項が含まれることを確認
        assert!(config.contains("推奨フォントウェイト: Normal"));
        assert!(config.contains("推奨フォント: Cascadia Code, Consolas"));
        
        // 使用例が含まれることを確認
        assert!(config.contains("使用例:"));
        assert!(config.contains("Write-Host"));
        
        // PowerShellのエスケープシーケンス形式が正しいことを確認（`e）
        assert!(config.contains("`e[38;2;"));
        assert!(config.contains("`e[0m"));
        
        // exportやset -xが含まれないことを確認
        assert!(!config.contains("export TWF_"));
        assert!(!config.contains("set -x TWF_"));
    }
    
    #[test]
    fn test_generate_powershell_config_with_empty_fonts() {
        use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
        
        // テスト用のカラースキーム（推奨フォントなし）
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };
        
        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };
        
        let font = FontConfig {
            weight: FontWeight::Light,
            recommended_fonts: vec![],
        };
        
        // 設定を生成
        let config = generate_powershell_config(&scheme, &font);
        
        // 基本的な構造が含まれることを確認
        assert!(config.contains("# === TWF Generated Config ==="));
        assert!(config.contains("# === End TWF Config ==="));
        
        // フォントウェイトは含まれるが、推奨フォントリストは含まれない
        assert!(config.contains("推奨フォントウェイト: Light"));
        assert!(!config.contains("推奨フォント:"));
        
        // PowerShell形式の設定が含まれることを確認
        assert!(config.contains("$env:TWF_"));
    }
    
    #[test]
    fn test_powershell_config_format_differs_from_bash_and_fish() {
        use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
        
        // テスト用のカラースキームを作成
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };
        
        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };
        
        let font = FontConfig {
            weight: FontWeight::Normal,
            recommended_fonts: vec!["Fira Code".to_string()],
        };
        
        // 各シェルの設定を生成
        let bash_config = generate_bash_config(&scheme, &font);
        let fish_config = generate_fish_config(&scheme, &font);
        let powershell_config = generate_powershell_config(&scheme, &font);
        
        // Bash設定にはexportが含まれる
        assert!(bash_config.contains("export TWF_"));
        
        // Fish設定にはset -xが含まれる
        assert!(fish_config.contains("set -x TWF_"));
        
        // PowerShell設定には$env:が含まれる
        assert!(powershell_config.contains("$env:TWF_"));
        
        // PowerShell設定にはexportやset -xが含まれない
        assert!(!powershell_config.contains("export TWF_"));
        assert!(!powershell_config.contains("set -x TWF_"));
        
        // PowerShellのエスケープシーケンスは`eを使用
        assert!(powershell_config.contains("`e["));
        
        // Bash/Fishのエスケープシーケンスは\\eを使用
        assert!(bash_config.contains("\\e["));
        assert!(fish_config.contains("\\e["));
        
        // PowerShellには\\eが含まれない
        assert!(!powershell_config.contains("\\e["));
    }
}

#[cfg(test)]
mod config_applier_tests {
    use super::*;
    use crate::models::{AnsiColors, ColorScheme, FontConfig, FontWeight, Lab, Rgb};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_applier_write_config() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join(".bashrc");
        let backup_dir = temp_dir.path().join("backups");

        // テスト用のカラースキームを作成
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };

        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };

        let font = FontConfig {
            weight: FontWeight::Normal,
            recommended_fonts: vec!["Fira Code".to_string()],
        };

        // 既存の設定ファイルを作成
        tokio::fs::write(&config_file, "# Original config\nexport PATH=/usr/local/bin:$PATH\n")
            .await
            .unwrap();

        // ConfigApplierを作成
        let applier = ConfigApplier::new(backup_dir);

        // 設定を書き込み
        applier
            .write_config(&config_file, &ShellType::Bash, &scheme, &font)
            .await
            .unwrap();

        // ファイルの内容を確認
        let content = tokio::fs::read_to_string(&config_file).await.unwrap();

        // 元の設定が残っていることを確認
        assert!(content.contains("# Original config"));
        assert!(content.contains("export PATH=/usr/local/bin:$PATH"));

        // TWF設定が追加されていることを確認
        assert!(content.contains("=== TWF Generated Config ==="));
        assert!(content.contains("export TWF_ANSI_RED="));
        assert!(content.contains("=== End TWF Config ==="));
    }

    #[tokio::test]
    async fn test_config_applier_removes_old_twf_block() {
        // テンポラリディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join(".bashrc");
        let backup_dir = temp_dir.path().join("backups");

        // 既存のTWFブロックを含む設定ファイルを作成
        let old_config = r#"# Original config
export PATH=/usr/local/bin:$PATH

# === TWF Generated Config ===
export TWF_ANSI_RED='\e[38;2;200;0;0m'
# === End TWF Config ===

# More config
alias ll='ls -la'
"#;
        tokio::fs::write(&config_file, old_config).await.unwrap();

        // テスト用のカラースキームを作成
        let ansi_colors = AnsiColors {
            black: Rgb::new(0, 0, 0),
            red: Rgb::new(255, 0, 0),
            green: Rgb::new(0, 255, 0),
            yellow: Rgb::new(255, 255, 0),
            blue: Rgb::new(0, 0, 255),
            magenta: Rgb::new(255, 0, 255),
            cyan: Rgb::new(0, 255, 255),
            white: Rgb::new(255, 255, 255),
            bright_black: Rgb::new(128, 128, 128),
            bright_red: Rgb::new(255, 128, 128),
            bright_green: Rgb::new(128, 255, 128),
            bright_yellow: Rgb::new(255, 255, 128),
            bright_blue: Rgb::new(128, 128, 255),
            bright_magenta: Rgb::new(255, 128, 255),
            bright_cyan: Rgb::new(128, 255, 255),
            bright_white: Rgb::new(255, 255, 255),
        };

        let scheme = ColorScheme {
            foreground: Rgb::new(240, 240, 240),
            background: Lab { l: 20.0, a: 0.0, b: 0.0 },
            ansi_colors,
            palette_256: None,
            supports_true_color: true,
        };

        let font = FontConfig {
            weight: FontWeight::Normal,
            recommended_fonts: vec![],
        };

        // ConfigApplierを作成
        let applier = ConfigApplier::new(backup_dir);

        // 設定を書き込み
        applier
            .write_config(&config_file, &ShellType::Bash, &scheme, &font)
            .await
            .unwrap();

        // ファイルの内容を確認
        let content = tokio::fs::read_to_string(&config_file).await.unwrap();

        // 元の設定が残っていることを確認
        assert!(content.contains("# Original config"));
        assert!(content.contains("# More config"));

        // 古いTWFブロックが削除されていることを確認
        assert!(!content.contains("200;0;0"));

        // 新しいTWF設定が追加されていることを確認
        assert!(content.contains("=== TWF Generated Config ==="));
        assert!(content.contains("255;0;0")); // 新しい赤色の値
    }
}
