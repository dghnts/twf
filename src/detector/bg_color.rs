// 背景色検出（OSC 11）

use crate::models::{Result, Rgb};
use std::io::{self, Read, Write};
use std::time::Duration;

/// 背景色検出器
pub struct BgColorDetector {
    timeout: Duration,
}

impl BgColorDetector {
    /// 新しい背景色検出器を作成
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// ターミナルから背景色を取得
    /// OSC 11エスケープシーケンスを送信し、レスポンスをパース
    pub fn detect_background_color(&self) -> Result<Option<Rgb>> {
        // 標準入力/出力が端末に接続されているか確認
        if !atty::is(atty::Stream::Stdin) || !atty::is(atty::Stream::Stdout) {
            return Ok(None);
        }

        // 1. 標準入力を非カノニカルモードに設定
        let original_termios = match setup_raw_mode() {
            Ok(termios) => termios,
            Err(_) => return Ok(None), // エラーの場合はNoneを返す
        };

        // 2. OSC 11クエリを送信
        if let Err(_) = send_osc11_query() {
            let _ = restore_termios(&original_termios);
            return Ok(None);
        }

        // 3. タイムアウト付きでレスポンスを待機
        let response = match read_with_timeout(self.timeout) {
            Ok(resp) => resp,
            Err(_) => {
                let _ = restore_termios(&original_termios);
                return Ok(None);
            }
        };

        // 4. 元のターミナル設定に戻す
        let _ = restore_termios(&original_termios);

        // 5. レスポンスをパース
        if let Some(rgb) = parse_osc11_response(&response) {
            Ok(Some(rgb))
        } else {
            Ok(None)
        }
    }
}

/// OSC 11クエリを送信
fn send_osc11_query() -> io::Result<()> {
    let mut stdout = io::stdout();
    // OSC 11クエリ: "\x1b]11;?\x07"
    stdout.write_all(b"\x1b]11;?\x07")?;
    stdout.flush()?;
    Ok(())
}

/// タイムアウト付きでレスポンスを読み取り
fn read_with_timeout(timeout: Duration) -> io::Result<String> {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    // 別スレッドで標準入力から読み取り
    thread::spawn(move || {
        let mut stdin = io::stdin();
        let mut buffer = [0u8; 256];
        let mut response = String::new();

        // レスポンスを読み取る（最大256バイト）
        loop {
            match stdin.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    // OSC 11レスポンスの終端（BEL: \x07 または ST: \x1b\\）を検出
                    if response.contains('\x07') || response.contains("\x1b\\") {
                        let _ = tx.send(response);
                        break;
                    }
                }
                _ => break,
            }
        }
    });

    // タイムアウト付きで待機
    rx.recv_timeout(timeout)
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "タイムアウト"))
}

/// OSC 11レスポンスをパースしてRGB値を抽出
/// レスポンス形式: "\x1b]11;rgb:RRRR/GGGG/BBBB\x07" または "\x1b]11;rgb:RRRR/GGGG/BBBB\x1b\\"
pub fn parse_osc11_response(response: &str) -> Option<Rgb> {
    // レスポンスから "rgb:RRRR/GGGG/BBBB" 部分を抽出
    let start_marker = "\x1b]11;rgb:";
    let start_idx = response.find(start_marker)?;
    let rgb_start = start_idx + start_marker.len();

    // 終端マーカーを探す（BEL: \x07 または ST: \x1b\\）
    let rgb_part = &response[rgb_start..];
    let end_idx = rgb_part
        .find('\x07')
        .or_else(|| rgb_part.find("\x1b\\"))
        .unwrap_or(rgb_part.len());

    let rgb_str = &rgb_part[..end_idx];

    // "RRRR/GGGG/BBBB" をパース
    let parts: Vec<&str> = rgb_str.split('/').collect();
    if parts.len() != 3 {
        return None;
    }

    // 16進数をパース（4桁 → 8ビット）
    let r = u16::from_str_radix(parts[0], 16).ok()? / 256;
    let g = u16::from_str_radix(parts[1], 16).ok()? / 256;
    let b = u16::from_str_radix(parts[2], 16).ok()? / 256;

    Some(Rgb::new(r as u8, g as u8, b as u8))
}

// プラットフォーム固有の実装

#[cfg(unix)]
mod unix_impl {
    use std::io;
    use std::os::unix::io::AsRawFd;

    pub struct Termios {
        termios: libc::termios,
    }

