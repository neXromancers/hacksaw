/// Token to determine how the output is formatted.
#[derive(Debug, PartialEq)]
pub enum FormatToken {
    WindowId,
    Geometry,
    Width,
    Height,
    X,
    Y,
    Literal(String),
}

// Get around structopt automatic Vec handling.
pub(crate) type Format = Vec<FormatToken>;

pub(crate) fn parse_format_string(input: &str) -> Result<Format, String> {
    let mut tokens = Vec::new();
    let mut input = input.as_bytes();

    loop {
        let (token, rest) = match input.split_first() {
            Some((b'%', format_rest)) => match format_rest.split_first() {
                Some((b'i', rest)) => (FormatToken::WindowId, rest),
                Some((b'g', rest)) => (FormatToken::Geometry, rest),
                Some((b'w', rest)) => (FormatToken::Width, rest),
                Some((b'h', rest)) => (FormatToken::Height, rest),
                Some((b'x', rest)) => (FormatToken::X, rest),
                Some((b'y', rest)) => (FormatToken::Y, rest),
                Some((b'%', rest)) => (FormatToken::Literal("%".to_owned()), rest),
                Some((c, _)) => break Err(format!("Unknown format '%{}'", *c as char)),
                None => break Err("Incorrectly terminated '%'".to_owned()),
            },
            Some((_, _)) => {
                let next_perc = input.iter().position(|&c| c == b'%');
                let (literal, rest) = input.split_at(next_perc.unwrap_or_else(|| input.len()));
                let literal = FormatToken::Literal(String::from_utf8_lossy(literal).into_owned());
                (literal, rest)
            }
            None => break Ok(tokens),
        };

        tokens.push(token);
        input = rest;
    }
}

#[test]
fn test_parse_format_string() {
    assert_eq!(
        parse_format_string("%wx%h+%x+%y"),
        Ok(vec![
            FormatToken::Width,
            FormatToken::Literal("x".into()),
            FormatToken::Height,
            FormatToken::Literal("+".into()),
            FormatToken::X,
            FormatToken::Literal("+".into()),
            FormatToken::Y,
        ])
    );

    assert_eq!(
        parse_format_string("%%h"),
        Ok(vec![
            FormatToken::Literal("%".into()),
            FormatToken::Literal("h".into())
        ])
    );

    assert_eq!(parse_format_string("%g"), Ok(vec![FormatToken::Geometry]));

    assert!(parse_format_string("%-").is_err());
    assert!(parse_format_string("%-").unwrap_err().contains("'%-'"));

    assert_eq!(parse_format_string(""), Ok(vec![]));

    assert_eq!(
        parse_format_string("hello world"),
        Ok(vec![FormatToken::Literal("hello world".into())])
    );
}
