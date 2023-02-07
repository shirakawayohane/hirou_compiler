mod expression;
mod statement;
mod token;
mod toplevel;
mod ty;
mod util;

use nom::{
    combinator::{eof, map},
    error::{context, VerboseError},
    multi::{many1, many_till},
    sequence::delimited,
    IResult,
};

use nom_locate::{position, LocatedSpan};

use crate::ast::{BinaryOp, FunctionDecl, Located, Module, Statement, TopLevel};

use self::{toplevel::parse_toplevel, util::skip0};

pub type Span<'a> = LocatedSpan<&'a str>;

type ParseResult<'a, T> = IResult<Span<'a>, Located<'a, T>, VerboseError<Span<'a>>>;
type NotLocatedParseResult<'a, T> = IResult<Span<'a>, T, VerboseError<Span<'a>>>;

pub fn parse_module<'a>(input: Span<'a>) -> IResult<Span, Module, VerboseError<Span<'a>>> {
    context(
        "module",
        map(
            many1(delimited(skip0, parse_toplevel, skip0)),
            |toplevels| Module { toplevels },
        ),
    )(input)
}
