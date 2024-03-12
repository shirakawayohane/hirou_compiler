mod expression;
mod statement;
mod token;
mod toplevel;
mod ty;
mod util;

use nom::{combinator::map, error::VerboseError, multi::many1, sequence::delimited, IResult};

use nom_locate::{position, LocatedSpan};

use crate::ast::{Located, Module};

use self::{toplevel::parse_toplevel, util::skip0};

pub type Span<'a> = LocatedSpan<&'a str>;

type ParseResult<'a, T> = IResult<Span<'a>, Located<T>, VerboseError<Span<'a>>>;
type NotLocatedParseResult<'a, T> = IResult<Span<'a>, T, VerboseError<Span<'a>>>;

pub fn parse_module(input: Span<'_>) -> IResult<Span, Module, VerboseError<Span<'_>>> {
    let mut toplevels = Vec::new();
    let mut rest = input;
    loop {
        (rest, _) = skip0(rest)?;
        if rest.is_empty() {
            break;
        }
        let toplevel;
        (rest, toplevel) = parse_toplevel(rest)?;
        toplevels.push(toplevel);
    }
    Ok((rest, Module { toplevels }))
}

#[test]
fn test_parse_module() {
    let input = Span::new(
        "
fn sub(): i32 { }
// comment
fn main():void {}
",
    );
    let result = parse_module(input);
    assert!(result.is_ok());
    let (rest, module) = result.unwrap();
    assert!(rest.is_empty());
    assert_eq!(module.toplevels.len(), 2);
}
