use nom::{
    branch::{alt, permutation},
    character::complete::{multispace0, multispace1, space0, space1},
    combinator::{map, opt},
    error::context,
    multi::many0,
    sequence::preceded,
};

use crate::{ast::Statement, util::unbox_located_expression};

use super::{
    expression::{parse_expression, parse_function_call_expression},
    token::*,
    ty::parse_type,
    util::*,
    NotLocatedParseResult, ParseResult, Span,
};

fn parse_asignment(input: Span) -> NotLocatedParseResult<Statement> {
    map(
        permutation((
            many0(asterisk),
            parse_identifier,
            skip0,
            opt(index_access),
            skip0,
            multispace0,
            equals,
            multispace0,
            parse_expression,
        )),
        |(asterisks, name, _, index_access, _, _, _, _, value_expr)| Statement::Asignment {
            deref_count: asterisks.len() as u32,
            index_access,
            name,
            expression: value_expr,
        },
    )(input)
}

fn parse_variable_decl(input: Span) -> NotLocatedParseResult<Statement> {
    map(
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
    )(input)
}

fn parse_function_call_statement(input: Span) -> NotLocatedParseResult<Statement> {
    map(parse_function_call_expression, |expr| Statement::Effect {
        expression: unbox_located_expression(expr),
    })(input)
}

fn parse_return_statement(input: Span) -> NotLocatedParseResult<Statement> {
    map(
        permutation((return_token, multispace1, opt(parse_expression))),
        |(_, _, opt_expr)| Statement::Return {
            expression: opt_expr,
        },
    )(input)
}

pub(super) fn parse_statement(input: Span) -> ParseResult<Statement> {
    located(alt((
        context("function_call_statement", parse_function_call_statement),
        context("return_statement", parse_return_statement),
        context("variable_decl_statement", parse_variable_decl),
        context("assign_statement", parse_asignment),
    )))(input)
}
