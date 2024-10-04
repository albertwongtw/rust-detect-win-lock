use chrono::Local;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{
            LibraryLoader::*,
            Power::*,
            RemoteDesktop::*,
            SystemServices::*
        },
        UI::WindowsAndMessaging::*
    }
};

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        let window_class = s!("window");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: window_class,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };
        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        let _ = CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("rust_detect_win_lock"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None
        );

        let mut message = MSG::default();
        while GetMessageA(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageA(&message);
        }

        // drop(guard);

        Ok(())
    }
}

static mut WINDOW_OFFSET_X: i32 = 0;
static mut WINDOW_OFFSET_Y: i32 = 0;
static mut TEXTS: Vec<String> = Vec::new();
extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                _ = RegisterPowerSettingNotification(hwnd, &GUID_SESSION_USER_PRESENCE, DEVICE_NOTIFY_WINDOW_HANDLE);
                _ = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_ALL_SESSIONS);
                WINDOW_OFFSET_X = LOWORD(GetDialogBaseUnits());
                WINDOW_OFFSET_Y = HIWORD(GetDialogBaseUnits());
                LRESULT(0)
            }
            WM_POWERBROADCAST => {
                println!("WM_POWERBROADCAST: {}", wparam.0);
                LRESULT(0)
            }
            WM_WTSSESSION_CHANGE => {
                println!("WM_WTSSESSION_CHANGE: {}", wparam.0);
                if wparam.0 == 7 || wparam.0 == 8 {
                    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
                    let message = format!("{now} {text}", text = if wparam.0 == 7 { "locked" } else { "unlocked" });
                    TEXTS.push(message.clone());
                    println!("{}", message);
                    _ = InvalidateRect(hwnd, None, true);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                println!("WM_PAINT");
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                let mut y = 10;
                for text in TEXTS.clone() {
                    _ = TextOutA(hdc, 10, y, text.as_bytes());
                    y += 20;
                }
                _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTORY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(hwnd, message, wparam, lparam)
        }
    }
}

#[allow(non_snake_case)]
fn LOWORD(l: i32) -> i32 {
    l & 0xffff
}

#[allow(non_snake_case)]
fn HIWORD(l: i32) -> i32 {
    (l >> 16) & 0xffff
}