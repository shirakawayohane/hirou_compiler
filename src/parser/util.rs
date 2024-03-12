use crate::ast::{Expression, Position, Range};

use super::{expression::parse_boxed_expression, token::*, *};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{line_ending, multispace1},
    combinator::eof,
    multi::many0,
    sequence::{preceded, tuple},
    Parser,
};

fn comment(s: Span<'_>) -> IResult<Span<'_>, (), VerboseError<Span<'_>>> {
    map(
        tuple((
            tag("//"),
            take_till(|c: char| c == '\r' || c == '\n'),
            alt((line_ending::<Span, VerboseError<Span>>, eof)),
        )),
        |(_, _, _)| (),
    )(s)
}

pub(super) fn skip0(input: Span<'_>) -> IResult<Span<'_>, (), VerboseError<Span<'_>>> {
    map(
        many0(alt((
            comment,
            map(tag(","), |_| ()),
            map(multispace1, |_| ()),
        ))),
        |_| (),
    )(input)
}

pub(super) fn skip1(input: Span<'_>) -> IResult<Span<'_>, (), VerboseError<Span<'_>>> {
    map(
        many1(alt((comment, map(comma, |_| ()), map(multispace1, |_| ())))),
        |_| (),
    )(input)
}

pub(super) fn located<'a, O>(
    mut parser: impl Parser<Span<'a>, O, VerboseError<Span<'a>>>,
) -> impl FnMut(Span<'a>) -> ParseResult<O> {
    move |input: Span<'a>| {
        let (s, _) = skip0(input)?;
        let (s, from) = position(s)?;
        let _input_at_start = s;
        let (s, output) = parser.parse(s)?;
        let (s, to) = position(s)?;
        let range = Range {
            from: Position {
                line: from.location_line(),
                col: from.get_column(),
            },
            to: Position {
                line: to.location_line(),
                col: to.get_column(),
            },
        };
        Ok((
            s,
            Located {
                range,
                value: output,
            },
        ))
    }
}

pub(super) fn index_access(input: Span<'_>) -> ParseResult<Box<Expression>> {
    delimited(lsqrbracket, parse_boxed_expression, rsqrbracket)(input)
}

pub(super) fn field_access(input: Span<'_>) -> NotLocatedParseResult<String> {
    preceded(dot, parse_identifier)(input)
}
