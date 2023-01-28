use super::{statement::parse_statement, token::*, util::*, *};

use nom::{
    branch::permutation,
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::map,
    error::context,
    multi::{many0, separated_list0},
    sequence::delimited,
};

fn parse_function_decl(input: Span) -> ParseResult<FunctionDecl> {
    fn parse_argument_list(input: Span) -> ParseResult<Vec<String>> {
        fn parse_argument(input: Span) -> ParseResult<String> {
            let (s, _) = skip0(input)?;
            let (s, (_typename, _, name)) =
                permutation((tag("int"), multispace1, parse_identifier))(s)?;
            Ok((s, name))
        }

        let (s, _) = skip0(input)?;
        let (s, params) = delimited(
            token::lparen,
            delimited(
                multispace0,
                separated_list0(comma, parse_argument),
                multispace0,
            ),
            token::rparen,
        )(s)?;

        Ok((s, params))
    }
    let (s, _) = skip0(input)?;
    let (s, (_, name, params)) = permutation((
        tag("int"),
        delimited(multispace0, parse_identifier, multispace0),
        parse_argument_list,
    ))(s)?;

    Ok((s, FunctionDecl { name, params }))
}

pub fn parse_block(input: Span) -> ParseResult<Vec<Statement>> {
    context(
        "block",
        delimited(
            lbracket,
            many0(delimited(
                skip0,
                map(parse_statement, |loc_stmt| loc_stmt.value),
                skip0,
            )),
            rbracket,
        ),
    )(input)
}

fn parse_function(input: Span) -> ParseResult<TopLevel> {
    context(
        "function",
        map(
            permutation((parse_function_decl, skip0, parse_block)),
            |(decl, _, body)| TopLevel::Function { decl, body },
        ),
    )(input)
}

pub(crate) fn parse_toplevel(input: Span) -> ParseResult<TopLevel> {
    context("toplevel", parse_function)(input)
}
