//! A simple windowing library.
mod utility;

use std::{ffi::{c_uint, c_void}, num::NonZeroU32, os::raw::c_int, ptr::NonNull};

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[cfg(target_os = "linux")]
use raw_window_handle::{XcbDisplayHandle, XcbWindowHandle};

#[cfg(target_os = "linux")]
use xcb::{x, Xid};

#[cfg(target_os = "windows")]
use raw_window_handle::{Win32WindowHandle, WindowsDisplayHandle};

#[cfg(target_os = "windows")]
use std::{mem::MaybeUninit, num::NonZeroIsize, ptr};

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
    
    #[cfg(target_os = "linux")]
    connection: xcb::Connection,
    #[cfg(target_os = "linux")]
    window: u32,
    #[cfg(target_os = "linux")]
    screen: c_int,
    #[cfg(target_os = "linux")]
    wm_del_window: x::Atom,
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
        { Self::new_win32(window_name, x, y, width, height) }

        #[cfg(target_os = "linux")]
        { Self::new_linux_x(window_name, x, y, width, height) }
    }

    /// Polls and parses system messages directed at the window and passes them on to the `event_closure` closure.
    pub fn poll_messages(&mut self, event_closure: impl FnMut(WindowEvent)) {
        #[cfg(target_os = "windows")]
        { self.poll_messages_win32(event_closure); }

        #[cfg(target_os = "linux")]
        { self.poll_messages_linux_x(event_closure); }
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        #[cfg(target_os = "windows")]
        { self.raw_window_handle_win32() }

        #[cfg(target_os = "linux")]
        { self.raw_window_handle_linux_x() }
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        #[cfg(target_os = "windows")]
        { self.raw_display_handle_windows() }

        #[cfg(target_os = "linux")]
        { self.raw_display_handle_linux_x() }
    }
}

#[cfg(target_os = "linux")]
impl Window {
    fn new_linux_x(
        window_name: &str,
        x: i32, y: i32,
        width: i32, height: i32,
    ) -> Self {
        let (conn, screen_num) = xcb::Connection::connect_with_xlib_display().unwrap();

        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize).unwrap();

        let window: x::Window = conn.generate_id();

        let cookie = conn.send_request_checked(&x::CreateWindow {
            depth: x::COPY_FROM_PARENT as u8,
            wid: window,
            parent: screen.root(),
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
            border_width: 0,
            class: x::WindowClass::InputOutput,
            visual: screen.root_visual(),
            value_list: &[
                x::Cw::BackPixel(screen.white_pixel()),
                x::Cw::EventMask(x::EventMask::BUTTON_PRESS | x::EventMask::BUTTON_RELEASE | x::EventMask::KEY_PRESS
                    | x::EventMask::KEY_RELEASE | x::EventMask::EXPOSURE | x::EventMask::POINTER_MOTION
                    | x::EventMask::STRUCTURE_NOTIFY
                ),
            ],
        });
        conn.check_request(cookie).unwrap();

