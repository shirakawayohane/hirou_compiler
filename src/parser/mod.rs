mod expression;
mod statement;
mod token;
mod toplevel;
mod util;

use nom::{
    combinator::{eof, map},
    error::{context, VerboseError},
    multi::many_till,
    sequence::delimited,
    IResult,
};
use nom_locate::{position, LocatedSpan};

use crate::ast::{BinaryOp, FunctionDecl, Module, Statement, TopLevel};

use self::{toplevel::parse_toplevel, util::skip0};

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    line: u32,
    col: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Range<'a> {
    pub from: Position,
    pub to: Position,
    pub fragment: &'a str,
}

#[derive(Debug)]
pub struct Located<'a, T> {
    range: Range<'a>,
    value: T,
}

type ParseResult<'a, T> = IResult<Span<'a>, T, VerboseError<Span<'a>>>;

pub fn parse_module(input: Span) -> ParseResult<Module> {
    context(
        "module",
        map(
            many_till(
                delimited(skip0, parse_toplevel, skip0),
                eof::<Span, VerboseError<Span>>,
            ),
            |(toplevels, _)| Module { toplevels },
        ),
    )(input)
}
