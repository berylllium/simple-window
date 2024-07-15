# `simple_window` - Simple cross-platform windowing library.

simple_window is a simple, lightweight, cross-platform library to create and query windows.

> **_NOTE:_** The library currently only supports GNU/Linux + X11 and Windows. Support for Wayland is planned in the future.

## Basic Usage

```rs
use simple_window::{Window, WindowEvent, WindowInputEvent};
 
fn main() {
    let mut is_running = true;
 
    let mut window = Window::new("Example Window", 200, 200, 400, 600);
 
    while is_running {
        window.poll_messages(|event| {
            match event {
                WindowEvent::Close => is_running = false,
                WindowEvent::Resize(width, height) => println!("Window resized: {}, {}", width, height),
                WindowEvent::Input(event) => match event {
                    WindowInputEvent::MouseMove(x, y) => println!("Mouse moved!: {}, {}", x, y),
                    WindowInputEvent::KeyDown(key) => println!("Key pressed: {}", key.as_str()),
                    WindowInputEvent::KeyUp(key) => println!("Key released: {}", key.as_str()),
                    WindowInputEvent::MouseWheelMove(dz) => println!("Mouse wheel {}", if dz > 0 { "up" } else { "down" }),
                    WindowInputEvent::MouseDown(button) => println!("Mouse {} down.", button.as_str()),
                    WindowInputEvent::MouseUp(button) => println!("Mouse {} up.", button.as_str()),
                },
            }
        });
    }
}
```

## Support
This library is intended to support only GNU/Linux & Windows. I have no intenion whatsoever of adding support for MacOS, but I am open to pull requests.

## Documentation
Please visit the [docs.rs](https://docs.rs/crate/simple-window/latest) page for documentation.
