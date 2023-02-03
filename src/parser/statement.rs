use nom::{
    branch::{alt, permutation},
    bytes::complete::tag,
    character::complete::{multispace0, multispace1, space0, space1},
    combinator::{map, opt},
    error::context,
    multi::many0,
    sequence::preceded,
};

use crate::ast::{Statement};

use super::{expression::parse_expression, token::*, ty::parse_type, util::*, ParseResult, Span};

fn parse_asignment(input: Span) -> ParseResult<Statement> {
    located(map(
        permutation((
            many0(asterisk),
            parse_identifier,
            multispace0,
            equals,
            multispace0,
            parse_expression,
        )),
        |(asterisks, name, _, _, _, expression)| Statement::Asignment {
            deref_count: asterisks.len() as u32,
            name,
            expression,
        },
    ))(input)
}

fn parse_variable_decl(input: Span) -> ParseResult<Statement> {
    located(map(
        permutation((
            let_token,
            preceded(space1, parse_identifier),
            context(
                "type_annotation",
                map(
                    permutation((space0, colon, space0, parse_type)),
                    |(_, _, _, ty)| ty,
                ),
            ),
            preceded(skip0, equals),
            preceded(skip0, parse_expression),
        )),
        |(_, name, ty, _, expression)| Statement::VariableDecl {
            ty,
            name,
            value: expression,
        },
    ))(input)
}

fn parse_discarded_expression_statement(input: Span) -> ParseResult<Statement> {
    located(map(parse_expression, |expression| {
        Statement::DiscardedExpression { expression }
    }))(input)
}

fn parse_return_statement(input: Span) -> ParseResult<Statement> {
    located(map(
        permutation((tag("return"), multispace1, opt(parse_expression))),
        |(_, _, opt_expr)| Statement::Return {
            expression: opt_expr,
        },
    ))(input)
}

pub(super) fn parse_statement(input: Span) -> ParseResult<Statement> {
    map(
        permutation((
            alt((
                context("return_statement", parse_return_statement),
                context("assignment", parse_asignment),
                context("variable_decl", parse_variable_decl),
                context("discarded_expression", parse_discarded_expression_statement),
            )),
            multispace0,
            semicolon,
        )),
        |(loc_stmt, _, _)| loc_stmt,
    )(input)
}
