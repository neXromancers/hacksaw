extern crate xcb;

fn main() {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let window = conn.generate_id();

    let values = [
        (xcb::CW_BACK_PIXEL, screen.white_pixel()),
        (
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_EXPOSURE
                | xcb::EVENT_MASK_KEY_PRESS // we'll need this later
                | xcb::EVENT_MASK_BUTTON_PRESS
                | xcb::EVENT_MASK_BUTTON_RELEASE,
        ),
    ];

    xcb::create_window(
        &conn,
        xcb::COPY_FROM_PARENT as u8,
        window,
        screen.root(),
        0,
        0,
        150,
        150,
        10,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &values,
    );

    xcb::map_window(&conn, window);

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
}
