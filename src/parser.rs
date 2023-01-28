use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_till, take_till1},
    character::complete::digit1,
    character::complete::{char, line_ending},
    combinator::{eof, map, not, opt},
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::delimited,
    IResult, Parser,
};
use nom_locate::{position, LocatedSpan};

use crate::ast::{BinaryOp, Expression, Function, FunctionDecl, Module, Statement};

pub type Span<'a> = LocatedSpan<&'a str>;

pub struct Position {
    line: u32,
    col: usize,
}

pub struct Range<'a> {
    pub from: Position,
    pub to: Position,
    pub fragment: &'a str,
}

pub struct Located<'a, T> {
    range: Range<'a>,
    value: T,
}

fn located<'a, O, E>(
    mut parser: impl Parser<Span<'a>, O, SyntaxError<Span<'a>>>,
) -> impl FnMut(Span<'a>) -> ParseResult<Located<O>>
where E: ParseError<Span<'a>> {
    move |input: Span<'a>| {
        let (s, _) = skip0(input)?;
        let (s, from) = position(s)?;
        let input_at_start = s;
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
            fragment: &input_at_start[0..(to.location_offset() - from.location_offset())],
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

#[derive(Debug, Clone, Copy)]
pub enum SyntaxErrorKind {
    // InvalidSymbol,
    // ExpectChar { char: char },
    // Failed,
    UnMapped,
}

#[derive(Debug, Clone)]
pub struct SyntaxError<I> {
    loc: I,
    kind: SyntaxErrorKind,
    leaf_kinds: Vec<nom::error::ErrorKind>,
}

impl<'a, I> nom::error::ParseError<I> for SyntaxError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self {
            loc: input,
            kind: SyntaxErrorKind::UnMapped,
            leaf_kinds: vec![kind],
        }
    }

    fn append(input: I, kind: nom::error::ErrorKind, other: Self) -> Self {
        // TODO: cloneする必要ある？
        let mut kinds = other.leaf_kinds.clone();
        kinds.push(kind);
        Self {
            loc: input,
            kind: other.kind,
            leaf_kinds: kinds,
        }
    }
}

type ParseResult<'a, T> = IResult<Span<'a>, T, SyntaxError<Span<'a>>>;

fn comment(s: Span) -> ParseResult<()> {
    map(
        permutation((
            tag("//"),
            take_till(|c: char| c == '\r' || c == '\n'),
            alt((line_ending::<Span, SyntaxError<Span>>, eof)),
        )),
        |(_, _, _)| (),
    )(s)
}

fn multispace0(s: Span) -> ParseResult<()> {
    map(nom::character::complete::multispace0, |_| ())(s)
}

fn multispace1(s: Span) -> ParseResult<()> {
    map(nom::character::complete::multispace1, |_| ())(s)
}

fn parse_number_literal(input: Span) -> ParseResult<Expression> {
    let (s, num) = digit1(input)?;
    Ok((
        s,
        Expression::IntValue {
            value: num.parse::<i32>().unwrap(),
        },
    ))
}

fn skip0(input: Span) -> ParseResult<()> {
    let (s, ()) = map(many0(alt((comment, multispace1))), |_| ())(input)?;
    Ok((s, ()))
}

#[inline(always)]
fn lparen(input: Span) -> ParseResult<char> {
    char('(')(input)
}

#[inline(always)]
fn rparen(input: Span) -> ParseResult<char> {
    char(')')(input)
}

#[inline(always)]
fn lbracket(input: Span) -> ParseResult<char> {
    char('{')(input)
}

#[inline(always)]
fn rbracket(input: Span) -> ParseResult<char> {
    char('}')(input)
}

#[inline(always)]
fn comma(input: Span) -> ParseResult<char> {
    char(',')(input)
}

#[inline(always)]
fn semi(input: Span) -> ParseResult<char> {
    char(';')(input)
}

