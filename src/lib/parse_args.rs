use super::parse_format::{parse_format_string, Format};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hacksaw", max_term_width = 80)]
pub(crate) struct Opt {
    #[structopt(
        short = "n",
        long = "no-guides",
        help = "Disable fighter pilot guide lines"
    )]
    pub(crate) no_guides: bool,

    #[structopt(
        short = "g",
        long = "guide-thickness",
        default_value = "1",
        help = "Thickness of fighter pilot guide lines"
    )]
    pub(crate) guide_thickness: u16,

    #[structopt(
        short = "s",
        long = "select-thickness",
        default_value = "1",
        help = "Thickness of selection box lines"
    )]
    pub(crate) select_thickness: u16,

    #[structopt(
        short = "c",
        long = "colour",
        default_value = "#7f7f7f",
        parse(try_from_str = parse_hex),
        help = "Hex colour of the lines (RGB or RGBA), '#' optional"
    )]
    pub(crate) line_colour: u32,

    #[structopt(
        short = "f",
        long = "format",
        default_value = "%g",
        parse(try_from_str = parse_format_string),
        allow_hyphen_values = true,
        help = "\
Output format. You can use:
      %x for x-coordinate,
      %y for y-coordinate,
      %w for width,
      %h for height,
      %i for selected window id,
      %g as a shorthand for %wx%h+%x+%y (X geometry),
      %% for a literal '%'.
Other %-codes will cause an error."
    )]
    pub(crate) format: Format,

    #[structopt(
        short = "r",
        long = "remove-decorations",
        default_value = "0",
        help = "Number of (nested) window manager frames to try and remove"
    )]
    pub(crate) remove_decorations: u32,
}

#[derive(Debug)]
struct ParseHexError<'a> {
    reason: String,
    source: &'a str,
}

impl<'a> std::fmt::Display for ParseHexError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Could not parse \"{}\": {}", self.source, self.reason)
    }
}

fn parse_hex_slice(slice: &str) -> Result<u32, ParseHexError> {
    u32::from_str_radix(slice, 16).map_err(|err| ParseHexError {
        reason: err.to_string(),
        source: slice,
    })
}

/// Parse an HTML-color-like hex input
fn parse_hex(hex: &str) -> Result<u32, ParseHexError> {
    let hex = hex.trim_start_matches('#');
    let mut color;

    match hex.len() {
        3 | 4 => {
            color = 0x11 * parse_hex_slice(&hex[2..3])?
                + 0x11_00 * parse_hex_slice(&hex[1..2])?
                + 0x11_00_00 * parse_hex_slice(&hex[0..1])?;

            if hex.len() == 4 {
                color |= 0x11_00_00_00 * parse_hex_slice(&hex[3..4])?
            } else {
                color |= 0xFF_00_00_00;
            }
        }

        6 | 8 => {
            color = parse_hex_slice(&hex)?;

            if hex.len() == 6 {
                color |= 0xFF_00_00_00;
            }
        }

        _ => {
            return Err(ParseHexError {
                reason: "Hex colour should have length 3, 4, 6, or 8".to_owned(),
                source: hex,
            })
        }
    }

    Ok(color)
}
