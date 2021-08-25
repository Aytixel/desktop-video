mod bindings { windows::include_bindings!(); }

use bindings::Windows::Win32::{
    UI::WindowsAndMessaging::*,
    Foundation::*,
    System::LibraryLoader::GetModuleHandleA,
    Graphics::Gdi::*,
    UI::Controls::*
};
use std::ptr;

unsafe extern "system" fn window_proc2(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message as u32 {
        WM_PAINT => {
            println!("WM_PAINT");

            let width = 1152;
            let height = 648;
            let mut ps: PAINTSTRUCT = Default::default();
            let hdc = BeginPaint(window, &mut ps);

            let bmp = HBITMAP(LoadImageA(None, "temp.bmp", IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE | LR_CREATEDIBSECTION).0);

            debug_assert!(!bmp.is_null());

            let dc_src = CreateCompatibleDC(None);
            let bmp_prev = SelectObject(dc_src, bmp);

            BitBlt(hdc, 0, 0, width, height, dc_src, 0, 0, SRCCOPY);
            
            SelectObject(dc_src, bmp_prev);
            DeleteDC(dc_src);
            DeleteObject(bmp);

            EndPaint(window, &ps);
            ReleaseDC(None, hdc);

            LRESULT(0)
        }
        WM_DESTROY => {
            println!("WM_DESTROY");
            PostQuitMessage(0);

            LRESULT(0)
        }
        _ => DefWindowProcA(window, message, wparam, lparam),
    }
}

unsafe extern "system" fn window_proc (tophandle: HWND, _: LPARAM) -> BOOL {
    let p = FindWindowExW(tophandle, HWND(0), "SHELLDLL_DefView", "");

    if p.eq(&HWND(0)) { return BOOL(1) }
            
    let workerw = FindWindowExW(HWND(0), tophandle, "WorkerW", "");
    let instance = GetModuleHandleA(None);

    debug_assert!(instance.0 != 0);

    let window_class = "window";
    let wc = WNDCLASSA {
        hInstance: instance,
        lpszClassName: PSTR(format!("{}\0", window_class).as_ptr() as _),
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc2),
        ..Default::default()
    };
    let atom = RegisterClassA(&wc);

    debug_assert!(atom != 0);

    let window = CreateWindowExA(
        Default::default(),
        window_class,
        "Desktop Video",
        WS_POPUP | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        1920,
        1080,
        None,
        None,
        instance,
        ptr::null_mut(),
    );
    
    debug_assert!(window.0 != 0);
    SetParent(window, workerw);

    let mut message = MSG::default();

    while GetMessageA(&mut message, HWND(0), 0, 0).into() { DispatchMessageA(&mut message); }
            
    BOOL(0)
}

fn main() {
    unsafe {
        let progman = FindWindowW("Progman", PWSTR::NULL);

        SendMessageTimeoutW(progman, 0x052C, WPARAM(0), LPARAM(0), SMTO_NORMAL, 1000, &mut 0);
        EnumWindows(Some(window_proc), LPARAM(0));
    }
}
