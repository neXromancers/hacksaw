extern crate structopt;
extern crate xcb;
use structopt::StructOpt;
use xcb::shape;

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

fn grab_pointer_set_cursor(conn: &xcb::Connection, root: u32) {
    let font = conn.generate_id();
    xcb::open_font(&conn, font, "cursor");

    // TODO: create cursor with a Pixmap
    // https://stackoverflow.com/questions/40578969/how-to-create-a-cursor-in-x11-from-raw-data-c
    let cursor = conn.generate_id();
    xcb::create_glyph_cursor(&conn, cursor, font, font, 0, 30, 0, 0, 0, 0, 0, 0);

    xcb::grab_pointer(
        &conn,
        true,
        root,
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

#[derive(StructOpt, Debug)]
#[structopt(name = "hacksaw")]
struct Opt {
    #[structopt(
        short = "n",
        long = "no-guides",
        help = "Disable fighter pilot guide lines"
    )]
    no_guides: bool,

    #[structopt(
        short = "g",
        long = "guide-thickness",
        default_value = "1",
        help = "Thickness of fighter pilot guide lines"
    )]
    guide_thickness: u16,

    #[structopt(
        short = "s",
        long = "select-thickness",
        default_value = "1",
        help = "Thickness of selection box lines"
    )]
    select_thickness: u16,

    #[structopt(
        short = "c",
        long = "colour",
        default_value = "#7f7f7f",
        parse(try_from_str = "parse_hex"),
        help = "Hex colour of the lines (RGB or RGBA), '#' optional"
    )]
    line_colour: u32,
}

/// Parse an HTML-color-like hex input
fn parse_hex(hex: &str) -> Result<u32, String> {
    let hex = hex.trim_start_matches('#');

    let mut color;

    match hex.len() {
        3 | 4 => {
            color = 0x11 * u32::from_str_radix(&hex[2..3], 16).expect("Invalid hex")
                + 0x11_00 * u32::from_str_radix(&hex[1..2], 16).expect("Invalid hex")
                + 0x11_00_00 * u32::from_str_radix(&hex[0..1], 16).expect("Invalid hex");

            if hex.len() == 4 {
                color |= 0x11_00_00_00 * u32::from_str_radix(&hex[3..4], 16).expect("Invalid hex");
            } else {
                color |= 0xFF_00_00_00;
            }
        }

        6 | 8 => {
            color = u32::from_str_radix(&hex, 16).expect("Invalid hex");

            if hex.len() == 6 {
                color = color | 0xFF_00_00_00;
            }
        }

        _ => return Err("Bad hex colour".to_string()),
    }

    Ok(color)
}

fn main() {
    let opt = Opt::from_args();

    let line_width = opt.select_thickness;
    let guide_width = opt.guide_thickness;
    let line_colour = opt.line_colour;

    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let window = conn.generate_id();

    grab_pointer_set_cursor(&conn, screen.root());

    let scr_height = screen.height_in_pixels();
    let scr_width = screen.width_in_pixels();

    // TODO event handling for expose/keypress
    let values = [
        // ?RGB. First 4 bytes appear to do nothing
        (xcb::CW_BACK_PIXEL, line_colour),
        (
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS, // we'll need this later
        ),
        (xcb::CW_OVERRIDE_REDIRECT, 1u32), // Don't be window managed
    ];

    xcb::create_window(
        &conn,
        xcb::COPY_FROM_PARENT as u8, // usually 32?
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

    set_shape(&conn, window, &[xcb::Rectangle::new(0, 0, 0, 0)]);

    xcb::map_window(&conn, window);

    conn.flush();

    // TODO formalise the fact that motion comes after press?
    let mut start_x = 0;
    let mut start_y = 0;

    let mut width = 0;
    let mut height = 0;

    let mut in_selection = false;
    let mut ignore_next_release = false;

    loop {
        let ev = conn.wait_for_event().unwrap();
        match ev.response_type() {
            xcb::BUTTON_PRESS => {
                let button_press: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(&ev) };

                let detail = button_press.detail();
                if detail == 3 {
                    println!("Exiting due to right click");
                    return;
                } else {
                    set_shape(&conn, window, &[]);
                    conn.flush();
                    start_x = button_press.event_x();
                    start_y = button_press.event_y();
                    in_selection = !(detail == 4 || detail == 5);
                    ignore_next_release = detail == 4 || detail == 5;
                }
            }
            xcb::KEY_PRESS => {
                // TODO fix this by grabbing keyboard
                // TODO only quit on Esc and similar
                println!("Exiting due to key press");
                return;
            }
            xcb::MOTION_NOTIFY => {
                let motion: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&ev) };
                let x = motion.event_x();
                let y = motion.event_y();

                // TODO investigate efficiency of let mut outside loop vs let inside
                let left_x = x.min(start_x);
                let top_y = y.min(start_y);
                let right_x = x.max(start_x);
                let bot_y = y.max(start_y);

                width = (x - start_x).abs() as u16;
                height = (y - start_y).abs() as u16;

                let mut rects = match (opt.no_guides, in_selection) {
                    (_, true) => vec![
                        // Selection rectangle
                        // The last one is longer to compensate for the missing square
                        xcb::Rectangle::new(
                            left_x - line_width as i16,
                            top_y,
                            line_width,
                            height + line_width,
                        ),
                        xcb::Rectangle::new(
                            left_x - line_width as i16,
                            top_y - line_width as i16,
                            width + line_width,
                            line_width,
                        ),
                        xcb::Rectangle::new(
                            right_x as i16,
                            top_y - line_width as i16,
                            line_width,
                            height + line_width,
                        ),
                        xcb::Rectangle::new(left_x, bot_y, width + line_width, line_width),
                    ],
                    (false, false) => vec![
                        // Guides
                        xcb::Rectangle::new(x - guide_width as i16 / 2, 0, guide_width, scr_height),
                        xcb::Rectangle::new(0, y - guide_width as i16 / 2, scr_width, guide_width),
                    ],
                    (true, false) => vec![],
                };

                set_shape(&conn, window, &rects);
                conn.flush();
            }
            xcb::BUTTON_RELEASE => {
                let motion: &xcb::ButtonReleaseEvent = unsafe { xcb::cast_event(&ev) };
                let detail = motion.detail();
                if detail == 4 || detail == 5 {
                    continue; // Scroll wheel up/down release
                } else if ignore_next_release {
                    ignore_next_release = false;
                    continue;
                } else {
                    break;
                }
                // Move on after mouse released
            }
            _ => continue,
        };
    }
    // Now we have taken coordinates, we print them out
    println!("{}x{}+{}+{}", width, height, start_x, start_y);
}
