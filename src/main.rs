extern crate vlc_static as vlc;

mod bindings { windows::include_bindings!(); }

use bindings::Windows::Win32::{
    UI::WindowsAndMessaging::*,
    Foundation::*,
    System::LibraryLoader::GetModuleHandleA
};
use vlc::{
    Instance,
    Media,
    MediaPlayer,
    MediaPlayerVideoEx,
    sys::get_vlc_dll
};
use std::ptr;
use std::thread::sleep;
use std::time::Duration;
use std::env::args;
use core::ffi::c_void;

static mut MONITOR_WIDTH: i32 = 1280;
static mut MONITOR_HEIGHT: i32 = 720;
static mut WORKERW: HWND = HWND::NULL;

fn main() {
    unsafe {
        let video_path = args().nth(1).unwrap();
        let vlc_instance = Instance::new().unwrap();
        let media = Media::new_path(&vlc_instance, video_path.as_str()).unwrap();
        let media_player = MediaPlayer::new(&vlc_instance).unwrap();
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

        (get_vlc_dll().libvlc_audio_set_mute)(media_player.raw(), 1);

        media_player.set_key_input(false);
        media_player.set_mouse_input(false);
        media_player.set_hwnd(window.0 as *mut c_void);
        media_player.set_media(&media);
        media_player.play().unwrap();

        let mut message = MSG::default();

        loop  {
            sleep(Duration::from_millis(100));

            if !media_player.is_playing() { play(&media_player, &media); }
            if GetMessageA(&mut message, HWND::NULL, 0, 0).into() { DispatchMessageA(&mut message); }
        }
    }
}

fn play(media_player: &MediaPlayer, media: &Media) {
    media_player.set_media(media);
    media_player.play().unwrap();
}

unsafe extern "system" fn get_workerw_proc (window: HWND, _: LPARAM) -> BOOL {
    let p = FindWindowExW(window, HWND::NULL, "SHELLDLL_DefView", "");

    if p.eq(&HWND::NULL) { return BOOL(1) }
            
    WORKERW = FindWindowExW(HWND::NULL, window, "WorkerW", "");
            
    BOOL(0)
}

unsafe extern "system" fn window_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message as u32 {
        WM_PAINT => {
            LRESULT::NULL
        }
        WM_DESTROY => {
            PostQuitMessage(0);

            LRESULT::NULL
        }
        _ => DefWindowProcA(window, message, wparam, lparam),
    }
}