use nom::{
    branch::{alt, permutation},
    character::complete::digit1,
    combinator::{map, opt},
    error::context,
    multi::many0,
    sequence::{delimited, pair, preceded},
};

use crate::{
    ast::Expression,
    util::{box_located_expression, unbox_located_expression},
};

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
        permutation((many0(asterisk), parse_identifier, skip0, opt(index_access))),
        |(asterisks, name, _, index_access)| {
            Box::new(Expression::VariableRef {
                deref_count: asterisks.len() as u32,
                index_access: index_access.map(box_located_expression),
                name,
            })
        },
    ))(input)
}

fn parse_arguments(input: Span) -> NotLocatedParseResult<Vec<Located<Box<Expression>>>> {
    let mut args = Vec::new();
    let mut s = input;
    loop {
        (s, _) = skip0(s)?;
        if rparen(s).is_ok() {
            break;
        }
        let (rest_s, v) = parse_expression(s)?;
        let located_boxed_expression = v.map(|x| Box::new(x));
        args.push(located_boxed_expression);
        s = rest_s;
    }
    Ok((s, args))
}

pub(super) fn parse_intrinsic_op_expression(input: Span) -> ParseResult<Box<Expression>> {
    located(map(
        delimited(
            lparen,
            delimited(
                skip0,
                pair(
                    alt((
                        map(plus, |_| BinaryOp::Add),
                        map(minus, |_| BinaryOp::Sub),
                        map(asterisk, |_| BinaryOp::Mul),
                        map(slash, |_| BinaryOp::Div),
                    )),
                    preceded(skip0, parse_arguments),
                ),
                skip0,
            ),
            rparen,
        ),
        |(binop, args)| Box::new(Expression::BinaryExpr { op: binop, args }),
    ))(input)
}

pub(super) fn parse_function_call_expression(input: Span) -> ParseResult<Box<Expression>> {
    located(map(
        delimited(
            lparen,
            pair(parse_identifier, preceded(skip0, parse_arguments)),
            rparen,
        ),
        |(name, args)| Box::new(Expression::CallExpr { name, args }),
    ))(input)
}

fn parse_boxed_expression(input: Span) -> ParseResult<Box<Expression>> {
    context(
        "expression",
        alt((
            parse_number_literal,
            parse_function_call_expression,
            parse_intrinsic_op_expression,
            parse_variable_ref,
        )),
    )(input)
}

pub(super) fn parse_expression(input: Span) -> ParseResult<Expression> {
    let (s, loc_expr) = parse_boxed_expression(input)?;
    Ok((s, unbox_located_expression(loc_expr)))
}
