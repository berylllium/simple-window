//! A simple windowing library.
mod utility;

use std::{mem::MaybeUninit, num::NonZeroIsize, ptr};

use raw_window_handle::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};

#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::{HWND, HINSTANCE, LPARAM, LRESULT, RECT, WPARAM},
    System::LibraryLoader::GetModuleHandleA,
    UI::WindowsAndMessaging::{
        AdjustWindowRectEx, LoadCursorW, LoadIconW, MessageBoxA, ShowWindow, CreateWindowExW, DestroyWindow, 
        DefWindowProcW, PeekMessageW, TranslateMessage, DispatchMessageW, GetClientRect,
        RegisterClassW, WNDCLASSW, MSG,
        CS_DBLCLKS, IDC_ARROW, IDI_APPLICATION, MB_ICONEXCLAMATION, MB_OK, SW_SHOW, SW_SHOWNOACTIVATE, 
        WS_CAPTION, WS_EX_APPWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
        WM_DESTROY, PM_REMOVE, WM_CLOSE, WM_ERASEBKGND, WM_EXITSIZEMOVE, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
        WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP, WM_USER
    },
};

pub enum WindowEvent {
    Close,
    Resize(u32, u32),
    Input(WindowInputEvent),
}

pub enum WindowInputEvent {
    KeyDown(Keys),
    KeyUp(Keys),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    MouseMove(i16, i16),
    MouseWheelMove(i16),
}

/// A cross-platform window wrapper.
/// 
/// # Examples
/// ```
/// use simple_window::{Window, WindowEvent, WindowInputEvent};
/// 
/// fn main() {
///     let mut is_running = true;
/// 
///     let mut window = Window::new("Example Window", 200, 200, 400, 600);
/// 
///     while is_running {
///         window.poll_messages(|event| {
///             match event {
///                 WindowEvent::Close => is_running = false,
///                 WindowEvent::Resize(width, height) => println!("Window resized: {}, {}", width, height),
///                 WindowEvent::Input(event) => match event {
///                     WindowInputEvent::MouseMove(x, y) => println!("Mouse moved!: {}, {}", x, y),
///                     WindowInputEvent::KeyDown(key) => println!("Key pressed: {}", key.as_str()),
///                     WindowInputEvent::KeyUp(key) => println!("Key released: {}", key.as_str()),
///                     WindowInputEvent::MouseWheelMove(dz) => println!("Mouse wheel {}", if dz > 0 { "up" } else { "down" }),
///                     WindowInputEvent::MouseDown(button) => println!("Mouse {} down.", button.as_str()),
///                     WindowInputEvent::MouseUp(button) => println!("Mouse {} up.", button.as_str()),
///                 },
///             }
///         });
///     }
/// }
/// ```
pub struct Window {
    previous_size: (u32, u32),

    #[cfg(target_os = "windows")]
    h_instance: HINSTANCE,
    #[cfg(target_os = "windows")]
    hwnd: HWND,
}

#[cfg(target_os = "windows")]
const CUSTOM_CLOSE_MESSAGE: u32 = WM_USER + 0;
#[cfg(target_os = "windows")]
const CUSTOM_SIZE_MESSAGE: u32 = WM_USER + 1;

#[cfg(target_os = "windows")]
extern "system" fn win32_process_message(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    use windows_sys::Win32::{Foundation::GetLastError, UI::WindowsAndMessaging::{PostMessageW, PostQuitMessage}};

    match msg {
        WM_ERASEBKGND => 1,
        WM_CLOSE => {
            unsafe { PostMessageW(hwnd, CUSTOM_CLOSE_MESSAGE, 0, 0); }
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0); }
            0
        },
        WM_EXITSIZEMOVE => {
            // println!("A");
            if unsafe { PostMessageW(hwnd, CUSTOM_SIZE_MESSAGE, 0, 0) } == 0 {
                println!("Failed to post. {}", unsafe { GetLastError() });
            }

            unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) }
        },
        _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
    }
}

