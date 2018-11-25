extern crate xcb;

fn main() {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let window = conn.generate_id();

    let width = screen.width_in_pixels();
    let height = screen.height_in_pixels();

    println!("width {} height {}", width, height);

    let values = [
        // ?RGB. First 4 bytes appear to do nothing
        (xcb::CW_BACK_PIXEL, 0x00_00_00_00),
        (
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_EXPOSURE
                | xcb::EVENT_MASK_KEY_PRESS // we'll need this later
        ),
        (xcb::CW_OVERRIDE_REDIRECT, 1 as u32), // Don't be window managed
    ];

    xcb::create_window(
        &conn,
        xcb::COPY_FROM_PARENT as u8,
        window,
        screen.root(),
        0, // x
        0, // y
        0, // width
        0, // height
        0,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &values,
    );

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

    xcb::grab_pointer(
        &conn,
        true,
        screen.root(),
        (xcb::EVENT_MASK_BUTTON_RELEASE
            | xcb::EVENT_MASK_BUTTON_PRESS
            | xcb::EVENT_MASK_BUTTON_1_MOTION) as u16,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::NONE,
        xcb::NONE,
        xcb::CURRENT_TIME,
    ).get_reply()
    .unwrap();

    conn.flush();

    // TODO formalise the fact that motion comes after press?
    let mut start_x = 0;
    let mut start_y = 0;

    let mut x = 0;
    let mut y = 0;

    loop {
        let ev = conn.wait_for_event().unwrap();
        match ev.response_type() {
            xcb::BUTTON_PRESS => {
                let button_press: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(&ev) };
                println!(
                    "Mouse press: x={}, y={}",
                    button_press.event_x(),
                    button_press.event_y()
                );
                start_x = button_press.event_x();
                start_y = button_press.event_y();
            }
            xcb::BUTTON_RELEASE => {
                let button_release: &xcb::ButtonReleaseEvent = unsafe { xcb::cast_event(&ev) };
                println!(
                    "Mouse release: x={}, y={}",
                    button_release.event_x(),
                    button_release.event_y()
                );
                break; // Move on after mouse released
            }
            xcb::MOTION_NOTIFY => {
                let motion: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&ev) };
                println!(
                    "Mouse motion: x={}, y={}",
                    motion.event_x(),
                    motion.event_y()
                );
                x = motion.event_x();
                y = motion.event_y();
            }
            _ => continue,
        };
    }
    // Now we have taken coordinates, we use them
    let width = (x - start_x).abs();
    let height = (y - start_y).abs();
    println!("{}x{}+{}+{}", width, height, start_x, start_y);
}
