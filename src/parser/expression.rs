use nom::{
    branch::{alt, permutation},
    character::complete::digit1,
    combinator::{map, opt},
    error::context,
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded},
};

use crate::{ast::Expression, util::unbox_located_expression};

use super::{token::*, util::*, *};

fn parse_number_literal(input: Span) -> ParseResult<Box<Expression>> {
    located(map(digit1, |str: Span| {
        Box::new(Expression::NumberLiteral {
            value: str.to_string(),
        })
    }))(input)
}

fn parse_variable_ref(input: Span) -> ParseResult<Box<Expression>> {
    located(map(
        permutation((many0(asterisk), parse_identifier)),
        |(asterisks, name)| {
            Box::new(Expression::VariableRef {
                deref_count: asterisks.len() as u32,
                name,
            })
        },
    ))(input)
}

fn parse_multiplicative_expression(input: Span) -> ParseResult<Box<Expression>> {
    located(map(
        permutation((
            parse_postfix_expression,
            skip0,
            opt(permutation((
                alt((
                    map(asterisk, |_| BinaryOp::Mul),
                    map(slash, |_| BinaryOp::Div),
                )),
                preceded(skip0, parse_postfix_expression),
            ))),
        )),
        |(lhs, _, op_and_rhs)| {
            if let Some((op, rhs)) = op_and_rhs {
                Box::new(Expression::BinaryExpr { op, lhs, rhs })
            } else {
                lhs.value
            }
        },
    ))(input)
}

fn parse_additive_expression(input: Span) -> ParseResult<Box<Expression>> {
    located(map(
        permutation((
            parse_multiplicative_expression,
            skip0,
            opt(permutation((
                alt((map(plus, |_| BinaryOp::Add), map(minus, |_| BinaryOp::Sub))),
                preceded(skip0, parse_postfix_expression),
            ))),
        )),
        |(lhs, _, op_and_rhs)| {
            if let Some((op, rhs)) = op_and_rhs {
                Box::new(Expression::BinaryExpr { op, lhs, rhs })
            } else {
                lhs.value
            }
        },
    ))(input)
}

fn parse_primary_expression(input: Span) -> ParseResult<Box<Expression>> {
    let (s, _) = skip0(input)?;
    alt((
        parse_number_literal,
        delimited(lparen, parse_boxed_expression, rparen),
        parse_variable_ref,
    ))(s)
}

fn parse_function_call_expression(input: Span) -> ParseResult<Box<Expression>> {
    located(map(
        pair(
            parse_identifier,
            delimited(
                lparen,
                separated_list0(comma, map(parse_expression, |loc_expr| loc_expr.value)),
                rparen,
            ),
        ),
        |(name, args)| Box::new(Expression::CallExpr { name, args }),
    ))(input)
}

fn parse_postfix_expression(input: Span) -> ParseResult<Box<Expression>> {
    alt((parse_primary_expression, parse_function_call_expression))(input)
}

fn parse_boxed_expression(input: Span) -> ParseResult<Box<Expression>> {
    context(
        "expression",
        alt((parse_function_call_expression, parse_additive_expression)),
    )(input)
}

pub(super) fn parse_expression(input: Span) -> ParseResult<Expression> {
    let (s, loc_expr) = parse_boxed_expression(input)?;
    Ok((s, unbox_located_expression(loc_expr)))
}