impl Window {
    /// Creates a new window at position (`x`, `y`), and name `window_name`.
    pub fn new(
        window_name: &str,
        x: i32, y: i32,
        width: i32, height: i32,
    ) -> Self {
        #[cfg(target_os = "windows")]
        Self::new_win32(window_name, x, y, width, height)
    }

    /// Polls and parses system messages directed at the window and passes them on to the `event_closure` closure.
    pub fn poll_messages(&mut self, event_closure: impl FnMut(WindowEvent)) {
        #[cfg(target_os = "windows")]
        self.poll_messages_win32(event_closure);
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        #[cfg(target_os = "windows")]
        self.raw_window_handle_win32()
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        #[cfg(target_os = "windows")]
        self.raw_display_handle_windows()
    }

    fn wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
}

#[cfg(target_os = "windows")]
impl Window {
    pub const WINDOW_CLASS_NAME: &'static str = "window_class";

    fn new_win32(
        window_name: &str,
        x: i32, y: i32,
        width: i32, height: i32,
    ) -> Self {
        let window_class_name_utf16 = Self::wide_null(Self::WINDOW_CLASS_NAME);
        let application_name_utf16 = Self::wide_null(window_name);

        let h_instance = unsafe { GetModuleHandleA(ptr::null()) };

        let icon = unsafe { LoadIconW(h_instance, IDI_APPLICATION) };

        let wc = WNDCLASSW {
            style: CS_DBLCLKS,
            lpfnWndProc: Some(win32_process_message),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: icon,
            hCursor: unsafe { LoadCursorW(0, IDC_ARROW) },
            hbrBackground: 0,
            lpszClassName: window_class_name_utf16.as_ptr(),
            lpszMenuName: ptr::null(),
        };

        if unsafe { RegisterClassW(&wc) } == 0 {
            unsafe {
                MessageBoxA(
                    0,
                    "Window registration failed.".as_ptr(),
                    "Error".as_ptr(),
                    MB_ICONEXCLAMATION | MB_OK
                );
            }

            log::error!("Window registration failed.");
            panic!("Window registration failed.");
        }

        let client_x = x;
        let client_y = y;
        let client_width = width;
        let client_height = height;

        let mut window_x = client_x;
        let mut window_y = client_y;
        let mut window_width = client_width;
        let mut window_height = client_height;

        let window_style = WS_OVERLAPPED | WS_SYSMENU | WS_CAPTION | WS_MAXIMIZEBOX | WS_MINIMIZEBOX | WS_THICKFRAME;
        let window_ex_style = WS_EX_APPWINDOW;

        let mut border_rect = RECT { left: 0, right: 0, top: 0, bottom: 0 };
        unsafe { AdjustWindowRectEx(&mut border_rect, window_style, 0, window_ex_style); }

        window_x += border_rect.left;
        window_y += border_rect.top;
        window_width += border_rect.right - border_rect.left;
        window_height += border_rect.bottom - border_rect.top;

        let handle = unsafe {
            CreateWindowExW(
                window_ex_style, window_class_name_utf16.as_ptr(), application_name_utf16.as_ptr(),
                window_style, window_x, window_y, window_width, window_height,
                0, 0, h_instance, ptr::null()
            )
        };

        if handle == 0 {
            unsafe {
                MessageBoxA(
                    0,
                    "Window creation failed.".as_ptr(),
                    "Error".as_ptr(),
                    MB_ICONEXCLAMATION | MB_OK
                );
            }

            log::error!("Window creation failed.");
            panic!("Window creation failed.");
        }

        // Show the window.
        let should_activate = true;
        let show_window_command_flags = if should_activate { SW_SHOW } else { SW_SHOWNOACTIVATE };

        unsafe { ShowWindow(handle, show_window_command_flags); }

        Self {
            previous_size: (window_width as u32, window_height as u32),
            h_instance,
            hwnd: handle,
        }
    }

