use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, none_of},
    combinator::{map, opt},
    error::context,
    multi::{many0, separated_list1},
    sequence::{delimited, preceded, tuple},
};

use crate::ast::*;

use super::{token::*, ty::parse_type, util::*, *};

fn parse_number_literal(input: Span) -> NotLocatedParseResult<Expression> {
    map(digit1, |str: Span| {
        Expression::NumberLiteral(NumberLiteralExpr {
            value: str.to_string(),
        })
    })(input)
}

fn parse_variable_ref(input: Span) -> NotLocatedParseResult<Expression> {
    map(parse_identifier, |name| {
        Expression::VariableRef(VariableRefExpr { name })
    })(input)
}

fn parse_arguments(input: Span) -> NotLocatedParseResult<Vec<LocatedExpr>> {
    let mut args = Vec::new();
    let mut s = input;
    loop {
        (s, _) = skip0(s)?;
        if rparen(s).is_ok() {
            break;
        }
        let (rest_s, expr) = parse_boxed_expression(s)?;
        args.push(expr);
        s = rest_s;
    }
    Ok((s, args))
}

pub(super) fn parse_intrinsic_op_expression(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            lparen,
            delimited(
                skip0,
                tuple((
                    alt((
                        map(plus, |_| BinaryOp::Add),
                        map(minus, |_| BinaryOp::Sub),
                        map(asterisk, |_| BinaryOp::Mul),
                        map(slash, |_| BinaryOp::Div),
                    )),
                    parse_boxed_expression,
                    parse_boxed_expression,
                )),
                skip0,
            ),
            rparen,
        ),
        |(binop, lhs, rhs)| {
            Expression::BinaryExpr(BinaryExpr {
                op: binop,
                lhs: lhs,
                rhs,
            })
        },
    )(input)
}

pub(super) fn parse_function_call_expression(input: Span) -> NotLocatedParseResult<Expression> {
    fn parse_generic_arguments(input: Span) -> NotLocatedParseResult<Vec<Located<UnresolvedType>>> {
        delimited(
            langlebracket,
            separated_list1(skip1, parse_type),
            ranglebracket,
        )(input)
    }
    map(
        delimited(
            lparen,
            tuple((
                parse_identifier,
                opt(parse_generic_arguments),
                parse_arguments,
            )),
            rparen,
        ),
        |(name, generic_args, args)| {
            Expression::Call(CallExpr {
                name,
                generic_args,
                args,
            })
        },
    )(input)
}

#[test]
fn test_parse_function_call_expression() {
    // write test
    let result = parse_function_call_expression(Span::new("(write \"%d\", x)"));
    assert!(result.is_ok());
    let (rest, expr) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    match expr {
        Expression::Call(call_expr) => {
            assert_eq!(call_expr.name, "write");
            assert!(call_expr.generic_args.is_none());
            assert_eq!(call_expr.args.len(), 2);
            assert_eq!(
                *call_expr.args[0].value,
                Expression::StringLiteral(StringLiteralExpr {
                    value: "%d".to_string()
                })
            );
            assert_eq!(
                *call_expr.args[1].value,
                Expression::VariableRef(VariableRefExpr {
                    name: "x".to_string()
                })
            );
        }
        _ => panic!("unexpected expression type"),
    }
}

fn parse_deref_expression(input: Span) -> NotLocatedParseResult<Expression> {
    map(preceded(asterisk, parse_boxed_expression), |expr| {
        Expression::DerefExpr(DerefExpr { target: expr })
    })(input)
}

fn parse_string_literal(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            skip0,
            delimited(
                doublequote,
                map(
                    many0(alt((
                        map(tag("\\\""), |_| "\"".to_string()),
                        map(tag("\\r"), |_| "\r".to_string()),
                        map(tag("\\n"), |_| "\n".to_string()),
                        map(tag("\\t"), |_| "\t".to_string()),
                        map(none_of("\""), |c| c.to_string()),
                    ))),
                    |chars| chars.join(""),
                ),
                doublequote,
            ),
            skip0,
        ),
        |value| Expression::StringLiteral(StringLiteralExpr { value }),
    )(input)
}

#[test]
fn test_parse_string_literal() {
    let result = parse_string_literal(Span::new("\"%d\""));
    assert!(result.is_ok());
    let (rest, expr) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    assert_eq!(
        expr,
        Expression::StringLiteral(StringLiteralExpr {
            value: "%d".to_string()
        })
    );
}

pub(super) fn parse_boxed_expression(input: Span) -> ParseResult<Box<Expression>> {
    let (rest, expr) = located(map(
        alt((
            context("deref", parse_deref_expression),
            context("string_literal", parse_string_literal),
            context("number_literal", parse_number_literal),
            context("call", parse_function_call_expression),
            context("binop", parse_intrinsic_op_expression),
            context("variable_ref", parse_variable_ref),
        )),
        |x| Box::new(x),
    ))(input)?;

    let (rest, opt_index_expr) = opt(located(index_access))(rest)?;

    if let Some(index_expr) = opt_index_expr {
        Ok((
            rest,
            Located {
                range: index_expr.range,
                value: Box::new(Expression::IndexAccess(IndexAccessExpr {
                    target: expr,
                    index: index_expr.value,
                })),
            },
        ))
    } else {
        Ok((rest, expr))
    }
}
