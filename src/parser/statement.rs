use nom::{
    branch::{alt, permutation},
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1, space0, space1},
    combinator::{map, opt},
    error::context,
    sequence::preceded,
};

use crate::ast::Statement;

use super::{
    expression::parse_expression, token::*, ty::parse_type, util::*, Located, ParseResult, Span,
};

fn parse_asignment(input: Span) -> ParseResult<Located<Statement>> {
    located(map(
        permutation((
            parse_identifier,
            multispace0,
            equals,
            multispace0,
            parse_expression,
        )),
        |(name, _, _, _, expression)| Statement::Asignment {
            name,
            expression: expression.value,
        },
    ))(input)
}

fn parse_variable_decl(input: Span) -> ParseResult<Located<Statement>> {
    located(map(
        permutation((
            let_token,
            preceded(space1, parse_identifier),
            map(
                permutation((space0, colon, space0, parse_type)),
                |(_, _, _, ty)| ty,
            ),
            preceded(skip0, char('=')),
            preceded(skip0, parse_expression),
        )),
        |(_, name, ty, _, loc_expr)| Statement::VariableDecl {
            ty,
            name,
            value: loc_expr.value,
        },
    ))(input)
}

fn parse_discarded_expression_statement(input: Span) -> ParseResult<Located<Statement>> {
    located(map(parse_expression, |expr| {
        Statement::DiscardedExpression {
            expression: expr.value,
        }
    }))(input)
}

fn parse_return_statement(input: Span) -> ParseResult<Located<Statement>> {
    located(map(
        permutation((tag("return"), multispace1, opt(parse_expression))),
        |(_, _, opt_expr)| Statement::Return {
            expression: opt_expr.map(|x| x.value),
        },
    ))(input)
}

pub(super) fn parse_statement(input: Span) -> ParseResult<Located<Statement>> {
    context(
        "statement",
        map(
            permutation((
                alt((
                    parse_return_statement,
                    parse_asignment,
                    parse_variable_decl,
                    parse_discarded_expression_statement,
                )),
                multispace0,
                semicolon,
            )),
            |(loc_stmt, _, _)| loc_stmt,
        ),
    )(input)
}
