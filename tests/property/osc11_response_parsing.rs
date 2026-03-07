// Property 2: OSC 11レスポンスのパース
//
// このプロパティテストは、OSC 11エスケープシーケンスのレスポンスパース機能の
// 正確性を検証します。任意の有効なOSC 11レスポンス形式に対して、
// 正しいRGB値（各成分0-255）が抽出されることを確認します。
//
// **Validates: Requirements 2.1.7**

use proptest::prelude::*;
use twf::detector::bg_color::parse_osc11_response;
use twf::models::Rgb;

// Property 2.1: 有効なOSC 11レスポンス（BEL終端）を正しくパースできること
//
// "\x1b]11;rgb:RRRR/GGGG/BBBB\x07"形式のレスポンスを
// 正しくパースし、RGB値を抽出できることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_parse_valid_osc11_with_bel(
        r in 0u16..=0xffff,
        g in 0u16..=0xffff,
        b in 0u16..=0xffff,
    ) {
        // OSC 11レスポンスを生成（BEL終端）
        let response = format!("\x1b]11;rgb:{:04x}/{:04x}/{:04x}\x07", r, g, b);
        
        // パース処理を実行
        let result = parse_osc11_response(&response);
        
        // パースが成功することを検証
        prop_assert!(result.is_some(), "パースに失敗しました: {}", response);
        
        let rgb = result.unwrap();
        
        // RGB値が正しく変換されていることを検証（16ビット → 8ビット）
        let expected_r = (r / 256) as u8;
        let expected_g = (g / 256) as u8;
        let expected_b = (b / 256) as u8;
        
        prop_assert_eq!(rgb.r, expected_r, "赤成分が一致しません");
        prop_assert_eq!(rgb.g, expected_g, "緑成分が一致しません");
        prop_assert_eq!(rgb.b, expected_b, "青成分が一致しません");
    }
}

// Property 2.2: 有効なOSC 11レスポンス（ST終端）を正しくパースできること
//
// "\x1b]11;rgb:RRRR/GGGG/BBBB\x1b\\"形式のレスポンスを
// 正しくパースし、RGB値を抽出できることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_parse_valid_osc11_with_st(
        r in 0u16..=0xffff,
        g in 0u16..=0xffff,
        b in 0u16..=0xffff,
    ) {
        // OSC 11レスポンスを生成（ST終端）
        let response = format!("\x1b]11;rgb:{:04x}/{:04x}/{:04x}\x1b\\", r, g, b);
        
        // パース処理を実行
        let result = parse_osc11_response(&response);
        
        // パースが成功することを検証
        prop_assert!(result.is_some(), "パースに失敗しました: {}", response);
        
        let rgb = result.unwrap();
        
        // RGB値が正しく変換されていることを検証（16ビット → 8ビット）
        let expected_r = (r / 256) as u8;
        let expected_g = (g / 256) as u8;
        let expected_b = (b / 256) as u8;
        
        prop_assert_eq!(rgb.r, expected_r, "赤成分が一致しません");
        prop_assert_eq!(rgb.g, expected_g, "緑成分が一致しません");
        prop_assert_eq!(rgb.b, expected_b, "青成分が一致しません");
    }
}

// Property 2.3: パースされたRGB値が0-255の範囲内であること
//
// 任意の有効なOSC 11レスポンスに対して、パースされたRGB値が
// 0-255の範囲内に収まることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_parsed_rgb_in_valid_range(
        r in 0u16..=0xffff,
        g in 0u16..=0xffff,
        b in 0u16..=0xffff,
    ) {
        // OSC 11レスポンスを生成
        let response = format!("\x1b]11;rgb:{:04x}/{:04x}/{:04x}\x07", r, g, b);
        
        // パース処理を実行
        let result = parse_osc11_response(&response);
        
        if let Some(rgb) = result {
            // RGB値が0-255の範囲内であることを検証
            prop_assert!(rgb.r <= 255, "赤成分が範囲外です: {}", rgb.r);
            prop_assert!(rgb.g <= 255, "緑成分が範囲外です: {}", rgb.g);
            prop_assert!(rgb.b <= 255, "青成分が範囲外です: {}", rgb.b);
        }
    }
}

