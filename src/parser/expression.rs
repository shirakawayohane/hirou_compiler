use nom::{
    branch::{alt, permutation},
    character::complete::digit1,
    combinator::{map, opt},
    multi::{separated_list1},
    sequence::{delimited, preceded, tuple},
};

use crate::{ast::*};

use super::{token::*, ty::parse_type, util::*, *};

fn parse_number_literal(input: Span) -> NotLocatedParseResult<Expression> {
    map(digit1, |str: Span| {
        Expression::NumberLiteral(NumberLiteralExpr {
            value: str.to_string(),
        })
    })(input)
}

fn parse_variable_ref(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        permutation((parse_identifier, skip0, opt(index_access))),
        |(name, _, _index_access)| Expression::VariableRef(VariableRefExpr { name }),
    )(input)
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
            preceded(skip0, rparen),
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
                opt(preceded(skip0, parse_generic_arguments)),
                preceded(skip0, parse_arguments),
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

pub(super) fn parse_boxed_expression(input: Span) -> ParseResult<Box<Expression>> {
    located(map(
        alt((
            parse_number_literal,
            parse_function_call_expression,
            parse_intrinsic_op_expression,
            parse_variable_ref,
        )),
        |x| Box::new(x),
    ))(input)
}

pub(super) fn parse_expression(input: Span) -> NotLocatedParseResult<Expression> {
    alt((
        parse_number_literal,
        parse_function_call_expression,
        parse_intrinsic_op_expression,
        parse_variable_ref,
    ))(input)
}
