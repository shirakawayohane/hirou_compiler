use super::*;

use nom::{
    bytes::complete::{tag, take},
    character::complete::{char, digit1},
    combinator::not,
    error::VerboseErrorKind,
    sequence::preceded,
};

// トークン間の空白をスキップし、本筋に集中するためのコンビネーター
fn token<'a, F, O>(f: F) -> impl FnMut(Span<'a>) -> NotLocatedParseResult<'a, ()>
where
    F: FnMut(Span<'a>) -> NotLocatedParseResult<'a, O>,
{
    map(preceded(skip0, f), |_| ())
}

macro_rules! token_char {
    ($name:ident, $arg:expr) => {
        #[inline(always)]
        pub(super) fn $name(input: Span) -> NotLocatedParseResult<()> {
            token(char($arg))(input)
        }
    };
}

macro_rules! token_tag {
    ($name:ident, $arg:expr) => {
        #[inline(always)]
        pub(super) fn $name(input: Span) -> NotLocatedParseResult<()> {
            token(tag($arg))(input)
        }
    };
}

#[test]
fn test_token_char() {
    assert!(lparen("(".into()).is_ok());
}

token_char!(lparen, '(');
token_char!(rparen, ')');
token_char!(lbracket, '{');
token_char!(rbracket, '}');
token_char!(lsqrbracket, '[');
token_char!(rsqrbracket, ']');
token_char!(langlebracket, '<');
token_char!(ranglebracket, '>');
token_char!(comma, ',');
token_char!(colon, ':');
token_char!(plus, '+');
token_char!(minus, '-');
token_char!(asterisk, '*');
token_char!(slash, '/');
token_char!(dot, '.');
token_char!(underscore, '_');
token_char!(ampersand, '&');
token_tag!(fn_token, "fn");
token_tag!(struct_token, "struct");
token_tag!(record_token, "record");
token_tag!(return_token, "return");
token_tag!(doublequote, "\"");
token_tag!(threedots, "...");
token_tag!(sizeof_token, "sizeof");
token_tag!(if_token, "if");
token_tag!(when_token, "when");
token_tag!(var_decl_token, ":=");
token_tag!(assign_token, ":=<");
token_tag!(and_token, "and");
token_tag!(or_token, "or");
token_tag!(not_token, "not");
token_tag!(eq_token, "=");
token_tag!(neq_token, "!=");
token_tag!(gte_token, ">=");
token_tag!(lte_token, "<=");
token_tag!(gt_token, ">");
token_tag!(lt_token, "<");
token_tag!(alloc_token, "alloc");
token_tag!(salloc_token, "salloc");
token_tag!(interface_token, "interface");
token_tag!(impl_token, "impl");
token_tag!(for_token, "for");
token_tag!(self_token, "self");
token_tag!(use_token, "use");
token_tag!(double_colon, "::");
token_tag!(while_token, "while");

pub(super) fn parse_identifier(input: Span) -> NotLocatedParseResult<String> {
    let (first_skipped, _) = skip0(input)?;
    let (s, _) = not(digit1)(first_skipped)?;
    let (s, _) = skip0(s)?;

    let mut take_count: usize = 0;
    let mut last_char: char = ' ';
    while take_count < s.fragment().len() {
        let c: char = s.fragment().chars().nth(take_count).unwrap();
        match c {
            '0'..='9' | '_' | '-' | '!' | '?' => take_count += 1,
            '>' => {
                if last_char != '-' {
                    break;
                }
                take_count += 1;
            }
            '<' => {
                let next_char = s.fragment().chars().nth(take_count + 1).unwrap();
                if next_char != '-' {
                    break;
                }
            }
            _ => {
                if c.is_alphabetic() {
                    take_count += 1;
                } else {
                    break;
                }
            }
        }
        last_char = c;
    }

    if take_count == 0 {
        return Err(nom::Err::Error(VerboseError {
            errors: vec![(s, VerboseErrorKind::Context("identifier"))],
        }));
    }

    map(take(take_count), |x: Span| x.to_string())(first_skipped)
}

#[test]
fn parse_identifier_test() {
    assert!(parse_identifier("print-i32".into()).is_ok());
    assert!(parse_identifier("buf[i]".into()).is_ok());
    let (rest, ident) = parse_identifier(" ->bool<T> (self: T)".into()).unwrap();
    assert_eq!(&ident, &"->bool");
    assert_eq!(rest.fragment(), &"<T> (self: T)");
    assert!(parse_identifier("}".into()).is_err());
    assert!(parse_identifier("{".into()).is_err());
    assert!(parse_identifier("(".into()).is_err());
    assert!(parse_identifier(")".into()).is_err());

    let (rest, ident) = parse_identifier("hoge,".into()).unwrap();
    assert_eq!(ident, "hoge");
    assert_eq!(rest.to_string().as_str(), ",");

    let (rest, ident) = parse_identifier("vec<T>".into()).unwrap();
    assert_eq!(ident, "vec");
    assert_eq!(rest.to_string().as_str(), "<T>");
}

pub(super) fn parse_namespace_path(input: Span) -> NotLocatedParseResult<crate::ast::NamespacePath> {
    use nom::multi::separated_list1;
    map(
        separated_list1(double_colon, parse_identifier),
        |segments| crate::ast::NamespacePath { segments }
    )(input)
}

#[test]
fn parse_namespace_path_test() {
    let (rest, path) = parse_namespace_path("Vec::push".into()).unwrap();
    assert_eq!(path.segments, vec!["Vec", "push"]);
    assert_eq!(rest.fragment(), &"");

    let (rest, path) = parse_namespace_path("std::Vec::push".into()).unwrap();
    assert_eq!(path.segments, vec!["std", "Vec", "push"]);
    assert_eq!(rest.fragment(), &"");

    let (rest, path) = parse_namespace_path("simple".into()).unwrap();
    assert_eq!(path.segments, vec!["simple"]);
    assert_eq!(rest.fragment(), &"");
}
