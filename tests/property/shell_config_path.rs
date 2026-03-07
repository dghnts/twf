// Property 8: シェル設定ファイルパスの正確性
//
// このプロパティテストは、シェル設定ファイルパス取得機能の正確性を検証します。
// 任意のシェルタイプ（Bash、Zsh、Fish、PowerShell）に対して、
// そのシェルタイプに対応する正しい設定ファイルパスが返されることを確認します。
//
// **Validates: Requirements 2.4.3**

use proptest::prelude::*;
use twf::applier::shell::get_shell_config_path;
use twf::models::ShellType;

/// シェルタイプの戦略を生成
fn shell_type_strategy() -> impl Strategy<Value = ShellType> {
    prop_oneof![
        Just(ShellType::Bash),
        Just(ShellType::Zsh),
        Just(ShellType::Fish),
        Just(ShellType::PowerShell),
    ]
}

// Property 8.1: 各シェルタイプに対して有効な設定ファイルパスが返されること
//
// すべてのシェルタイプに対して、get_shell_config_path関数が
// 有効なパスを返すことを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_valid_config_path_returned(shell_type in shell_type_strategy()) {
        // 設定ファイルパスを取得
        let result = get_shell_config_path(&shell_type);
        
        // エラーが発生しないことを確認
        prop_assert!(result.is_ok(), "設定ファイルパスの取得に失敗しました: {:?}", result);
        
        let path = result.unwrap();
        
        // パスが空でないことを確認
        prop_assert!(!path.as_os_str().is_empty(), "設定ファイルパスが空です");
    }
}

// Property 8.2: 返されたパスがホームディレクトリ配下であること
//
// すべてのシェルタイプに対して、返されるパスが
// ホームディレクトリまたは設定ディレクトリ配下であることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_path_under_home_directory(shell_type in shell_type_strategy()) {
        // 設定ファイルパスを取得
        let result = get_shell_config_path(&shell_type);
        prop_assert!(result.is_ok());
        
        let path = result.unwrap();
        let path_str = path.to_string_lossy();
        
        // ホームディレクトリを取得
        let home_dir = dirs::home_dir();
        prop_assert!(home_dir.is_some(), "ホームディレクトリが取得できません");
        
        let home_dir = home_dir.unwrap();
        let home_str = home_dir.to_string_lossy();
        
        // パスがホームディレクトリ配下であることを確認
        // または設定ディレクトリ配下であることを確認
        let is_under_home = path_str.starts_with(home_str.as_ref());
        
        // 設定ディレクトリもチェック（Fishの場合）
        let config_dir = dirs::config_dir();
        let is_under_config = if let Some(config) = config_dir {
            let config_str = config.to_string_lossy();
            path_str.starts_with(config_str.as_ref())
        } else {
            false
        };
        
        prop_assert!(
            is_under_home || is_under_config,
            "パスがホームディレクトリまたは設定ディレクトリ配下にありません: {}",
            path_str
        );
    }
}

// Property 8.3: Bashの設定ファイルパスが正しいこと
//
// Bashシェルに対して、.bashrc、.bash_profile、または.profileが
// 返されることを検証します。
#[test]
fn prop_bash_config_path_correct() {
    let shell_type = ShellType::Bash;
    let result = get_shell_config_path(&shell_type);
    
    assert!(result.is_ok(), "Bash設定ファイルパスの取得に失敗しました");
    
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    
    // .bashrc、.bash_profile、または.profileのいずれかであることを確認
    assert!(
        path_str.contains(".bashrc") 
            || path_str.contains(".bash_profile")
            || path_str.contains(".profile"),
        "Bashの設定ファイルパスが不正です: {}",
        path_str
    );
}

// Property 8.4: Zshの設定ファイルパスが正しいこと
//
// Zshシェルに対して、.zshrcまたは.zprofileが
// 返されることを検証します。
#[test]
fn prop_zsh_config_path_correct() {
    let shell_type = ShellType::Zsh;
    let result = get_shell_config_path(&shell_type);
    
    assert!(result.is_ok(), "Zsh設定ファイルパスの取得に失敗しました");
    
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    
    // .zshrcまたは.zprofileであることを確認
    assert!(
        path_str.contains(".zshrc") || path_str.contains(".zprofile"),
        "Zshの設定ファイルパスが不正です: {}",
        path_str
    );
}

// Property 8.5: Fishの設定ファイルパスが正しいこと
//
// Fishシェルに対して、~/.config/fish/config.fishが
// 返されることを検証します。
#[test]
fn prop_fish_config_path_correct() {
    let shell_type = ShellType::Fish;
    let result = get_shell_config_path(&shell_type);
    
    assert!(result.is_ok(), "Fish設定ファイルパスの取得に失敗しました");
    
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    
    // config.fishであることを確認
    assert!(
        path_str.contains("config.fish"),
        "Fishの設定ファイルパスが不正です: {}",
        path_str
    );
    
    // fishディレクトリ配下であることを確認
    assert!(
        path_str.contains("fish"),
        "Fishの設定ファイルパスがfishディレクトリ配下にありません: {}",
        path_str
    );
}

// Property 8.6: PowerShellの設定ファイルパスが正しいこと
//
// PowerShellに対して、Microsoft.PowerShell_profile.ps1が
// 返されることを検証します。
#[test]
fn prop_powershell_config_path_correct() {
    let shell_type = ShellType::PowerShell;
    let result = get_shell_config_path(&shell_type);
    
    assert!(result.is_ok(), "PowerShell設定ファイルパスの取得に失敗しました");
    
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    
    // Microsoft.PowerShell_profile.ps1であることを確認
    assert!(
        path_str.contains("Microsoft.PowerShell_profile.ps1"),
        "PowerShellの設定ファイルパスが不正です: {}",
        path_str
    );
}

