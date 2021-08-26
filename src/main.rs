mod bindings { windows::include_bindings!(); }

use bindings::Windows::Win32::{
    UI::WindowsAndMessaging::*,
    Foundation::*,
    System::LibraryLoader::GetModuleHandleA,
    Graphics::Gdi::*,
    UI::Controls::*
};
use std::ptr;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::{
    Instant,
    Duration
};

const VIDEO_PATH: &'static str = "demo2.mp4";
const FRAME_RATE: u64 = 30;
static mut MONITOR_WIDTH: i32 = 1280;
static mut MONITOR_HEIGHT: i32 = 720;

fn main() {
    unsafe {
        let progman = FindWindowW("Progman", PWSTR::NULL);

        SendMessageTimeoutW(progman, 0x052C, WPARAM::NULL, LPARAM::NULL, SMTO_NORMAL, 1000, &mut 0);
        EnumWindows(Some(get_workerw_proc), LPARAM::NULL);

        let instance = GetModuleHandleA(None);

        debug_assert!(!instance.is_null());

        let window_class = "window";
        let wc = WNDCLASSA {
            hInstance: instance,
            lpszClassName: PSTR(format!("{}\0", window_class).as_ptr() as _),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            ..Default::default()
        };
        let atom = RegisterClassA(&wc);

        debug_assert!(atom != 0);

        MONITOR_WIDTH = GetSystemMetrics(SM_CXSCREEN);
        MONITOR_HEIGHT = GetSystemMetrics(SM_CYSCREEN);

        let window = CreateWindowExA(
            Default::default(),
            window_class,
            "Desktop Video",
            WS_POPUP | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            MONITOR_WIDTH,
            MONITOR_HEIGHT,
            None,
            None,
            instance,
            ptr::null_mut(),
        );
        
        debug_assert!(!window.is_null());
        SetParent(window, WORKERW);

        let mut message = MSG::default();

        while GetMessageA(&mut message, HWND::NULL, 0, 0).into() { DispatchMessageA(&mut message); }
    }
}

static mut WORKERW: HWND = HWND::NULL;

unsafe extern "system" fn get_workerw_proc (window: HWND, _: LPARAM) -> BOOL {
    let p = FindWindowExW(window, HWND::NULL, "SHELLDLL_DefView", "");

    if p.eq(&HWND::NULL) { return BOOL(1) }
            
    WORKERW = FindWindowExW(HWND::NULL, window, "WorkerW", "");
            
    BOOL(0)
}

unsafe extern "system" fn window_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message as u32 {
        WM_CREATE => {
            if Path::new("temp/").exists() { fs::remove_dir_all("temp/").unwrap() }

            fs::create_dir("temp/").unwrap();
            
            Command::new("ffmpeg/bin/ffmpeg").arg("-i").arg(VIDEO_PATH).arg("-r").arg(FRAME_RATE.to_string()).arg("-s").arg(format!("{}x{}", MONITOR_WIDTH, MONITOR_HEIGHT)).arg("temp/temp_%03d.bmp").output().unwrap();

            LRESULT::NULL
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = Default::default();
            let hdc = BeginPaint(window, &mut ps);

            std::thread::spawn(move || {
                let mut bmp_vec = vec![];
                let mut total_image_count = 0;

                for path in fs::read_dir("temp/").unwrap() {
                    let bmp = HBITMAP(LoadImageA(None, path.unwrap().path().to_str().unwrap(), IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE | LR_CREATEDIBSECTION).0);

                    debug_assert!(!bmp.is_null());

                    bmp_vec.push(bmp);
                    total_image_count += 1;
                }

                loop {
                    let mut count = 0;

                    for _ in 0..total_image_count {
                        let now = Instant::now();
                        let bmp = bmp_vec[count];

                        let dc_src = CreateCompatibleDC(None);
                        let bmp_prev = SelectObject(dc_src, bmp);

                        BitBlt(hdc, 0, 0, MONITOR_WIDTH, MONITOR_HEIGHT, dc_src, 0, 0, SRCCOPY);
                        
                        SelectObject(dc_src, bmp_prev);
                        DeleteDC(dc_src);

                        sleep(Duration::from_millis(1000 / FRAME_RATE) - now.elapsed());

                        count += 1;
                    }
                }
            });

            LRESULT::NULL
        }
        WM_DESTROY => {
            PostQuitMessage(0);

            LRESULT::NULL
        }
        _ => DefWindowProcA(window, message, wparam, lparam),
    }
}