    fn poll_messages_win32(&mut self, mut event_closure: impl FnMut(WindowEvent)) {
        let mut message = MaybeUninit::<MSG>::uninit();

        while unsafe { PeekMessageW(message.as_mut_ptr(), self.hwnd, 0, 0, PM_REMOVE) } != 0 {
            unsafe {
                if !(message.assume_init().message == CUSTOM_CLOSE_MESSAGE
                    || message.assume_init().message == CUSTOM_SIZE_MESSAGE) {
                    TranslateMessage(message.as_mut_ptr());
                    DispatchMessageW(message.as_mut_ptr());
                }
            }
            
            match unsafe { message.assume_init().message } {
                CUSTOM_CLOSE_MESSAGE => {
                    (event_closure)(WindowEvent::Close);
                },
                CUSTOM_SIZE_MESSAGE => {
                    let mut r = MaybeUninit::<RECT>::uninit();
                    unsafe { GetClientRect(self.hwnd, r.as_mut_ptr()); }


                    let width = unsafe { r.assume_init().right - r.assume_init().left } as u32;
                    let height = unsafe { r.assume_init().bottom - r.assume_init().top } as u32;
                    
                    if self.previous_size != (width, height) {
                        self.previous_size = (width, height);
                        (event_closure)(WindowEvent::Resize(width, height));
                    }
                },
                WM_MOUSEMOVE => {
                    let mouse_pos = utility::get_x_y_lparam(unsafe{ message.assume_init().lParam });
                    (event_closure)(WindowEvent::Input(WindowInputEvent::MouseMove(mouse_pos.0, mouse_pos.1)));
                },
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    // Check for repeats and prevent sending.
                    if ((unsafe { message.assume_init().lParam } >> 30) & 1) as u8 == 0 {
                        let key = Keys::from_usize(unsafe { message.assume_init().wParam });
                        (event_closure)(WindowEvent::Input(WindowInputEvent::KeyDown(key)));
                    }
                },
                WM_KEYUP | WM_SYSKEYUP => {
                    let key = Keys::from_usize(unsafe { message.assume_init().wParam });
                    (event_closure)(WindowEvent::Input(WindowInputEvent::KeyUp(key)));
                },
                WM_MOUSEWHEEL => {
                    let dz = if utility::get_wheel_delta_wparam(unsafe { message.assume_init().wParam }) < 0 {
                        -1i16
                    } else {
                        1i16
                    };

                    (event_closure)(WindowEvent::Input(WindowInputEvent::MouseWheelMove(dz)));
                },
                WM_LBUTTONDOWN => (event_closure)(WindowEvent::Input(WindowInputEvent::MouseDown(MouseButton::Left))),
                WM_MBUTTONDOWN => (event_closure)(WindowEvent::Input(WindowInputEvent::MouseDown(MouseButton::Middle))),
                WM_RBUTTONDOWN => (event_closure)(WindowEvent::Input(WindowInputEvent::MouseDown(MouseButton::Right))),
                WM_LBUTTONUP => (event_closure)(WindowEvent::Input(WindowInputEvent::MouseUp(MouseButton::Left))),
                WM_MBUTTONUP => (event_closure)(WindowEvent::Input(WindowInputEvent::MouseUp(MouseButton::Middle))),
                WM_RBUTTONUP => (event_closure)(WindowEvent::Input(WindowInputEvent::MouseUp(MouseButton::Right))),
                _ => (),
            }

        }
    }

    fn raw_window_handle_win32(&self) -> RawWindowHandle {
        let mut handle = Win32WindowHandle::new(NonZeroIsize::new(self.hwnd).unwrap());
        handle.hinstance = NonZeroIsize::new(self.h_instance);

        RawWindowHandle::Win32(handle)
    }

    fn raw_display_handle_windows(&self) -> RawDisplayHandle {
        RawDisplayHandle::Windows(WindowsDisplayHandle::new())
    }
}