        let cookie = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: x::ATOM_WM_NAME,
            r#type: x::ATOM_STRING,
            data: window_name.as_bytes(),
        });
        conn.check_request(cookie).unwrap();

        conn.send_request(&x::MapWindow {
            window,
        });

        // Get atoms.
        let (wm_protocols, wm_del_window) = {
            let cookies = (
                conn.send_request(&x::InternAtom {
                    only_if_exists: true,
                    name: b"WM_PROTOCOLS",
                }),
                conn.send_request(&x::InternAtom {
                    only_if_exists: true,
                    name: b"WM_DELETE_WINDOW",
                }),
            );

            (
                conn.wait_for_reply(cookies.0).unwrap().atom(),
                conn.wait_for_reply(cookies.1).unwrap().atom(),
            )
        };

        conn.check_request(conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: wm_protocols,
            r#type: x::ATOM_ATOM,
            data: &[wm_del_window],
        })).unwrap();

        conn.flush().unwrap();

        Self {
            previous_size: (0, 0),
            connection: conn,
            screen: screen_num,
            window: window.resource_id(),
            wm_del_window,
        }
    }

    fn poll_messages_linux_x(&mut self, mut event_closure: impl FnMut(WindowEvent)) {
        while let Some(event) = self.connection.poll_for_event().unwrap() {
            if let xcb::Event::X(event) = event { match event {
                    x::Event::KeyPress(event) => {
                        let key = self.translate_key_code(event.detail());
                        (event_closure)(WindowEvent::Input(WindowInputEvent::KeyDown(key)));
                    },
                    x::Event::KeyRelease(event) => {
                        let key = self.translate_key_code(event.detail());
                        (event_closure)(WindowEvent::Input(WindowInputEvent::KeyUp(key)));
                    },
                    x::Event::ButtonPress(event) => {
                        let button = match event.detail() as c_uint{
                            x11::xlib::Button1 => MouseButton::Left,
                            x11::xlib::Button2 => MouseButton::Middle,
                            x11::xlib::Button3 => MouseButton::Right,
                            _ => panic!("Unrecognized mouse button x keycode.")
                        };

                        (event_closure)(WindowEvent::Input(WindowInputEvent::MouseDown(button)));
                    },
                    x::Event::ButtonRelease(event) => {
                        let button = match event.detail() as c_uint{
                            x11::xlib::Button1 => MouseButton::Left,
                            x11::xlib::Button2 => MouseButton::Middle,
                            x11::xlib::Button3 => MouseButton::Right,
                            _ => panic!("Unrecognized mouse button x keycode.")
                        };

                        (event_closure)(WindowEvent::Input(WindowInputEvent::MouseUp(button)));
                    },
                    x::Event::MotionNotify(event) => {
                        let x = event.event_x();
                        let y = event.event_x();
                        
                        (event_closure)(WindowEvent::Input(WindowInputEvent::MouseMove(x, y)));
                    },
                    x::Event::ConfigureNotify(event) => {
                        // Window resize. Also triggered by window move.

                        let x = event.width() as u32;
                        let y = event.height() as u32;

                        if self.previous_size != (x, y) {
                            self.previous_size = (x, y);

                            (event_closure)(WindowEvent::Resize(x, y));
                        }
                    },
                    x::Event::ClientMessage(event) => {
                        if let x::ClientMessageData::Data32([atom, ..]) = event.data() {
                            if atom == self.wm_del_window.resource_id() {
                                (event_closure)(WindowEvent::Close);
                            }
                        }
                    },
                    _ => {},
                }
            }
        }
    }

    fn raw_window_handle_linux_x(&self) -> RawWindowHandle {
        let handle = XcbWindowHandle::new(NonZeroU32::new(self.window).unwrap());

        RawWindowHandle::Xcb(handle)
    }

    fn raw_display_handle_linux_x(&self) -> RawDisplayHandle {
        let handle = XcbDisplayHandle::new(
            Some(NonNull::new(self.connection.get_raw_conn() as *mut c_void).unwrap()), self.screen
        );

        RawDisplayHandle::Xcb(handle)
    }

    fn translate_key_code(&self, x_keycode: x::Keycode) -> Keys {

        let key_sym = unsafe {
            x11::xlib::XkbKeycodeToKeysym(
                self.connection.get_raw_dpy(),
                x_keycode as x11::xlib::KeyCode,
                0,
                if x_keycode as u32 & x11::xlib::ShiftMask == 0 { 1 } else { 0 }
            )
        };

        match key_sym as c_uint {
            x11::keysym::XK_BackSpace => Keys::Backspace,
            x11::keysym::XK_Return => Keys::Enter,
            x11::keysym::XK_Tab => Keys::Tab,
                //x11::keysym::XK_Shift: return keys::SHIFT,
                //x11::keysym::XK_Control: return keys::CONTROL,

            x11::keysym::XK_Pause => Keys::Pause,
            x11::keysym::XK_Caps_Lock => Keys::Capital,

            x11::keysym::XK_Escape => Keys::Escape,

                // Not supported
                // case : return keys::CONVERT,
                // case : return keys::NONCONVERT,
                // case : return keys::ACCEPT,

            x11::keysym::XK_Mode_switch => Keys::Modechange,

            x11::keysym::XK_space => Keys::Space,
            x11::keysym::XK_Prior => Keys::Prior,
            x11::keysym::XK_Next => Keys::Next,
            x11::keysym::XK_End => Keys::End,
            x11::keysym::XK_Home => Keys::Home,
            x11::keysym::XK_Left => Keys::Left,
            x11::keysym::XK_Up => Keys::Up,
            x11::keysym::XK_Right => Keys::Right,
            x11::keysym::XK_Down => Keys::Down,
            x11::keysym::XK_Select => Keys::Select,
            x11::keysym::XK_Print => Keys::Print,
            x11::keysym::XK_Execute => Keys::Execute,
            // x11::keysym::XK_snapshot: return keys::SNAPSHOT, // not supported
            x11::keysym::XK_Insert => Keys::Insert,
            x11::keysym::XK_Delete => Keys::Delete,
            x11::keysym::XK_Help => Keys::Help,

            x11::keysym::XK_Super_L => Keys::LWin,
            x11::keysym::XK_Super_R => Keys::RWin,
                // x11::keysym::XK_apps: return keys::APPS, // not supported

                // x11::keysym::XK_sleep: return keys::SLEEP, //not supported

            x11::keysym::XK_KP_0 => Keys::Numpad0,
            x11::keysym::XK_KP_1 => Keys::Numpad1,
            x11::keysym::XK_KP_2 => Keys::Numpad2,
            x11::keysym::XK_KP_3 => Keys::Numpad3,
            x11::keysym::XK_KP_4 => Keys::Numpad4,
            x11::keysym::XK_KP_5 => Keys::Numpad5,
            x11::keysym::XK_KP_6 => Keys::Numpad6,
            x11::keysym::XK_KP_7 => Keys::Numpad7,
            x11::keysym::XK_KP_8 => Keys::Numpad8,
            x11::keysym::XK_KP_9 => Keys::Numpad9,
            x11::keysym::XK_multiply => Keys::Multiply,
            x11::keysym::XK_KP_Add => Keys::Add,
            x11::keysym::XK_KP_Separator => Keys::Separator,
            x11::keysym::XK_KP_Subtract => Keys::Subtract,
            x11::keysym::XK_KP_Decimal => Keys::Decimal,
            x11::keysym::XK_KP_Divide => Keys::Divide,
            x11::keysym::XK_F1 => Keys::F1,
            x11::keysym::XK_F2 => Keys::F2,
            x11::keysym::XK_F3 => Keys::F3,
            x11::keysym::XK_F4 => Keys::F4,
            x11::keysym::XK_F5 => Keys::F5,
            x11::keysym::XK_F6 => Keys::F6,
            x11::keysym::XK_F7 => Keys::F7,
            x11::keysym::XK_F8 => Keys::F8,
            x11::keysym::XK_F9 => Keys::F9,
            x11::keysym::XK_F10 => Keys::F10,
            x11::keysym::XK_F11 => Keys::F11,
            x11::keysym::XK_F12 => Keys::F12,
            x11::keysym::XK_F13 => Keys::F13,
            x11::keysym::XK_F14 => Keys::F14,
            x11::keysym::XK_F15 => Keys::F15,
            x11::keysym::XK_F16 => Keys::F16,
            x11::keysym::XK_F17 => Keys::F17,
            x11::keysym::XK_F18 => Keys::F18,
            x11::keysym::XK_F19 => Keys::F19,
            x11::keysym::XK_F20 => Keys::F20,
            x11::keysym::XK_F21 => Keys::F21,
            x11::keysym::XK_F22 => Keys::F22,
            x11::keysym::XK_F23 => Keys::F23,
            x11::keysym::XK_F24 => Keys::F24,

            x11::keysym::XK_Num_Lock => Keys::Numlock,
            x11::keysym::XK_Scroll_Lock => Keys::Scroll,

            x11::keysym::XK_KP_Equal => Keys::NumpadEqual,

            x11::keysym::XK_Shift_L => Keys::LShift,
            x11::keysym::XK_Shift_R => Keys::RShift,
            x11::keysym::XK_Control_L => Keys::LControl,
            x11::keysym::XK_Control_R => Keys::RControl,
            // x11::keysym::XK_Menu: return keys::LMENU,
            x11::keysym::XK_Menu => Keys::RMenu,

            x11::keysym::XK_semicolon => Keys::Semicolon,
            x11::keysym::XK_plus => Keys::Plus,
            x11::keysym::XK_comma => Keys::Comma,
            x11::keysym::XK_minus => Keys::Minus,
            x11::keysym::XK_period => Keys::Period,
            x11::keysym::XK_slash => Keys::Slash,
            x11::keysym::XK_grave => Keys::Grave,

            x11::keysym::XK_a | x11::keysym::XK_A => Keys::A,
            x11::keysym::XK_b | x11::keysym::XK_B => Keys::B,
            x11::keysym::XK_c | x11::keysym::XK_C => Keys::C,
            x11::keysym::XK_d | x11::keysym::XK_D => Keys::D,
            x11::keysym::XK_e | x11::keysym::XK_E => Keys::E,
            x11::keysym::XK_f | x11::keysym::XK_F => Keys::F,
            x11::keysym::XK_g | x11::keysym::XK_G => Keys::G,
            x11::keysym::XK_h | x11::keysym::XK_H => Keys::H,
            x11::keysym::XK_i | x11::keysym::XK_I => Keys::I,
            x11::keysym::XK_j | x11::keysym::XK_J => Keys::J,
            x11::keysym::XK_k | x11::keysym::XK_K => Keys::K,
            x11::keysym::XK_l | x11::keysym::XK_L => Keys::L,
            x11::keysym::XK_m | x11::keysym::XK_M => Keys::M,
            x11::keysym::XK_n | x11::keysym::XK_N => Keys::N,
            x11::keysym::XK_o | x11::keysym::XK_O => Keys::O,
            x11::keysym::XK_p | x11::keysym::XK_P => Keys::P,
            x11::keysym::XK_q | x11::keysym::XK_Q => Keys::Q,
            x11::keysym::XK_r | x11::keysym::XK_R => Keys::R,
            x11::keysym::XK_s | x11::keysym::XK_S => Keys::S,
            x11::keysym::XK_t | x11::keysym::XK_T => Keys::T,
            x11::keysym::XK_u | x11::keysym::XK_U => Keys::U,
            x11::keysym::XK_v | x11::keysym::XK_V => Keys::V,
            x11::keysym::XK_w | x11::keysym::XK_W => Keys::W,
            x11::keysym::XK_x | x11::keysym::XK_X => Keys::X,
            x11::keysym::XK_y | x11::keysym::XK_Y => Keys::Y,
            x11::keysym::XK_z | x11::keysym::XK_Z => Keys::Z,
            _ => panic!("Unknown x keycode. Got: {}", x_keycode)
        }
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

    fn wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
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
 
