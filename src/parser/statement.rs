use nom::{
    branch::alt,
    combinator::{map, opt},
    error::context,
    sequence::tuple,
};

use crate::ast::{EffectStatement, ReturnStatement, Statement};

use super::{
    expression::parse_boxed_expression, token::*, util::*, NotLocatedParseResult, ParseResult, Span,
};

fn parse_return_statement(input: Span) -> NotLocatedParseResult<Statement> {
    map(
        tuple((return_token, skip1, opt(parse_boxed_expression))),
        |(_, _, opt_expr)| {
            Statement::Return(ReturnStatement {
                expression: opt_expr.map(|expr| expr.unbox()),
            })
        },
    )(input)
}

fn parse_effect_statement(input: Span) -> NotLocatedParseResult<Statement> {
    map(map(parse_boxed_expression, |x| x.unbox()), |loc_expr| {
        Statement::Effect(EffectStatement {
            expression: loc_expr,
        })
    })(input)
}

pub(super) fn parse_statement(input: Span) -> ParseResult<Statement> {
    located(alt((
        context("return_statement", parse_return_statement),
        context("effect_statement", parse_effect_statement),
    )))(input)
}
