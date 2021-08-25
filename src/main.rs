mod bindings {
    windows::include_bindings!();
}

use bindings::{
    Windows::Win32::UI::WindowsAndMessaging::*,
    Windows::Win32::Foundation::*
};
use std::ptr;

unsafe extern "system" fn window_proc (tophandle: HWND, _: LPARAM) -> BOOL {
    let p = FindWindowExW(tophandle, HWND(0), "SHELLDLL_DefView", "");

    if p.eq(&HWND(0)) { return BOOL(1) }
            
    let workerw = FindWindowExW(HWND(0), tophandle, "WorkerW", "");

    println!("{:?}", workerw);
            
    BOOL(0)
}

fn main() {
    unsafe {
        let progman = FindWindowW("Progman", PWSTR::NULL);

        SendMessageTimeoutW(progman, 0x052C, WPARAM(0), LPARAM(0), SMTO_NORMAL, 1000, &mut 0);

        EnumWindows(Some(window_proc), LPARAM(0));
    }
}