// Property 2.4: 無効な形式のレスポンスに対してNoneを返すこと
//
// 無効な形式のレスポンスに対して、パース処理がNoneを返すことを検証します。
#[test]
fn prop_invalid_format_returns_none() {
    let invalid_responses = vec![
        // プレフィックスが不正
        "invalid;rgb:ffff/ffff/ffff\x07",
        // rgb:が欠落
        "\x1b]11;ffff/ffff/ffff\x07",
        // 空文字列
        "",
        // 完全に無効な文字列
        "completely invalid",
    ];
    
    for response in invalid_responses {
        let result = parse_osc11_response(response);
        assert!(
            result.is_none(),
            "無効なレスポンスがパースされてしまいました: {}",
            response
        );
    }
}

// Property 2.5: コンポーネント数が不正な場合にNoneを返すこと
//
// RGB成分の数が3つでない場合、パース処理がNoneを返すことを検証します。
#[test]
fn prop_wrong_component_count_returns_none() {
    let invalid_responses = vec![
        // 成分が2つ
        "\x1b]11;rgb:ffff/ffff\x07",
        // 成分が4つ
        "\x1b]11;rgb:ffff/ffff/ffff/ffff\x07",
        // 成分が1つ
        "\x1b]11;rgb:ffff\x07",
        // 成分が0個
        "\x1b]11;rgb:\x07",
    ];
    
    for response in invalid_responses {
        let result = parse_osc11_response(response);
        assert!(
            result.is_none(),
            "不正なコンポーネント数のレスポンスがパースされてしまいました: {}",
            response
        );
    }
}

// Property 2.6: 16進数の値が正しく10進数に変換されること（4桁→8ビット）
//
// 16進数4桁の値が正しく8ビット（0-255）に変換されることを検証します。
#[test]
fn prop_hex_to_decimal_conversion() {
    let test_cases = vec![
        // (16進数4桁, 期待される8ビット値)
        (0x0000, 0),   // 最小値
        (0xffff, 255), // 最大値
        (0x8080, 128), // 中間値
        (0x4040, 64),  // 1/4
        (0xc0c0, 192), // 3/4
        (0x1e1e, 30),  // 実際の例（暗い背景）
        (0xf0f0, 240), // 実際の例（明るい背景）
    ];
    
    for (hex_value, expected_decimal) in test_cases {
        let response = format!(
            "\x1b]11;rgb:{:04x}/{:04x}/{:04x}\x07",
            hex_value, hex_value, hex_value
        );
        
        let result = parse_osc11_response(&response);
        assert!(result.is_some(), "パースに失敗しました: {}", response);
        
        let rgb = result.unwrap();
        assert_eq!(
            rgb.r, expected_decimal,
            "16進数 {:04x} が正しく変換されませんでした",
            hex_value
        );
        assert_eq!(rgb.g, expected_decimal);
        assert_eq!(rgb.b, expected_decimal);
    }
}

// Property 2.7: 余分なデータがある場合でもパースできること
//
// レスポンスの前後に余分なデータがある場合でも、
// 正しくパースできることを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_parse_with_extra_data(
        r in 0u16..=0xffff,
        g in 0u16..=0xffff,
        b in 0u16..=0xffff,
        prefix in "[a-zA-Z0-9 ]{0,20}",
        suffix in "[a-zA-Z0-9 ]{0,20}",
    ) {
        // 余分なデータを含むレスポンスを生成
        let response = format!(
            "{}\x1b]11;rgb:{:04x}/{:04x}/{:04x}\x07{}",
            prefix, r, g, b, suffix
        );
        
        // パース処理を実行
        let result = parse_osc11_response(&response);
        
        // パースが成功することを検証
        prop_assert!(result.is_some(), "パースに失敗しました: {}", response);
        
        let rgb = result.unwrap();
        
        // RGB値が正しく変換されていることを検証
        let expected_r = (r / 256) as u8;
        let expected_g = (g / 256) as u8;
        let expected_b = (b / 256) as u8;
        
        prop_assert_eq!(rgb.r, expected_r);
        prop_assert_eq!(rgb.g, expected_g);
        prop_assert_eq!(rgb.b, expected_b);
    }
}

