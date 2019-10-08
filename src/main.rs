extern crate structopt;
extern crate xcb;
mod util;

use structopt::StructOpt;
use util::{fill_format_string, parse_format_string, Format, HacksawResult};
use xcb::shape;

const CURSOR_GRAB_TRIES: i32 = 5;

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

fn grab_pointer_set_cursor(conn: &xcb::Connection, root: u32) -> bool {
    let font = conn.generate_id();
    xcb::open_font(&conn, font, "cursor");

    // TODO: create cursor with a Pixmap
    // https://stackoverflow.com/questions/40578969/how-to-create-a-cursor-in-x11-from-raw-data-c
    let cursor = conn.generate_id();
    xcb::create_glyph_cursor(&conn, cursor, font, font, 0, 30, 0, 0, 0, 0, 0, 0);

    for i in 0..CURSOR_GRAB_TRIES {
        let reply = xcb::grab_pointer(
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
        )
        .get_reply()
        .unwrap();

        if reply.status() as u32 == xcb::GRAB_STATUS_SUCCESS {
            return true;
        } else if i < CURSOR_GRAB_TRIES - 1 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    false
}

fn contained(x: i16, y: i16, width: i16, height: i16, p_x: i16, p_y: i16) -> bool {
    // TODO negative x/y offsets from bottom or right?
    x < p_x && y < p_y && p_x - x <= width && p_y - y <= height
}

fn viewable(conn: &xcb::Connection, win: xcb::Window) -> bool {
    let attrs = xcb::get_window_attributes(conn, win).get_reply().unwrap();
    (attrs.map_state() & xcb::MAP_STATE_VIEWABLE as u8) != 0
}

fn input_output(conn: &xcb::Connection, win: xcb::Window) -> bool {
    let attrs = xcb::get_window_attributes(conn, win).get_reply().unwrap();
    (attrs.class() & xcb::WINDOW_CLASS_INPUT_OUTPUT as u16) != 0
}

fn get_window_geom(conn: &xcb::Connection, win: xcb::Window) -> HacksawResult {
    let geom = xcb::get_geometry(conn, win).get_reply().unwrap();
    let (gx, gy, gw, gh, border): (i16, i16, u16, u16, u16) = (
        geom.x(),
        geom.y(),
        geom.width(),
        geom.height(),
        geom.border_width(),
    );

    HacksawResult {
        window: win,
        x: gx,
        y: gy,
        width: gw + 2 * border,
        height: gh + 2 * border,
    }
}

fn get_window_at_point(
    conn: &xcb::Connection, win: xcb::Window,
    x: i16, y: i16,
    remove_decorations: u32) -> HacksawResult {
    let tree = xcb::query_tree(conn, win).get_reply().unwrap();
    let children = tree
        .children()
        .iter()
        .filter(|&child| viewable(conn, *child))
        .filter(|&child| input_output(conn, *child))
        .filter_map(|&child| {
            let geom = get_window_geom(conn, child);
            if contained(geom.x, geom.y, geom.width as i16, geom.height as i16, x, y) {
                Some(geom)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut window = children[children.len() - 1];
    for _ in 0..remove_decorations {
        let tree = xcb::query_tree(conn, window.window).get_reply().unwrap();
        if tree.children_len() == 0 {
            break;
        }
        let firstborn = tree.children()[0];
        window = get_window_geom(conn, firstborn).relative_to(window);
    }
    window
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

    #[structopt(
        short = "f",
        long = "format",
        default_value = "%g",
        parse(try_from_str = "parse_format_string"),
        raw(allow_hyphen_values = "true"),
        help = "Output format. You can use %x for x-coordinate, %y for y-coordinate, %w for width, \
                %h for height, %i for selected window id, %g as a shorthand for %wx%h+%x+%y (the \
                default, X geometry) and %% for a literal '%'. Other %-codes will cause an error."
    )]
    format: Format,

    #[structopt(
        short = "r",
        long = "remove-decorations",
        default_value = "0",
        help = "Number of (nested) window manager frames to try and remove"
    )]
    remove_decorations: u32,
}

#[derive(Debug)]
struct ParseHexError {
    reason: String,
}

impl std::fmt::Display for ParseHexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl From<std::num::ParseIntError> for ParseHexError {
    fn from(err: std::num::ParseIntError) -> ParseHexError {
        ParseHexError {
            reason: err.to_string(),
        }
    }
}

