use crate::{ast::*, parser::ty::parse_type};

use super::{statement::parse_statement, token::*, util::*, *};

use nom::{
    branch::alt,
    character::complete::{multispace0, space0},
    combinator::{cut, map, opt},
    error::context,
    multi::separated_list0,
    sequence::{delimited, pair, tuple},
};

fn parse_generic_argument(input: Span) -> ParseResult<GenericArgument> {
    located(context(
        "generic_argument",
        map(parse_identifier, |name| GenericArgument { name }),
    ))(input)
}
fn parse_generic_arguments<'a>(
    input: Span<'a>,
) -> NotLocatedParseResult<Vec<Located<GenericArgument>>> {
    delimited(
        langlebracket,
        separated_list0(comma, parse_generic_argument),
        ranglebracket,
    )(input)
}

fn parse_argument(input: Span) -> NotLocatedParseResult<Argument> {
    alt((
        map(threedots, |_| Argument::VarArgs),
        map(
            tuple((parse_identifier, colon, parse_type)),
            |(name, _, ty)| Argument::Normal(ty, name),
        ),
    ))(input)
}

fn parse_arguments(input: Span) -> NotLocatedParseResult<Vec<Argument>> {
    let mut args = Vec::new();
    let (mut rest, _) = lparen(input)?;
    loop {
        (rest, _) = skip0(rest)?;
        if rest.starts_with(")") {
            break;
        }
        let arg;
        (rest, arg) = parse_argument(rest)?;
        args.push(arg);
    }
    let (rest, _) = rparen(rest)?;
    Ok((rest, args))
}

#[test]
fn test_parse_single_argument() {
    let input = "x:i32,".into();
    let result = parse_argument(input);
    assert!(result.is_ok());
    let (rest, arg) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), ",");
    let (ty, name) = match arg {
        Argument::Normal(ty, name) => (ty, name),
        _ => panic!("unexpected argument type"),
    };
    assert_eq!(name, "x");
    assert_eq!(
        ty.value,
        UnresolvedType::TypeRef(TypeRef {
            name: "i32".into(),
            generic_args: None
        })
    );
}

#[test]
fn test_parse_multiple_arguments() {
    let input = "(x:i32,y:f64)".into();
    let result = parse_arguments(input);
    assert!(result.is_ok());
    let (_, args) = result.unwrap();
    assert_eq!(args.len(), 2);
    let (ty, name) = match &args[0] {
        Argument::Normal(ty, name) => (ty, name),
        _ => panic!("unexpected argument type"),
    };
    assert_eq!(name, "x");
    assert_eq!(
        ty.value,
        UnresolvedType::TypeRef(TypeRef {
            name: "i32".into(),
            generic_args: None
        })
    );
    let (ty, name) = match &args[1] {
        Argument::Normal(ty, name) => (ty, name),
        _ => panic!("unexpected argument type"),
    };
    assert_eq!(name, "y");
    assert_eq!(
        ty.value,
        UnresolvedType::TypeRef(TypeRef {
            name: "f64".into(),
            generic_args: None
        })
    );
}

#[test]
fn test_parse_argument_with_error() {
    let input = "x i32".into();
    let result = parse_argument(input);
    assert!(result.is_err());
}

fn parse_function_decl(input: Span) -> ParseResult<FunctionDecl> {
    context(
        "function_decl",
        located(map(
            tuple((
                fn_token,
                delimited(multispace0, parse_identifier, multispace0),
                opt(parse_generic_arguments),
                // params
                parse_arguments,
                map(
                    tuple((space0, colon, space0, parse_type)),
                    |(_, _, _, ty)| ty,
                ),
            )),
            |(_, name, generic_args, params, ty)| FunctionDecl {
                name: name.into(),
                generic_args,
                args: params,
                return_type: ty,
                intrinsic: false,
            },
        )),
    )(input)
}

pub fn parse_block(input: Span) -> NotLocatedParseResult<Vec<Located<Statement>>> {
    let (s, _) = skip0(input)?;
    let (s, _) = lbracket(s)?;
    let (s, _) = skip0(s)?;
    let mut statements = Vec::new();
    let mut s = s;
    while !s.starts_with("}") {
        let (rest, stmt) = parse_statement(s)?;
        statements.push(stmt);
        (s, _) = skip0(rest)?;
    }
    let (s, _) = rbracket(s)?;
    Ok((s, statements))
}

fn parse_function(input: Span) -> ParseResult<TopLevel> {
    located(context(
        "function",
        map(
            tuple((parse_function_decl, skip0, parse_block)),
            |(decl, _, body)| {
                TopLevel::Function(Function {
                    decl: decl.value,
                    body,
                })
            },
        ),
    ))(input)
}

pub(crate) fn parse_toplevel(input: Span) -> ParseResult<TopLevel> {
    context("toplevel", parse_function)(input)
}

#[test]
fn test_parse_toplevel() {
    let result = parse_toplevel(
        "
fn print-i32(s: *u8, n: i32): void {
    (printf 1, n)
}
"
        .into(),
    );
    assert!(result.is_ok());
}