    pub fn setup_raw_mode() -> io::Result<Termios> {
        let stdin_fd = io::stdin().as_raw_fd();
        let termios = unsafe {
            let mut termios = std::mem::zeroed();
            if libc::tcgetattr(stdin_fd, &mut termios) != 0 {
                return Err(io::Error::last_os_error());
            }
            termios
        };

        let original = Termios { termios };

        // 非カノニカルモードに設定
        unsafe {
            let mut new_termios = termios;
            libc::cfmakeraw(&mut new_termios);
            new_termios.c_cc[libc::VMIN] = 0;
            new_termios.c_cc[libc::VTIME] = 0;

            if libc::tcsetattr(stdin_fd, libc::TCSANOW, &new_termios) != 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(original)
    }

    pub fn restore_termios(termios: &Termios) -> io::Result<()> {
        let stdin_fd = io::stdin().as_raw_fd();
        unsafe {
            if libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios.termios) != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
mod windows_impl {
    use std::io;
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_INPUT_HANDLE;
    use winapi::um::wincon::{ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT};

    pub struct Termios {
        mode: u32,
    }

    pub fn setup_raw_mode() -> io::Result<Termios> {
        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                return Err(io::Error::last_os_error());
            }

            let original = Termios { mode };

            // 非カノニカルモードに設定
            let new_mode = mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
            if SetConsoleMode(handle, new_mode) == 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(original)
        }
    }

    pub fn restore_termios(termios: &Termios) -> io::Result<()> {
        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            if SetConsoleMode(handle, termios.mode) == 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(())
        }
    }
}

// プラットフォーム固有の関数をエクスポート
#[cfg(unix)]
use unix_impl::{restore_termios, setup_raw_mode};

#[cfg(windows)]
use windows_impl::{restore_termios, setup_raw_mode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_osc11_response_with_bel() {
        // BEL終端のレスポンス
        let response = "\x1b]11;rgb:1e1e/1e1e/1e1e\x07";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_some());
        let rgb = rgb.unwrap();
        assert_eq!(rgb.r, 30); // 0x1e1e / 256 = 30
        assert_eq!(rgb.g, 30);
        assert_eq!(rgb.b, 30);
    }

    #[test]
    fn test_parse_osc11_response_with_st() {
        // ST終端のレスポンス
        let response = "\x1b]11;rgb:ffff/ffff/ffff\x1b\\";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_some());
        let rgb = rgb.unwrap();
        assert_eq!(rgb.r, 255); // 0xffff / 256 = 255
        assert_eq!(rgb.g, 255);
        assert_eq!(rgb.b, 255);
    }

    #[test]
    fn test_parse_osc11_response_black() {
        let response = "\x1b]11;rgb:0000/0000/0000\x07";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_some());
        let rgb = rgb.unwrap();
        assert_eq!(rgb.r, 0);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_parse_osc11_response_white() {
        let response = "\x1b]11;rgb:ffff/ffff/ffff\x07";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_some());
        let rgb = rgb.unwrap();
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 255);
        assert_eq!(rgb.b, 255);
    }

    #[test]
    fn test_parse_osc11_response_custom_color() {
        let response = "\x1b]11;rgb:8080/4040/c0c0\x07";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_some());
        let rgb = rgb.unwrap();
        assert_eq!(rgb.r, 128); // 0x8080 / 256 = 128
        assert_eq!(rgb.g, 64);  // 0x4040 / 256 = 64
        assert_eq!(rgb.b, 192); // 0xc0c0 / 256 = 192
    }

    #[test]
    fn test_parse_osc11_response_invalid_format() {
        // 不正な形式
        let response = "\x1b]11;invalid\x07";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_none());
    }

    #[test]
    fn test_parse_osc11_response_incomplete() {
        // 不完全なレスポンス
        let response = "\x1b]11;rgb:ffff/ffff";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_none());
    }

    #[test]
    fn test_parse_osc11_response_wrong_component_count() {
        // コンポーネント数が不正
        let response = "\x1b]11;rgb:ffff/ffff\x07";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_none());
    }

    #[test]
    fn test_parse_osc11_response_with_extra_data() {
        // 余分なデータがある場合
        let response = "some prefix\x1b]11;rgb:8080/8080/8080\x07some suffix";
        let rgb = parse_osc11_response(response);
        assert!(rgb.is_some());
        let rgb = rgb.unwrap();
        assert_eq!(rgb.r, 128);
        assert_eq!(rgb.g, 128);
        assert_eq!(rgb.b, 128);
    }
}