/// Parse an HTML-color-like hex input
fn parse_hex(hex: &str) -> Result<u32, ParseHexError> {
    let hex = hex.trim_start_matches('#');
    let mut color;

    match hex.len() {
        3 | 4 => {
            color = 0x11 * u32::from_str_radix(&hex[2..3], 16)?
                + 0x11_00 * u32::from_str_radix(&hex[1..2], 16)?
                + 0x11_00_00 * u32::from_str_radix(&hex[0..1], 16)?;

            if hex.len() == 4 {
                color |= 0x11_00_00_00 * u32::from_str_radix(&hex[3..4], 16)?
            } else {
                color |= 0xFF_00_00_00;
            }
        }

        6 | 8 => {
            color = u32::from_str_radix(&hex, 16)?;

            if hex.len() == 6 {
                color |= 0xFF_00_00_00;
            }
        }

        _ => {
            return Err(ParseHexError {
                reason: "Bad hex colour".to_owned(),
            })
        }
    }

    Ok(color)
}

fn run() -> i32 {
    let opt = Opt::from_args();

    let line_width = opt.select_thickness;
    let guide_width = opt.guide_thickness;
    let line_colour = opt.line_colour;
    let format = opt.format.0;

    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let root = screen.root();

    let window = conn.generate_id();

    // TODO fix pointer-grab? bug where hacksaw hangs if mouse held down before run
    if !grab_pointer_set_cursor(&conn, root) {
        eprintln!("Failed to grab cursor after {} tries, giving up", CURSOR_GRAB_TRIES);
        return 1;
    }

    let scr_height = screen.height_in_pixels();
    let scr_width = screen.width_in_pixels();

    // TODO event handling for expose/keypress
    let values = [
        // ?RGB. First 4 bytes appear to do nothing
        (xcb::CW_BACK_PIXEL, line_colour),
        (
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_EXPOSURE
            | xcb::EVENT_MASK_KEY_PRESS // we'll need this later
            | xcb::EVENT_MASK_STRUCTURE_NOTIFY
            | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
        ),
        (xcb::CW_OVERRIDE_REDIRECT, 1u32), // Don't be window managed
    ];

    xcb::create_window(
        &conn,
        xcb::COPY_FROM_PARENT as u8, // usually 32?
        window,
        root,
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

    let mut start_x = 0;
    let mut start_y = 0;

    let mut top_y = 0;
    let mut left_x = 0;
    let mut right_x;
    let mut bot_y;

    let mut width = 0;
    let mut height = 0;

    let mut in_selection = false;
    let mut ignore_next_release = false;

    // TODO start drawing guides even before first event, without excess duplication
    // TODO draw rectangle around window under cursor
    loop {
        let ev = conn.wait_for_event().unwrap();
        match ev.response_type() {
            xcb::BUTTON_PRESS => {
                let button_press: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(&ev) };

                let detail = button_press.detail();
                if detail == 3 {
                    eprintln!("Exiting due to right click");
                    return 1;
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
                eprintln!("Exiting due to key press");
                return 1;
            }
            xcb::MOTION_NOTIFY => {
                let motion: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&ev) };
                let x = motion.event_x();
                let y = motion.event_y();

                // TODO investigate efficiency of let mut outside loop vs let inside
                left_x = x.min(start_x);
                top_y = y.min(start_y);
                right_x = x.max(start_x);
                bot_y = y.max(start_y);

                if in_selection {
                    width = (x - start_x).abs() as u16;
                    height = (y - start_y).abs() as u16;
                }

                let rects = match (opt.no_guides, in_selection) {
                    (_, true) => vec![
                        // Selection rectangle
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

    xcb::ungrab_pointer(&conn, xcb::CURRENT_TIME);
    xcb::unmap_window(&conn, window);
    xcb::destroy_window(&conn, window);
    conn.flush();

    loop {
        let ev = conn.wait_for_event().unwrap();
        match ev.response_type() {
            xcb::UNMAP_NOTIFY => {
                break;
            }
            xcb::DESTROY_NOTIFY => {
                break;
            }
            _ => (),
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(40));

    let result;
    if width == 0 && height == 0 {
        // Grab window under cursor
        result = get_window_at_point(&conn, root, start_x, start_y, opt.remove_decorations);
    } else {
        result = HacksawResult {
            window: root,
            x: left_x,
            y: top_y,
            width,
            height,
        };
    }

    // Now we have taken coordinates, we print them out
    println!("{}", fill_format_string(format, result));

    0
}

fn main() {
    std::process::exit(run());
}
