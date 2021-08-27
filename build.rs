fn main() {
    windows::build! {
        Windows::Win32::{
            UI::WindowsAndMessaging::*,
            Foundation::*,
            System::LibraryLoader::GetModuleHandleA,
            Graphics::Gdi::*
        }
    };
}