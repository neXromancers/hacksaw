extern crate structopt;
extern crate xcb;
use std::cmp::{max, min};
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
        help = "Colour of all drawn lines"
    )]
    line_colour: u32,
}

/// Parse an HTML-color-like hex input
// TODO alpha channel
fn parse_hex(hex: &str) -> Result<u32, String> {
    let hex = hex.trim_start_matches('#');
    let hex_string = match hex.len() {
        3 => hex
            .chars()
            .map(|c| format!("{0}{0}", c))
            .collect::<String>(),
        6 => hex.to_string(),
        _ => return Err("Bad hex colour".to_string()),
    };
    Ok(u32::from_str_radix(&hex_string, 16).expect("Invalid char in hex colour"))
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

    let scr_height = screen.height_in_pixels();
    let scr_width = screen.width_in_pixels();

    // TODO event handling for expose/keypress
    // TODO color as commandline arg
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
    grab_pointer_set_cursor(&conn, screen.root());

    set_shape(&conn, window, &[xcb::Rectangle::new(0, 0, 0, 0)]);

    xcb::map_window(&conn, window);

    conn.flush();

    // TODO formalise the fact that motion comes after press?
    let mut start_x = 0;
    let mut start_y = 0;

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
                let x = motion.event_x();
                let y = motion.event_y();

                // TODO investigate efficiency of let mut outside loop vs let inside
                let top_x = min(x, start_x);
                let top_y = min(y, start_y);
                let bot_x = max(x, start_x);
                let bot_y = max(y, start_y);

                width = (x - start_x).abs() as u16;
                height = (y - start_y).abs() as u16;

                // TODO consider how these overlap with the actual geometry - do they need
                // offsetting?
                let mut rects = match (opt.no_guides, in_selection) {
                    (_, true) => vec![
                        // Selection rectangle
                        // The last one is longer to compensate for the missing square
                        xcb::Rectangle::new(top_x, top_y, line_width, height),
                        xcb::Rectangle::new(top_x, top_y, width, line_width),
                        xcb::Rectangle::new(bot_x, top_y, line_width, height),
                        xcb::Rectangle::new(top_x, bot_y, width + line_width, line_width),
                    ],
                    (false, false) => vec![
                        // Guides
                        xcb::Rectangle::new(x, 0, guide_width, scr_height),
                        xcb::Rectangle::new(0, y, scr_width, guide_width),
                    ],
                    (true, false) => vec![],
                };

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