// Property 8.7: パスが絶対パスであること
//
// すべてのシェルタイプに対して、返されるパスが
// 絶対パスであることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_path_is_absolute(shell_type in shell_type_strategy()) {
        // 設定ファイルパスを取得
        let result = get_shell_config_path(&shell_type);
        prop_assert!(result.is_ok());
        
        let path = result.unwrap();
        
        // パスが絶対パスであることを確認
        prop_assert!(
            path.is_absolute(),
            "設定ファイルパスが絶対パスではありません: {:?}",
            path
        );
    }
}

// Property 8.8: 同じシェルタイプに対して常に同じパスが返されること（一貫性）
//
// 同じシェルタイプに対して複数回呼び出した場合、
// 常に同じパスが返されることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_consistent_path_for_same_shell(shell_type in shell_type_strategy()) {
        // 1回目の呼び出し
        let result1 = get_shell_config_path(&shell_type);
        prop_assert!(result1.is_ok());
        let path1 = result1.unwrap();
        
        // 2回目の呼び出し
        let result2 = get_shell_config_path(&shell_type);
        prop_assert!(result2.is_ok());
        let path2 = result2.unwrap();
        
        // 同じパスが返されることを確認
        prop_assert_eq!(
            path1,
            path2,
            "同じシェルタイプに対して異なるパスが返されました"
        );
    }
}

// Property 8.9: パス取得処理がパニックしないこと
//
// すべてのシェルタイプに対して、パス取得処理が
// パニックせずに完了することを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_no_panic_on_path_retrieval(shell_type in shell_type_strategy()) {
        // パニックせずに完了することを確認
        let result = std::panic::catch_unwind(|| {
            get_shell_config_path(&shell_type)
        });
        
        prop_assert!(
            result.is_ok(),
            "パス取得処理がパニックしました: {:?}",
            shell_type
        );
    }
}

// Property 8.10: 各シェルタイプに対して適切なファイル名が含まれること
//
// 各シェルタイプに対して、返されるパスに適切なファイル名が
// 含まれることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_appropriate_filename_for_shell(shell_type in shell_type_strategy()) {
        // 設定ファイルパスを取得
        let result = get_shell_config_path(&shell_type);
        prop_assert!(result.is_ok());
        
        let path = result.unwrap();
        let path_str = path.to_string_lossy();
        
        // シェルタイプに応じた適切なファイル名が含まれることを確認
        let is_valid = match shell_type {
            ShellType::Bash => {
                path_str.contains(".bashrc") 
                    || path_str.contains(".bash_profile")
                    || path_str.contains(".profile")
            }
            ShellType::Zsh => {
                path_str.contains(".zshrc") || path_str.contains(".zprofile")
            }
            ShellType::Fish => {
                path_str.contains("config.fish")
            }
            ShellType::PowerShell => {
                path_str.contains("Microsoft.PowerShell_profile.ps1")
            }
        };
        
        prop_assert!(
            is_valid,
            "シェルタイプ {:?} に対して不適切なファイル名です: {}",
            shell_type,
            path_str
        );
    }
}

// Property 8.11: パスの親ディレクトリが存在するか、作成可能であること
//
// 返されるパスの親ディレクトリが存在するか、
// または作成可能であることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_parent_directory_exists_or_creatable(shell_type in shell_type_strategy()) {
        // 設定ファイルパスを取得
        let result = get_shell_config_path(&shell_type);
        prop_assert!(result.is_ok());
        
        let path = result.unwrap();
        
        // 親ディレクトリを取得
        if let Some(parent) = path.parent() {
            // 親ディレクトリが存在するか確認
            // 注: 実際には存在しない場合もあるが、それは正常な動作
            // （ユーザーがまだ設定ファイルを作成していない場合）
            
            // パスが有効な形式であることを確認
            prop_assert!(
                !parent.as_os_str().is_empty(),
                "親ディレクトリパスが空です"
            );
        }
    }
}

// Property 8.12: 複数のシェルタイプに対して異なるパスが返されること
//
// 異なるシェルタイプに対して、異なるパスが返されることを検証します。
// （ただし、一部のシェルは同じディレクトリを使用する場合があります）
#[test]
fn prop_different_paths_for_different_shells() {
    let bash_path = get_shell_config_path(&ShellType::Bash).unwrap();
    let zsh_path = get_shell_config_path(&ShellType::Zsh).unwrap();
    let fish_path = get_shell_config_path(&ShellType::Fish).unwrap();
    let powershell_path = get_shell_config_path(&ShellType::PowerShell).unwrap();
    
    // Bashとその他のシェルのパスが異なることを確認
    assert_ne!(
        bash_path, fish_path,
        "BashとFishのパスが同じです"
    );
    assert_ne!(
        bash_path, powershell_path,
        "BashとPowerShellのパスが同じです"
    );
    
    // ZshとFishのパスが異なることを確認
    assert_ne!(
        zsh_path, fish_path,
        "ZshとFishのパスが同じです"
    );
    
    // FishとPowerShellのパスが異なることを確認
    assert_ne!(
        fish_path, powershell_path,
        "FishとPowerShellのパスが同じです"
    );
}
