use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{i64, line_ending, not_line_ending, u64},
    multi::count,
    sequence::{preceded, terminated},
};

pub enum RespFrame {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RespFrame>),
}

impl RespFrame {
    pub fn as_string(self) -> Option<String> {
        match self {
            RespFrame::BulkString(s) | RespFrame::SimpleString(s) => Some(s),
            _ => None,
        }
    }
}

pub fn parse_frame(input: &str) -> IResult<&str, RespFrame> {
    alt((
        parse_simple_string,
        parse_simple_error,
        parse_integer,
        parse_bulk_string,
        parse_arrays,
    ))
    .parse(input)
}

fn parse_simple_string(input: &str) -> IResult<&str, RespFrame> {
    let (input, value) =
        preceded(tag("+"), terminated(not_line_ending, line_ending)).parse(input)?;

    Ok((input, RespFrame::SimpleString(value.to_string())))
}

fn parse_simple_error(input: &str) -> IResult<&str, RespFrame> {
    let (input, value) =
        preceded(tag("-"), terminated(not_line_ending, line_ending)).parse(input)?;

    Ok((input, RespFrame::Error(value.to_string())))
}

fn parse_integer(input: &str) -> IResult<&str, RespFrame> {
    let (input, value) = preceded(tag(":"), terminated(i64, line_ending)).parse(input)?;

    Ok((input, RespFrame::Integer(value)))
}

fn parse_bulk_string(input: &str) -> IResult<&str, RespFrame> {
    let (input, value) = preceded(tag("$"), terminated(u64, line_ending)).parse(input)?;

    let (input, value) = terminated(take(value as usize), line_ending).parse(input)?;

    Ok((input, RespFrame::BulkString(value.to_string())))
}

fn parse_arrays(input: &str) -> IResult<&str, RespFrame> {
    let (input, value) = preceded(tag("*"), terminated(u64, line_ending)).parse(input)?;

    let (input, value) = count(parse_frame, value as usize).parse(input)?;

    Ok((input, RespFrame::Array(value)))
}
