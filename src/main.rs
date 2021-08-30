mod bindings { windows::include_bindings!(); }

use bindings::Windows::Win32::{
    UI::WindowsAndMessaging::*,
    Foundation::*,
    System::LibraryLoader::GetModuleHandleA,
    Graphics::Gdi::*
};
use std::ptr;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread::{
    sleep,
    spawn
};
use std::mem::size_of;
use std::time::{
    Instant,
    Duration
};
use core::ffi::c_void;

const VIDEO_PATH: &'static str = "demo.mp4";
const FRAME_RATE: u64 = 30;
const FRAME_DURATION: u64 = 1000 / FRAME_RATE;
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
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = Default::default();
            let hdc = BeginPaint(window, &mut ps);
            let hdc_src = CreateCompatibleDC(None);
            
            spawn(move || {
                let mut bmp_vec = vec![];
                let video_duration = String::from_utf8(Command::new("ffmpeg/bin/ffprobe").arg("-v").arg("error").arg("-show_entries").arg("format=duration").arg("-of").arg("default=noprint_wrappers=1:nokey=1").arg(VIDEO_PATH).output().unwrap().stdout).unwrap().trim().parse::<f32>().unwrap();
                let total_image_count = (video_duration / (1.0 / FRAME_RATE as f32)).round() as u32;

                if !Path::new("temp/").exists() { fs::create_dir("temp/").unwrap() }

                let mut ffmpeg_thread = Command::new("ffmpeg/bin/ffmpeg").arg("-v").arg("error").arg("-i").arg(VIDEO_PATH).arg("-r").arg(FRAME_RATE.to_string()).arg("-s").arg(format!("{}x{}", MONITOR_WIDTH, MONITOR_HEIGHT)).arg("temp/temp_%01d.bmp").spawn().unwrap();

                for i in 1..total_image_count + 1 {
                    let path_str = format!("temp/temp_{}.bmp", i);

                    while !Path::new(path_str.as_str()).exists() { sleep(Duration::new(0, 1000)) }

                    loop {
                        sleep(Duration::new(0, 1000));

                        if let Ok(image) = fs::read(Path::new(path_str.as_str())) {
                            let mut bmp_info: BITMAPINFO = Default::default();

                            bmp_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
                            bmp_info.bmiHeader.biWidth = MONITOR_WIDTH;
                            bmp_info.bmiHeader.biHeight = MONITOR_HEIGHT;
                            bmp_info.bmiHeader.biPlanes = 1;
                            bmp_info.bmiHeader.biBitCount = 24;
                            bmp_info.bmiHeader.biCompression = BI_RGB as u32;
                            bmp_info.bmiHeader.biSizeImage = 0;
                            bmp_info.bmiHeader.biXPelsPerMeter = 0;
                            bmp_info.bmiHeader.biYPelsPerMeter = 0;
                            bmp_info.bmiHeader.biClrUsed = 0;
                            bmp_info.bmiHeader.biClrImportant = 0;

                            let bmp = CreateCompatibleBitmap(hdc, MONITOR_WIDTH, MONITOR_HEIGHT);

                            SetDIBits(None, bmp, 0, MONITOR_HEIGHT as u32, image[image.len() - (MONITOR_WIDTH * MONITOR_HEIGHT * 3) as usize..image.len()].as_ptr() as *const c_void, &bmp_info, DIB_RGB_COLORS);

                            bmp_vec.push(bmp);

                            break
                        }
                    }

                    loop {
                        sleep(Duration::new(0, 1000));

                        if let Ok(_) = fs::remove_file(format!("temp/temp_{}.bmp", i)) { break }
                    }
                }

                ffmpeg_thread.wait().unwrap();

                loop {
                    let mut count = 0;

                    for _ in 0..total_image_count {
                        let now = Instant::now();
                        let bmp = bmp_vec[count];

                        SelectObject(hdc_src, bmp);
                        BitBlt(hdc, 0, 0, MONITOR_WIDTH, MONITOR_HEIGHT, hdc_src, 0, 0, SRCCOPY);

                        sleep(Duration::from_millis(FRAME_DURATION) - now.elapsed());

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