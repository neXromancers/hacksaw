extern crate xcb;
use xcb::shape;

fn main() {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let window = conn.generate_id();

    // TODO event handling for expose/keypress
    let values = [
        // ?RGB. First 4 bytes appear to do nothing
        (xcb::CW_BACK_PIXEL, 0x00_00_00_00),
        (
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS, // we'll need this later
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

    let font = conn.generate_id();
    xcb::open_font(&conn, font, "cursor");

    // TODO: create cursor with a Pixmap
    // https://stackoverflow.com/questions/40578969/how-to-create-a-cursor-in-x11-from-raw-data-c
    let cursor = conn.generate_id();
    xcb::create_glyph_cursor(&conn, cursor, font, font, 0, 30, 0, 0, 0, 0, 0, 0);

    xcb::grab_pointer(
        &conn,
        true,
        screen.root(),
        (xcb::EVENT_MASK_BUTTON_RELEASE
            | xcb::EVENT_MASK_BUTTON_PRESS
            | xcb::EVENT_MASK_BUTTON_MOTION) as u16,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::NONE,
        cursor,
        xcb::CURRENT_TIME,
    ).get_reply()
    .unwrap();

    conn.flush();

    let temp_win = conn.generate_id();
    xcb::create_window(
        &conn,
        xcb::COPY_FROM_PARENT as u8,
        temp_win,
        screen.root(),
        0, // x
        0, // y
        200, // width
        499, // height
        0,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &[(xcb::CW_BACK_PIXEL, screen.black_pixel())],
    );

    let rects = [xcb::Rectangle::new(300, 300, 400, 300)];
    shape::rectangles(
        &conn,
        shape::SO_SET as u8,
        shape::SK_BOUNDING as u8,
        0,
        temp_win,
        0,
        0,
        &rects,
    );

    shape::combine(
        &conn,
        shape::SO_SET as u8,
        shape::SK_BOUNDING as u8,
        shape::SK_BOUNDING as u8,
        window,
        0,
        0,
        temp_win,
    );
    xcb::map_window(&conn, window);
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

                if button_press.detail() == 3 {
                    println!("Exiting due to right click");
                    return;
                }

                start_x = button_press.event_x();
                start_y = button_press.event_y();

                // For the case where there is no motion
                x = start_x;
                y = start_y;
            }
            xcb::KEY_PRESS => {
                println!("Exiting due to key press");
                return;
            }
            xcb::MOTION_NOTIFY => {
                let motion: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&ev) };
                x = motion.event_x();
                y = motion.event_y();
                // TODO draw rectangles (and guides)
            }
            xcb::BUTTON_RELEASE => {
                let motion: &xcb::ButtonReleaseEvent = unsafe { xcb::cast_event(&ev) };
                match motion.detail() {
                    5 => continue, // Scroll wheel down
                    4 => continue, // Scroll wheel up
                    _ => break,    // Move on after mouse released
                }
            }
            xcb::EXPOSE => {
                println!("expose!");
            }
            _ => continue,
        };
    }

    // Now we have taken coordinates, we use them
    let width = (x - start_x).abs();
    let height = (y - start_y).abs();
    println!("{}x{}+{}+{}", width, height, start_x, start_y);
}