#[cfg(target_os = "windows")]
impl Drop for Window {
    fn drop(&mut self) {
        unsafe { DestroyWindow(self.hwnd); }
    }
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Left => "Left",
            Self::Middle => "Middle",
            Self::Right => "Right",
        }
    }
}

pub enum Keys {
    Backspace,
    Enter,
    Tab,
    Shift,
    Control,

    Pause,
    Capital,

    Escape,

    Convert,
    Nonconvert,
    Accept,
    Modechange,

    Space,
    Prior,
    Next,
    End,
    Home,
    Left,
    Up,
    Right,
    Down,
    Select,
    Print,
    Execute,
    Snapshot,
    Insert,
    Delete,
    Help,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    LWin,
    RWin,
    Apps,

    Sleep,

    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    Multiply,
    Add,
    Separator,
    Subtract,
    Decimal,
    Divide,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    Numlock,
    Scroll,

    NumpadEqual,

    LShift,
    RShift,
    LControl,
    RControl,
    LMenu,
    RMenu,

    Semicolon,
    Plus,
    Comma,
    Minus,
    Period,
    Slash,
    Grave,
}

impl Keys {
    pub fn from_usize(s: usize) -> Self {
        match s {
            0x08 => Self::Backspace,
            0x0D => Self::Enter,
            0x09 => Self::Tab,
            0x10 => Self::Shift,
            0x11 => Self::Control,

            0x13 => Self::Pause,
            0x14 => Self::Capital,

            0x1B => Self::Escape,

            0x1C => Self::Convert,
            0x1D => Self::Nonconvert,
            0x1E => Self::Accept,
            0x1F => Self::Modechange,

            0x20 => Self::Space,
            0x21 => Self::Prior,
            0x22 => Self::Next,
            0x23 => Self::End,
            0x24 => Self::Home,
            0x25 => Self::Left,
            0x26 => Self::Up,
            0x27 => Self::Right,
            0x28 => Self::Down,
            0x29 => Self::Select,
            0x2A => Self::Print,
            0x2B => Self::Execute,
            0x2C => Self::Snapshot,
            0x2D => Self::Insert,
            0x2E => Self::Delete,
            0x2F => Self::Help,

            0x41 => Self::A,
            0x42 => Self::B,
            0x43 => Self::C,
            0x44 => Self::D,
            0x45 => Self::E,
            0x46 => Self::F,
            0x47 => Self::G,
            0x48 => Self::H,
            0x49 => Self::I,
            0x4A => Self::J,
            0x4B => Self::K,
            0x4C => Self::L,
            0x4D => Self::M,
            0x4E => Self::N,
            0x4F => Self::O,
            0x50 => Self::P,
            0x51 => Self::Q,
            0x52 => Self::R,
            0x53 => Self::S,
            0x54 => Self::T,
            0x55 => Self::U,
            0x56 => Self::V,
            0x57 => Self::W,
            0x58 => Self::X,
            0x59 => Self::Y,
            0x5A => Self::Z,

            0x5B => Self::LWin,
            0x5C => Self::RWin,
            0x5D => Self::Apps,

            0x5F => Self::Sleep,

            0x60 => Self::Numpad0,
            0x61 => Self::Numpad1,
            0x62 => Self::Numpad2,
            0x63 => Self::Numpad3,
            0x64 => Self::Numpad4,
            0x65 => Self::Numpad5,
            0x66 => Self::Numpad6,
            0x67 => Self::Numpad7,
            0x68 => Self::Numpad8,
            0x69 => Self::Numpad9,
            0x6A => Self::Multiply,
            0x6B => Self::Add,
            0x6C => Self::Separator,
            0x6D => Self::Subtract,
            0x6E => Self::Decimal,
            0x6F => Self::Divide,
            0x70 => Self::F1,
            0x71 => Self::F2,
            0x72 => Self::F3,
            0x73 => Self::F4,
            0x74 => Self::F5,
            0x75 => Self::F6,
            0x76 => Self::F7,
            0x77 => Self::F8,
            0x78 => Self::F9,
            0x79 => Self::F10,
            0x7A => Self::F11,
            0x7B => Self::F12,
            0x7C => Self::F13,
            0x7D => Self::F14,
            0x7E => Self::F15,
            0x7F => Self::F16,
            0x80 => Self::F17,
            0x81 => Self::F18,
            0x82 => Self::F19,
            0x83 => Self::F20,
            0x84 => Self::F21,
            0x85 => Self::F22,
            0x86 => Self::F23,
            0x87 => Self::F24,

            0x90 => Self::Numlock,
            0x91 => Self::Scroll,

            0x92 => Self::NumpadEqual,

            0xA0 => Self::LShift,
            0xA1 => Self::RShift,
            0xA2 => Self::LControl,
            0xA3 => Self::RControl,
            0xA4 => Self::LMenu,
            0xA5 => Self::RMenu,

            0xBA => Self::Semicolon,
            0xBB => Self::Plus,
            0xBC => Self::Comma,
            0xBD => Self::Minus,
            0xBE => Self::Period,
            0xBF => Self::Slash,
            0xC0 => Self::Grave,
            _ => panic!("Provided usize does not corrospond to a valid Key."),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Backspace => "Backspace",
            Self::Enter => "Enter",
            Self::Tab => "Tab",
            Self::Shift => "Shift",
            Self::Control => "Control",

            Self::Pause => "Pause",
            Self::Capital => "Capital",

            Self::Escape => "Escape",

            Self::Convert => "Convert",
            Self::Nonconvert => "Nonconvert",
            Self::Accept => "Accept",
            Self::Modechange => "Modechange",

            Self::Space => "Space",
            Self::Prior => "Prior",
            Self::Next => "Next",
            Self::End => "End",
            Self::Home => "Home",
            Self::Left => "Left",
            Self::Up => "Up",
            Self::Right => "Right",
            Self::Down => "Down",
            Self::Select => "Select",
            Self::Print => "Print",
            Self::Execute => "Execute",
            Self::Snapshot => "Snapshot",
            Self::Insert => "Insert",
            Self::Delete => "Delete",
            Self::Help => "Help",

            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",

            Self::LWin => "LWin",
            Self::RWin => "RWin",
            Self::Apps => "Apps",

            Self::Sleep => "Sleep",

            Self::Numpad0 => "Numpad0",
            Self::Numpad1 => "Numpad1",
            Self::Numpad2 => "Numpad2",
            Self::Numpad3 => "Numpad3",
            Self::Numpad4 => "Numpad4",
            Self::Numpad5 => "Numpad5",
            Self::Numpad6 => "Numpad6",
            Self::Numpad7 => "Numpad7",
            Self::Numpad8 => "Numpad8",
            Self::Numpad9 => "Numpad9",
            Self::Multiply => "Multiply",
            Self::Add => "Add",
            Self::Separator => "Separator",
            Self::Subtract => "Subtract",
            Self::Decimal => "Decimal",
            Self::Divide => "Divide",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::F13 => "F13",
            Self::F14 => "F14",
            Self::F15 => "F15",
            Self::F16 => "F16",
            Self::F17 => "F17",
            Self::F18 => "F18",
            Self::F19 => "F19",
            Self::F20 => "F20",
            Self::F21 => "F21",
            Self::F22 => "F22",
            Self::F23 => "F23",
            Self::F24 => "F24",

            Self::Numlock => "Numlock",
            Self::Scroll => "Scroll",

            Self::NumpadEqual => "NumpadEqual",

            Self::LShift => "LShift",
            Self::RShift => "RShift",
            Self::LControl => "LControl",
            Self::RControl => "RControl",
            Self::LMenu => "LMenu",
            Self::RMenu => "RMenu",

            Self::Semicolon => "Semicolon",
            Self::Plus => "Plus",
            Self::Comma => "Comma",
            Self::Minus => "Minus",
            Self::Period => "Period",
            Self::Slash => "Slash",
            Self::Grave => "Grave",
        }
    }
}
 