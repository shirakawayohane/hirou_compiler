use nom::{
    branch::{alt, permutation},
    character::complete::digit1,
    character::complete::{char, multispace0},
    combinator::map,
    error::context,
    multi::{many0, separated_list0},
    sequence::{delimited, pair},
};

use crate::ast::Expression;

use super::{token::*, util::*, *};

fn parse_number_literal(input: Span) -> ParseResult<Located<Expression>> {
    located(map(digit1, |str: Span| {
        let n = str.parse::<i32>().unwrap();
        Expression::NumberLiteral { value: n }
    }))(input)
}

fn parse_variable_ref(input: Span) -> ParseResult<Located<Expression>> {
    located(map(parse_identifier, |name| Expression::VariableRef {
        name,
    }))(input)
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
                expression.value,
            )
        },
    ))(s)?;
    let exp = fold_binexp(lhs.value, &rhss);
    Ok((s, *exp))
}

fn parse_additive_expression(input: Span) -> ParseResult<Located<Expression>> {
    fn parse_additive_expression_impl(input: Span) -> ParseResult<Expression> {
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
                    expression.value,
                )
            },
        ))(s)?;
        let exp = fold_binexp(lhs, &rhss);
        Ok((s, *exp))
    }
    located(parse_additive_expression_impl)(input)
}

fn parse_primary_expression(input: Span) -> ParseResult<Located<Expression>> {
    let (s, _) = skip0(input)?;
    alt((
        parse_number_literal,
        delimited(lparen, parse_expression, rparen),
        parse_variable_ref,
    ))(s)
}

fn parse_function_call_expression(input: Span) -> ParseResult<Located<Expression>> {
    located(map(
        pair(
            parse_identifier,
            delimited(
                lparen,
                separated_list0(comma, map(parse_expression, |loc_expr| loc_expr.value)),
                rparen,
            ),
        ),
        |(name, args)| Expression::CallExpr { name, args },
    ))(input)
}

fn parse_postfix_expression(input: Span) -> ParseResult<Located<Expression>> {
    alt((parse_primary_expression, parse_function_call_expression))(input)
}

pub(super) fn parse_expression(input: Span) -> ParseResult<Located<Expression>> {
    context(
        "expression",
        alt((parse_function_call_expression, parse_additive_expression)),
    )(input)
}
