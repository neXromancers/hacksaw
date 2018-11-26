extern crate xcb;
use std::cmp::{max, min};
use xcb::shape;

const LINE_WIDTH: u16 = 2;
const GUIDE_WIDTH: u16 = 1;

fn set_shape(conn: &xcb::Connection, window: xcb::Window, rects: &[xcb::Rectangle]) {
    shape::rectangles(
        &conn,
        shape::SO_SET as u8,
        shape::SK_BOUNDING as u8,
        0,
        window,
        0,
        0,
        &rects,
    );
}

fn set_title(conn: &xcb::Connection, window: xcb::Window, title: &str) {
    xcb::change_property(
        &conn,
        xcb::PROP_MODE_REPLACE as u8,
        window,
        xcb::ATOM_WM_NAME,
        xcb::ATOM_STRING,
        8,
        title.as_bytes(),
    );
}

fn grab_pointer_set_cursor(conn: &xcb::Connection, window: xcb::Window, screen: xcb::Screen) {
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
            | xcb::EVENT_MASK_BUTTON_MOTION
            | xcb::EVENT_MASK_POINTER_MOTION) as u16,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::NONE,
        cursor,
        xcb::CURRENT_TIME,
    ).get_reply()
    .unwrap();
}

fn main() {
    // TODO commandline options
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let window = conn.generate_id();

    let scr_height = screen.height_in_pixels();
    let scr_width = screen.width_in_pixels();

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
        0,          // x
        0,          // y
        scr_width,  // width
        scr_height, // height
        0,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &values,
    );

    set_title(&conn, window, "hacksaw");
    grab_pointer_set_cursor(&conn, window, screen);

    set_shape(&conn, window, &[xcb::Rectangle::new(0, 0, 0, 0)]);

    xcb::map_window(&conn, window);

    conn.flush();

    // TODO formalise the fact that motion comes after press?
    let mut start_x = 0;
    let mut start_y = 0;

    let mut x = 0;
    let mut y = 0;

    let mut width = 0;
    let mut height = 0;

    let mut in_selection = false;

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

                in_selection = true;
            }
            xcb::KEY_PRESS => {
                // TODO fix this by grabbing keyboard
                // TODO only quit on Esc and similar
                println!("Exiting due to key press");
                return;
            }
            xcb::MOTION_NOTIFY => {
                let motion: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&ev) };
                x = motion.event_x();
                y = motion.event_y();

                // TODO investigate efficiency of let mut outside loop vs let inside
                let top_x = min(x, start_x);
                let top_y = min(y, start_y);
                let bot_x = max(x, start_x);
                let bot_y = max(y, start_y);

                width = (x - start_x).abs() as u16;
                height = (y - start_y).abs() as u16;

                let mut rects = vec![
                    // Guides
                    xcb::Rectangle::new(x, 0, GUIDE_WIDTH, scr_height),
                    xcb::Rectangle::new(0, y, scr_width, GUIDE_WIDTH),
                ];

                if in_selection {
                    // Selection lines
                    // TODO consider how these overlap with the actual geometry - do they need
                    // offsetting?
                    rects.extend_from_slice(&[
                        xcb::Rectangle::new(top_x, top_y, line_width, height),
                        xcb::Rectangle::new(top_x, top_y, width, line_width),
                        xcb::Rectangle::new(bot_x, top_y, line_width, height),
                        xcb::Rectangle::new(top_x, bot_y, width + line_width, line_width),
                        // The last one is longer to compensate for the missing square
                    ]);
                }

                set_shape(&conn, window, &rects);
                conn.flush();
            }
            xcb::BUTTON_RELEASE => {
                let motion: &xcb::ButtonReleaseEvent = unsafe { xcb::cast_event(&ev) };
                match motion.detail() {
                    5 => continue, // Scroll wheel down
                    4 => continue, // Scroll wheel up
                    _ => break,    // Move on after mouse released
                }
            }
            _ => continue,
        };
    }

    // Now we have taken coordinates, we use them
    println!("{}x{}+{}+{}", width, height, start_x, start_y);
}
