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
                    WindowInputEvent::MouseMove(x, y) => {
                        println!("Mouse moved!: {}, {}", x, y);
                    },
                    WindowInputEvent::KeyDown(key) => {
                        println!("Key pressed: {}", key.as_str());
                    },
                    WindowInputEvent::KeyUp(key) => {
                        println!("Key released: {}", key.as_str());
                    },
                    WindowInputEvent::MouseWheelMove(dz) => {
                        println!("Mouse wheel {}", if dz > 0 { "up" } else { "down" });
                    },
                    WindowInputEvent::MouseDown(button) => {
                        println!("Mouse {} down.", button.as_str());
                    },
                    WindowInputEvent::MouseUp(button) => {
                        println!("Mouse {} up.", button.as_str());
                    }
                },
            }
        });
    }
}