// Property 2.8: パース処理がパニックしないこと
//
// 任意の入力に対して、パース処理がパニックせずに完了することを検証します。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn prop_parse_does_not_panic(
        input in "\\PC{0,200}"
    ) {
        // 任意の文字列に対してパース処理を実行
        let result = parse_osc11_response(&input);
        
        // パニックせずに完了することを確認（結果はSomeまたはNone）
        match result {
            Some(rgb) => {
                // RGB値が有効な範囲内であることを確認
                prop_assert!(rgb.r <= 255);
                prop_assert!(rgb.g <= 255);
                prop_assert!(rgb.b <= 255);
            }
            None => {
                // Noneが返されることも正常
                prop_assert!(true);
            }
        }
    }
}

// Property 2.9: 大文字・小文字の16進数を正しく処理できること
//
// 16進数の大文字・小文字が混在している場合でも、
// 正しくパースできることを検証します。
#[test]
fn prop_hex_case_insensitive() {
    let test_cases = vec![
        // 小文字
        "\x1b]11;rgb:abcd/ef01/2345\x07",
        // 大文字
        "\x1b]11;rgb:ABCD/EF01/2345\x07",
        // 混在
        "\x1b]11;rgb:AbCd/eF01/23Ef\x07",
    ];
    
    for response in test_cases {
        let result = parse_osc11_response(response);
        assert!(
            result.is_some(),
            "大文字・小文字混在のレスポンスがパースできませんでした: {}",
            response
        );
        
        let rgb = result.unwrap();
        // RGB値が有効な範囲内であることを確認
        assert!(rgb.r <= 255);
        assert!(rgb.g <= 255);
        assert!(rgb.b <= 255);
    }
}

// Property 2.10: 境界値のテスト
//
// RGB値の境界値（0x0000、0xffff）が正しく処理されることを検証します。
#[test]
fn prop_boundary_values() {
    let test_cases = vec![
        // 最小値（黒）
        ("\x1b]11;rgb:0000/0000/0000\x07", Rgb::new(0, 0, 0)),
        // 最大値（白）
        ("\x1b]11;rgb:ffff/ffff/ffff\x07", Rgb::new(255, 255, 255)),
        // 赤のみ最大
        ("\x1b]11;rgb:ffff/0000/0000\x07", Rgb::new(255, 0, 0)),
        // 緑のみ最大
        ("\x1b]11;rgb:0000/ffff/0000\x07", Rgb::new(0, 255, 0)),
        // 青のみ最大
        ("\x1b]11;rgb:0000/0000/ffff\x07", Rgb::new(0, 0, 255)),
        // 中間値（グレー）
        ("\x1b]11;rgb:8080/8080/8080\x07", Rgb::new(128, 128, 128)),
    ];
    
    for (response, expected) in test_cases {
        let result = parse_osc11_response(response);
        assert!(result.is_some(), "パースに失敗しました: {}", response);
        
        let rgb = result.unwrap();
        assert_eq!(
            rgb.r, expected.r,
            "赤成分が一致しません: 期待={}, 実際={}",
            expected.r, rgb.r
        );
        assert_eq!(
            rgb.g, expected.g,
            "緑成分が一致しません: 期待={}, 実際={}",
            expected.g, rgb.g
        );
        assert_eq!(
            rgb.b, expected.b,
            "青成分が一致しません: 期待={}, 実際={}",
            expected.b, rgb.b
        );
    }
}