#[inline(always)]
fn equals(input: Span) -> ParseResult<char> {
    char('=')(input)
}

fn parse_symbol_name(input: Span) -> ParseResult<String> {
    let (s, _) = not(digit1)(input)?;
    map(
        take_till1(|x: char| !x.is_alphabetic() && !x.is_digit(10) && !['-', '_'].contains(&x)),
        |s: Span| s.to_string(),
    )(s)
}

fn parse_variable_ref(input: Span) -> ParseResult<Expression> {
    let (s, _) = skip0(input)?;
    map(parse_symbol_name, |name| Expression::VariableRef { name })(s)
}

fn parse_asignment(input: Span) -> ParseResult<Statement> {
    let (s, _) = skip0(input)?;
    let (s, name) = parse_symbol_name(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = equals(s)?;
    let (s, _) = multispace0(s)?;
    let (s, expression) = parse_expression(s)?;
    Ok((s, Statement::Asignment { name, expression }))
}

fn parse_variable_decl(input: Span) -> ParseResult<Statement> {
    let (s, _) = skip0(input)?;
    let (s, _from) = position(s)?;

    let (s, (_, _, name, _, _, _, expr)) = permutation((
        tag("int"),
        multispace0,
        parse_symbol_name,
        multispace0,
        char('='),
        multispace0,
        parse_expression,
    ))(s)?;

    Ok((s, Statement::VariableDecl { name, value: expr }))
}

fn fold_binexp(first: Expression, rest: &[(BinaryOp, Expression)]) -> Box<Expression> {
    if rest.len() == 0 {
        return Box::new(first);
    } else {
        let (binop, second) = rest.get(0).unwrap().clone();

        if rest.len() == 1 {
            return Box::new(Expression::BinaryExpr {
                op: binop,
                lhs: Box::new(first),
                rhs: Box::new(second),
            });
        }

        Box::new(Expression::BinaryExpr {
            op: binop,
            lhs: Box::new(first),
            rhs: fold_binexp(second, &rest[1..]),
        })
    }
}

fn parse_multiplicative_expression(input: Span) -> ParseResult<Expression> {
    let (s, _) = skip0(input)?;
    let (s, lhs) = parse_postfix_expression(s)?;
    let (s, rhss) = many0(map(
        permutation((
            alt((char('*'), char('/'))),
            multispace0,
            parse_postfix_expression,
        )),
        |(op, _, expression)| {
            (
                match op {
                    '*' => BinaryOp::Mul,
                    '/' => BinaryOp::Div,
                    _ => unreachable!(),
                },
                expression,
            )
        },
    ))(s)?;
    let exp = fold_binexp(lhs, &rhss);
    Ok((s, *exp))
}

fn parse_additive_expression(input: Span) -> ParseResult<Expression> {
    let (s, _) = skip0(input)?;
    let (s, lhs) = parse_multiplicative_expression(s)?;
    let (s, rhss) = many0(map(
        permutation((
            alt((char('+'), char('-'))),
            multispace0,
            parse_postfix_expression,
        )),
        |(op, _, expression)| {
            (
                match op {
                    '+' => BinaryOp::Add,
                    '-' => BinaryOp::Sub,
                    _ => unreachable!(),
                },
                expression,
            )
        },
    ))(s)?;
    let exp = fold_binexp(lhs, &rhss);
    Ok((s, *exp))
}

fn parse_primary_expression(input: Span) -> ParseResult<Expression> {
    let (s, _) = skip0(input)?;
    alt((
        parse_number_literal,
        delimited(char('('), parse_expression, char(')')),
        parse_variable_ref,
    ))(s)
}

fn parse_function_call(input: Span) -> ParseResult<(String, Vec<Expression>)> {
    fn parse_argument_list(input: Span) -> ParseResult<Vec<Expression>> {
        let (s, _) = skip0(input)?;
        let mut ret = Vec::new();
        let (s, first) = parse_expression(s)?;
        ret.push(first);
        let (s, rest) = many0(permutation((char(','), multispace0, parse_expression)))(s)?;

        for arg in rest {
            ret.push(arg.2);
        }

        Ok((s, ret))
    }
    let (s, _) = skip0(input)?;
    let (s, _from) = position(s)?;
    let (s, function_name) = parse_symbol_name(s)?;
    let (s, args) = delimited(char('('), parse_argument_list, char(')'))(s)?;

    Ok((s, (function_name, args)))
}

fn parse_function_call_expression(input: Span) -> ParseResult<Expression> {
    let (s, _) = skip0(input)?;
    let (s, (function_name, args)) = parse_function_call(s)?;
    Ok((
        s,
        Expression::CallExpr {
            name: function_name.to_string(),
            args: args,
        },
    ))
}

fn parse_postfix_expression(input: Span) -> ParseResult<Expression> {
    let (s, _) = skip0(input)?;
    let (s, _) = position(s)?;
    alt((parse_primary_expression, parse_function_call_expression))(s)
}

fn parse_expression(input: Span) -> ParseResult<Expression> {
    alt((parse_function_call_expression, parse_additive_expression))(input)
}

fn parse_function_decl(input: Span) -> ParseResult<FunctionDecl> {
    fn parse_argument_list(input: Span) -> ParseResult<Vec<String>> {
        fn parse_argument(input: Span) -> ParseResult<String> {
            let (s, _) = skip0(input)?;
            let (s, (_typename, _, name)) =
                permutation((tag("int"), multispace1, parse_symbol_name))(s)?;
            Ok((s, name))
        }

        let (s, _) = skip0(input)?;
        let (s, params) = delimited(
            lparen,
            delimited(
                multispace0,
                separated_list0(comma, parse_argument),
                multispace0,
            ),
            rparen,
        )(s)?;

        Ok((s, params))
    }
    let (s, _) = skip0(input)?;
    let (s, (_, name, params)) = permutation((
        tag("int"),
        delimited(multispace0, parse_symbol_name, multispace0),
        parse_argument_list,
    ))(s)?;

    Ok((s, FunctionDecl { name, params }))
}

fn parse_discarded_expression_statement(input: Span) -> ParseResult<Statement> {
    dbg!(input);
    let (s, _) = skip0(input)?;
    let (s, expression) = parse_expression(s)?;

    dbg!(&expression);
    Ok((s, Statement::DiscardedExpression { expression }))
}

fn parse_return_statement(input: Span) -> ParseResult<Statement> {
    let (s, _) = skip0(input)?;
    let (s, _) = tag("return")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, expression) = opt(parse_expression)(s)?;
    Ok((s, Statement::Return { expression }))
}

fn parse_statement(input: Span) -> ParseResult<Statement> {
    let (s, _) = skip0(input)?;

    let (s, statement) = alt((
        parse_return_statement,
        parse_asignment,
        parse_variable_decl,
        parse_discarded_expression_statement,
    ))(s)?;

    let (s, _) = multispace0(s)?;
    let (s, _) = semi(s)?;
    Ok((s, statement))
}

pub fn parse_block(input: Span) -> ParseResult<Vec<Statement>> {
    let (s, _) = skip0(input)?;
    delimited(
        lbracket,
        many0(delimited(skip0, parse_statement, skip0)),
        rbracket,
    )(s)
}

pub fn parse_function(input: Span) -> ParseResult<Function> {
    let (s, (decl, statements)) = permutation((parse_function_decl, parse_block))(input)?;
    Ok((
        s,
        Function {
            decl,
            body: statements,
        },
    ))
}

pub fn parse_module(input: Span) -> ParseResult<Module> {
    map(
        permutation((
            delimited(skip0, many0(parse_function), skip0),
            eof::<Span, SyntaxError<Span>>,
        )),
        |(functions, _)| Module { functions },
    )(input)
}
