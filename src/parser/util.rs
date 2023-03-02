use crate::ast::{Expression, Located, Position, Range};

use super::{
    expression::parse_expression,
    token::{lsqrbracket, rsqrbracket},
    *,
};
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_till},
    character::complete::{line_ending, multispace1},
    combinator::{eof, map},
    error::VerboseError,
    multi::many0,
    Parser,
};

fn comment<'a>(s: Span<'a>) -> IResult<Span<'a>, (), VerboseError<Span<'a>>> {
    map(
        permutation((
            tag("//"),
            take_till(|c: char| c == '\r' || c == '\n'),
            alt((line_ending::<Span, VerboseError<Span>>, eof)),
        )),
        |(_, _, _)| (),
    )(s)
}

pub(super) fn skip0<'a>(input: Span<'a>) -> IResult<Span<'a>, (), VerboseError<Span<'a>>> {
    map(many0(alt((comment, map(multispace1, |_| ())))), |_| ())(input)
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

pub(super) fn index_access<'a>(input: Span<'a>) -> NotLocatedParseResult<Located<Expression>> {
    delimited(
        lsqrbracket,
        delimited(skip0, parse_expression, skip0),
        rsqrbracket,
    )(input)
}
