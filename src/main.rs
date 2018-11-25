extern crate xcb;

// use std::fs::File;
// use std::io::Write;

fn main() {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let window = conn.generate_id();

    let width = screen.width_in_pixels();
    let height = screen.height_in_pixels();

    println!("width {} height {}", width, height);

    let screenshot = xcb::get_image(
        &conn,
        xcb::IMAGE_FORMAT_Z_PIXMAP as u8,
        screen.root(),
        0,
        0,
        width,
        height,
        0xFF_FF_FF_FF_u32,
    ).get_reply()
    .unwrap();

    // let mut buf = File::create("out.bin").unwrap();
    // buf.write(screenshot.data()).unwrap();

    let bg = conn.generate_id();
    xcb::create_pixmap(
        &conn,
        xcb::COPY_FROM_PARENT as u8,
        bg,
        screenshot,
        width,
        height,
    );

    let values = [
        // ?RGB. First 4 bytes appear to do nothing
        (xcb::CW_BACK_PIXEL, 0x00_00_00_00),
        // (xcb::CW_BACK_PIXMAP, bg),
        (
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_EXPOSURE
                | xcb::EVENT_MASK_KEY_PRESS // we'll need this later
                | xcb::EVENT_MASK_BUTTON_PRESS
                | xcb::EVENT_MASK_BUTTON_RELEASE,
        ),
        (xcb::CW_OVERRIDE_REDIRECT, 1 as u32), // Don't be window managed
    ];

    xcb::create_window(
        &conn,
        xcb::COPY_FROM_PARENT as u8,
        window,
        screen.root(),
        0,
        0,
        width / 2,
        height / 2,
        0,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &values,
    );

    // // Set transparency.
    // let opacity_atom = xcb::intern_atom(&conn, false, "_NET_WM_WINDOW_OPACITY")
    //     .get_reply()
    //     .expect("Couldn't create atom _NET_WM_WINDOW_OPACITY")
    //     .atom();
    // // let opacity = u32::max_value();
    // let opacity = 0;
    // xcb::change_property(
    //     &conn,
    //     xcb::PROP_MODE_REPLACE as u8,
    //     window,
    //     opacity_atom,
    //     xcb::ATOM_CARDINAL,
    //     32,
    //     &[opacity],
    // );

    xcb::map_window(&conn, window);

    let title = "hacksaw";
    // setting title
    xcb::change_property(
        &conn,
        xcb::PROP_MODE_REPLACE as u8,
        window,
        xcb::ATOM_WM_NAME,
        xcb::ATOM_STRING,
        8,
        title.as_bytes(),
    );

    conn.flush();

    loop {
        let ev = conn.wait_for_event();
        match ev {
            None => {
                break;
            }
            Some(ev) => {
                let r = ev.response_type();
                if r == xcb::BUTTON_PRESS as u8 {
                    let button_press: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(&ev) };
                    println!(
                        "Mouse press: x={}, y={}",
                        button_press.event_x(),
                        button_press.event_y()
                    );
                } else if r == xcb::BUTTON_RELEASE as u8 {
                    let button_release: &xcb::ButtonReleaseEvent = unsafe { xcb::cast_event(&ev) };
                    println!(
                        "Mouse release: x={}, y={}",
                        button_release.event_x(),
                        button_release.event_y()
                    );
                    break; // Move on after mouse released
                }
            }
        };
    }
    // Now we have taken coordinates, we use them
}

// let vinfo: xcb::XVisualInfo;
// unsafe {
//     xcb::XMatchVisualInfo(conn.get_raw_dpy(), screen_num, 32, xcb::Truecolor, &vinfo);
// }

// int main(int argc, char* argv[])
// {
//     Display* display = XOpenDisplay(NULL);

//     XVisualInfo vinfo;
//     XMatchVisualInfo(display, DefaultScreen(display), 32, TrueColor, &vinfo);

//     XSetWindowAttributes attr;
//     attr.colormap = XCreateColormap(display, DefaultRootWindow(display), vinfo.visual, AllocNone);
//     attr.border_pixel = 0;
//     attr.background_pixel = 0;

//     Window win = XCreateWindow(display, DefaultRootWindow(display), 0, 0, 300, 200, 0, vinfo.depth, InputOutput, vinfo.visual, CWColormap | CWBorderPixel | CWBackPixel, &attr);

//     XDestroyWindow(display, win);
//     XCloseDisplay(display);
//     return 0;
// }
