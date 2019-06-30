pub mod parser;
pub use self::parser::FormatToken;

extern crate nom;

// TODO window id
#[derive(Clone, Copy)]
pub struct HacksawResult {
    pub window: u32,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

pub fn fill_format_string(format: Vec<FormatToken>, result: HacksawResult) -> String {
    format
        .into_iter()
        .map(|token| match token {
            FormatToken::WindowId => result.window.to_string(),
            FormatToken::Geometry => format!(
                "{}x{}+{}+{}",
                result.width, result.height, result.x, result.y
            ),
            FormatToken::Width => result.width.to_string(),
            FormatToken::Height => result.height.to_string(),
            FormatToken::X => result.x.to_string(),
            FormatToken::Y => result.y.to_string(),
            FormatToken::Literal(s) => s,
        })
        .collect::<Vec<_>>()
        .join("")
}

// This newtype is needed to sidestep StructOpt's Vec behaviour
#[derive(Debug)]
pub struct Format(pub Vec<FormatToken>);

pub fn parse_format_string(input: &str) -> Result<Format, String> {
    match parser::parse_all(input) {
        Ok(("", v)) => Ok(Format { 0: v }),
        Err(s) => Err(format!("Format string parse error: {:?}", s)),
        Ok((s, _)) => Err(format!("Format string parse error near \"{}\"", s)),
    }
}
