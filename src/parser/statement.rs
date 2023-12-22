use nom::{
    branch::alt,
    character::complete::{multispace0, multispace1, space0, space1},
    combinator::{map, opt},
    error::context,
    multi::many0,
    sequence::{preceded, tuple},
};

use crate::ast::{
    AssignmentStatement, EffectStatement, ReturnStatement, Statement, VariableDeclStatement,
};

use super::{
    expression::{parse_boxed_expression, parse_function_call_expression},
    token::*,
    ty::parse_type,
    util::*,
    NotLocatedParseResult, ParseResult, Span,
};

fn parse_asignment(input: Span) -> NotLocatedParseResult<Statement> {
    map(
        tuple((
            many0(asterisk),
            parse_identifier,
            skip0,
            opt(index_access),
            skip0,
            multispace0,
            equals,
            multispace0,
            parse_boxed_expression,
        )),
        |(asterisks, name, _, index_access, _, _, _, _, value_expr)| {
            Statement::Assignment(AssignmentStatement {
                deref_count: asterisks.len() as u32,
                index_access: index_access.map(|expr| expr.unbox()),
                name,
                expression: value_expr.unbox(),
            })
        },
    )(input)
}

fn parse_variable_decl(input: Span) -> NotLocatedParseResult<Statement> {
    map(
        tuple((
            let_token,
            preceded(space1, parse_identifier),
            context(
                "type_annotation",
                map(
                    tuple((space0, colon, space0, parse_type)),
                    |(_, _, _, ty)| ty,
                ),
            ),
            equals,
            preceded(skip0, parse_boxed_expression),
        )),
        |(_, name, ty, _, expression)| {
            Statement::VariableDecl(VariableDeclStatement {
                ty,
                name,
                value: expression.unbox(),
            })
        },
    )(input)
}

fn parse_function_call_statement(input: Span) -> NotLocatedParseResult<Statement> {
    map(located(parse_function_call_expression), |loc_expr| {
        Statement::Effect(EffectStatement {
            expression: loc_expr,
        })
    })(input)
}

fn parse_return_statement(input: Span) -> NotLocatedParseResult<Statement> {
    map(
        tuple((return_token, multispace1, opt(parse_boxed_expression))),
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
        context("function_call_statement", parse_function_call_statement),
        context("return_statement", parse_return_statement),
        context("variable_decl_statement", parse_variable_decl),
        context("assign_statement", parse_asignment),
        context("effect_statement", parse_effect_statement),
    )))(input)
}
