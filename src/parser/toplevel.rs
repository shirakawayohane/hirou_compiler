use crate::{
    ast::*,
    common::{AllocMode, StructKind},
    parser::ty::{parse_generic_argument_decls, parse_type},
};

use super::{statement::parse_statement, token::*, util::*, *};

use nom::{
    branch::alt,
    combinator::{cut, opt, peek},
    error::context,
    multi::separated_list1,
    sequence::{preceded, tuple},
};

#[test]
fn test_parse_generic_arguments() {
    let result = parse_generic_argument_decls(Span::new("<T>"));
    assert!(result.is_ok());
    let (rest, args) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].value.name, "T");

    let result = parse_generic_argument_decls(Span::new("<T, U>"));
    assert!(result.is_ok());
    let (rest, args) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].value.name, "T");
    assert_eq!(args[1].value.name, "U");
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
        if rest.starts_with(')') {
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
fn test_parse_zero_argument() {
    let result = parse_arguments(Span::new("()"));
    assert!(result.is_ok());
    let (rest, args) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    assert_eq!(args.len(), 0);
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

fn parse_alloc_mode(input: Span) -> NotLocatedParseResult<AllocMode> {
    alt((
        map(alloc_token, |_| AllocMode::Heap),
        map(salloc_token, |_| AllocMode::Stack),
    ))(input)
}

fn parse_function_decl(input: Span) -> ParseResult<FunctionDecl> {
    context(
        "function_decl",
        located(map(
            tuple((
                opt(parse_alloc_mode),
                fn_token,
                parse_identifier,
                opt(parse_generic_argument_decls),
                // params
                parse_arguments,
                map(tuple((colon, parse_type)), |(_, ty)| ty),
            )),
            |(alloc_mode, _, name, generic_args, params, ty)| FunctionDecl {
                alloc_mode,
                name,
                generic_args,
                args: params,
                return_type: ty,
                is_intrinsic: false,
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
    while !s.starts_with('}') {
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
            tuple((parse_function_decl, skip0, cut(parse_block))),
            |(decl, _, body)| {
                TopLevel::Function(Function {
                    decl: decl.value,
                    body,
                })
            },
        ),
    ))(input)
}

fn parse_interface(input: Span) -> ParseResult<TopLevel> {
    let (s, _) = peek(interface_token)(input)?;
    cut(located(context(
        "interface",
        map(
            tuple((
                context("interface", interface_token),
                context("identifier", parse_identifier),
                context("generic_arguments", parse_generic_argument_decls),
                context("arguments", parse_arguments),
                preceded(colon, parse_type),
            )),
            |(_, name, generic_args, args, return_type)| {
                TopLevel::Interface(Interface {
                    name,
                    generic_args,
                    args,
                    return_type,
                })
            },
        ),
    )))(s)
}

#[test]
fn parse_interface_invalid_input() {
    assert!(dbg!(parse_interface(
        "interface ->bool<T> (self: T) : bool".into()
    ))
    .is_ok());
}

fn parse_impl(input: Span) -> ParseResult<TopLevel> {
    let (s, _) = peek(impl_token)(input)?;
    cut(located(context(
        "implementation",
        map(
            tuple((
                opt(parse_alloc_mode),
                impl_token,
                parse_identifier,
                opt(parse_generic_argument_decls),
                for_token,
                parse_type,
                parse_arguments,
                preceded(colon, parse_type),
                parse_block,
            )),
            |(alloc_mode, _, name, generic_args, _, target_ty, args, return_type, body)| {
                TopLevel::Implemantation(Implementation {
                    decl: ImplementationDecl {
                        alloc_mode,
                        name,
                        generic_args,
                        target_ty,
                        args,
                        return_type,
                    },
                    body,
                })
            },
        ),
    )))(s)
}

fn parse_field(input: Span) -> NotLocatedParseResult<(String, Located<UnresolvedType>)> {
    map(
        tuple((parse_identifier, colon, located(parse_type))),
        |(name, _, ty)| (name, ty.value),
    )(input)
}
fn parse_fields(input: Span) -> NotLocatedParseResult<Vec<(String, Located<UnresolvedType>)>> {
    let mut fields = Vec::new();
    let mut rest = input;
    loop {
        (rest, _) = skip0(rest)?;
        if rest.starts_with('}') {
            break;
        }
        let field;
        (rest, field) = parse_field(rest)?;
        fields.push(field);
    }
    Ok((rest, fields))
}

fn parse_struct(input: Span) -> ParseResult<TopLevel> {
    let (s, _) = peek(alt((struct_token, record_token)))(input)?;
    context(
        "struct",
        cut(located(map(
            tuple((
                alt((
                    map(struct_token, |_| StructKind::Struct),
                    map(record_token, |_| StructKind::Record),
                )),
                parse_identifier,
                opt(parse_generic_argument_decls),
                delimited(lbracket, parse_fields, rbracket),
            )),
            |(struct_kind, name, generic_args, fields)| {
                TopLevel::TypeDef(TypeDef {
                    kind: TypeDefKind::StructLike(StructLikeTypeDef {
                        struct_kind,
                        generic_args,
                        fields,
                    }),
                    name,
                })
            },
        ))),
    )(s)
}

#[test]
fn test_parse_struct() {
    assert!(matches!(
        parse_toplevel("struct Vec<T> { size: i32, data: T }".into())
            .unwrap()
            .1
            .value,
        TopLevel::TypeDef(TypeDef {
            name: _,
            kind: TypeDefKind::StructLike(StructLikeTypeDef {
                struct_kind: StructKind::Struct,
                generic_args: _,
                fields: _
            })
        })
    ))
}

pub(crate) fn parse_toplevel(input: Span) -> ParseResult<TopLevel> {
    context(
        "toplevel",
        alt((parse_function, parse_struct, parse_interface, parse_impl)),
    )(input)
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

    let result = parse_toplevel("struct Vec<T> { size: i32, data: *T }".into());
    assert!(result.is_ok());

    let result = parse_function(
        "
        fn vec<T>(): Vec<T> {
        }"
        .into(),
    );
    assert!(result.is_ok())
}

#[test]
fn test_parse_record() {
    assert!(matches!(
        parse_toplevel(r#"record A { v: i32 }"#.into())
            .unwrap()
            .1
            .value,
        TopLevel::TypeDef(TypeDef {
            name: _,
            kind: TypeDefKind::StructLike(StructLikeTypeDef {
                struct_kind: StructKind::Record,
                generic_args: _,
                fields: _
            })
        })
    ))
}