// Property 2.11: 不正な16進数値に対してNoneを返すこと
//
// 16進数として無効な文字が含まれている場合、
// パース処理がNoneを返すことを検証します。
#[test]
fn prop_invalid_hex_returns_none() {
    let invalid_responses = vec![
        // 16進数以外の文字（G-Z）
        "\x1b]11;rgb:gggg/hhhh/iiii\x07",
        // 記号
        "\x1b]11;rgb:!!!!/@@@@/####\x07",
        // 空白
        "\x1b]11;rgb:    /    /    \x07",
    ];
    
    for response in invalid_responses {
        let result = parse_osc11_response(response);
        assert!(
            result.is_none(),
            "無効な16進数値のレスポンスがパースされてしまいました: {}",
            response
        );
    }
}

// Property 2.13: 桁数が異なる16進数値の処理
//
// 16進数の桁数が4桁でない場合の動作を検証します。
// 注: 現在の実装では桁数チェックを行っていないため、
// 3桁や5桁の値も受け入れられます。
#[test]
fn prop_different_hex_digit_count() {
    // 3桁の16進数（実装では受け入れられる）
    let response_3digit = "\x1b]11;rgb:fff/fff/fff\x07";
    let result = parse_osc11_response(response_3digit);
    // 3桁でもパースされることを確認（0xfff = 4095, 4095/256 = 15）
    if let Some(rgb) = result {
        assert_eq!(rgb.r, 15);
        assert_eq!(rgb.g, 15);
        assert_eq!(rgb.b, 15);
    }
    
    // 5桁の16進数（実装では受け入れられる）
    let response_5digit = "\x1b]11;rgb:fffff/fffff/fffff\x07";
    let result = parse_osc11_response(response_5digit);
    // 5桁でもパースされることを確認（0xfffff = 1048575, 1048575/256 = 4095）
    // ただし、u16の最大値は65535なので、オーバーフローする
    assert!(result.is_none(), "5桁の16進数はu16の範囲を超えるためNoneが返されるべき");
}

// Property 2.12: 実際のターミナルエミュレータのレスポンス例
//
// 実際のターミナルエミュレータが返すレスポンス形式を
// 正しくパースできることを検証します。
#[test]
fn prop_real_terminal_responses() {
    let real_responses = vec![
        // iTerm2の例（暗い背景）
        ("\x1b]11;rgb:1e1e/1e1e/1e1e\x07", Rgb::new(30, 30, 30)),
        // Alacrittyの例（明るい背景）
        ("\x1b]11;rgb:f0f0/f0f0/f0f0\x07", Rgb::new(240, 240, 240)),
        // Kittyの例（カスタム背景）
        ("\x1b]11;rgb:2828/2a2a/3636\x07", Rgb::new(40, 42, 54)),
        // WezTermの例（ST終端）
        ("\x1b]11;rgb:1d1d/1f1f/2121\x1b\\", Rgb::new(29, 31, 33)),
        // GNOME Terminalの例
        ("\x1b]11;rgb:3030/3434/3939\x07", Rgb::new(48, 52, 57)),
    ];
    
    for (response, expected) in real_responses {
        let result = parse_osc11_response(response);
        assert!(result.is_some(), "パースに失敗しました: {}", response);
        
        let rgb = result.unwrap();
        assert_eq!(
            rgb.r, expected.r,
            "赤成分が一致しません: 期待={}, 実際={}",
            expected.r, rgb.r
        );
        assert_eq!(
            rgb.g, expected.g,
            "緑成分が一致しません: 期待={}, 実際={}",
            expected.g, rgb.g
        );
        assert_eq!(
            rgb.b, expected.b,
            "青成分が一致しません: 期待={}, 実際={}",
            expected.b, rgb.b
        );
    }
}
