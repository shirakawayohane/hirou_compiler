use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, none_of},
    combinator::{cut, opt},
    error::context,
    multi::many0,
    sequence::{pair, preceded, terminated, tuple},
};

use crate::ast::*;

use super::{
    token::*,
    ty::{parse_generic_arguments, parse_type},
    util::*,
    *,
};

fn parse_number_literal(input: Span) -> NotLocatedParseResult<Expression> {
    map(digit1, |str: Span| {
        Expression::NumberLiteral(NumberLiteralExpr {
            value: str.to_string(),
        })
    })(input)
}

fn parse_variable_ref(input: Span) -> NotLocatedParseResult<Expression> {
    map(parse_identifier, |name| {
        Expression::VariableRef(VariableRefExpr { name })
    })(input)
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

pub(super) fn parse_intrinsic_binop_expression(input: Span) -> NotLocatedParseResult<Expression> {
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
                        map(eq_token, |_| BinaryOp::Equals),
                        map(neq_token, |_| BinaryOp::NotEquals),
                        map(lte_token, |_| BinaryOp::LessThanOrEquals),
                        map(lt_token, |_| BinaryOp::LessThan),
                        map(gte_token, |_| BinaryOp::GreaterThanOrEquals),
                        map(gt_token, |_| BinaryOp::GreaterThan),
                    )),
                    parse_boxed_expression,
                    parse_boxed_expression,
                )),
                skip0,
            ),
            rparen,
        ),
        |(binop, lhs, rhs)| {
            Expression::Binary(BinaryExpr {
                op: binop,
                lhs,
                rhs,
            })
        },
    )(input)
}
pub(super) fn parse_intrinsic_unary_op_expression(
    input: Span,
) -> NotLocatedParseResult<Expression> {
    map(
        preceded(
            lparen,
            tuple((
                alt((map(not_token, |_| {
                    // panic!();
                    UnaryOp::Not
                }),)),
                cut(terminated(parse_boxed_expression, rparen)),
            )),
        ),
        |(op, target)| {
            Expression::Unary(UnaryExpr {
                op,
                operand: target,
            })
        },
    )(input)
}

#[test]
fn parse_test_intrinsic_unary_op_expression() {
    assert!(parse_boxed_expression("(not a b)".into()).is_err());
    assert!(parse_boxed_expression("(not a)".into()).is_ok());
}

pub(super) fn parse_intrinsic_multi_op_expression(
    input: Span,
) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            lparen,
            pair(
                alt((
                    map(and_token, |_| MultiOp::And),
                    map(or_token, |_| MultiOp::Or),
                )),
                cut(many1(parse_boxed_expression)),
            ),
            rparen,
        ),
        |(op, operands)| Expression::Multi(MultiExpr { op, operands }),
    )(input)
}
pub(super) fn parse_function_call_expression(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            lparen,
            tuple((
                parse_identifier,
                opt(parse_generic_arguments),
                parse_arguments,
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

#[test]
fn test_parse_function_call_expression() {
    // write test
    let result = parse_function_call_expression(Span::new("(write \"%d\", x)"));
    assert!(result.is_ok());
    let (rest, expr) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    match expr {
        Expression::Call(call_expr) => {
            assert_eq!(call_expr.name, "write");
            assert!(call_expr.generic_args.is_none());
            assert_eq!(call_expr.args.len(), 2);
            assert_eq!(
                *call_expr.args[0].value,
                Expression::StringLiteral(StringLiteralExpr {
                    value: "%d".to_string()
                })
            );
            assert_eq!(
                *call_expr.args[1].value,
                Expression::VariableRef(VariableRefExpr {
                    name: "x".to_string()
                })
            );
        }
        _ => panic!("unexpected expression type"),
    }
}

fn parse_if_expression(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            lparen,
            tuple((
                if_token,
                parse_boxed_expression,
                parse_boxed_expression,
                parse_boxed_expression,
            )),
            rparen,
        ),
        |(_, cond, then, els)| Expression::If(IfExpr { cond, then, els }),
    )(input)
}

fn parse_when_expression(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            lparen,
            tuple((when_token, parse_boxed_expression, parse_boxed_expression)),
            rparen,
        ),
        |(_, cond, then)| Expression::When(WhenExpr { cond, then }),
    )(input)
}

#[test]
fn test_parse_if_expression() {
    let result = parse_if_expression(Span::new("(if a b c)"));
    assert!(result.is_ok());
    let (rest, expr) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    if let Expression::If(if_expr) = expr {
        assert_eq!(
            *if_expr.cond.value,
            Expression::VariableRef(VariableRefExpr {
                name: "a".to_string()
            })
        );
        assert_eq!(
            *if_expr.then.value,
            Expression::VariableRef(VariableRefExpr {
                name: "b".to_string()
            })
        );
        assert_eq!(
            *if_expr.els.value,
            Expression::VariableRef(VariableRefExpr {
                name: "c".to_string()
            })
        );
    } else {
        panic!();
    }
}

fn parse_deref_expression(input: Span) -> NotLocatedParseResult<Expression> {
    map(preceded(asterisk, parse_boxed_expression), |expr| {
        Expression::DerefExpr(DerefExpr { target: expr })
    })(input)
}

fn parse_string_literal(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            skip0,
            delimited(
                doublequote,
                map(
                    many0(alt((
                        map(tag("\\\""), |_| "\"".to_string()),
                        map(tag("\\r"), |_| "\r".to_string()),
                        map(tag("\\n"), |_| "\n".to_string()),
                        map(tag("\\t"), |_| "\t".to_string()),
                        map(none_of("\""), |c| c.to_string()),
                    ))),
                    |chars| chars.join(""),
                ),
                doublequote,
            ),
            skip0,
        ),
        |value| Expression::StringLiteral(StringLiteralExpr { value }),
    )(input)
}

#[test]
fn test_parse_string_literal() {
    let result = parse_string_literal(Span::new("\"%d\""));
    assert!(result.is_ok());
    let (rest, expr) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    assert_eq!(
        expr,
        Expression::StringLiteral(StringLiteralExpr {
            value: "%d".to_string()
        })
    );
}

fn parse_field(input: Span) -> NotLocatedParseResult<(String, LocatedExpr)> {
    context(
        "field",
        map(
            tuple((parse_identifier, colon, cut(parse_boxed_expression))),
            |(name, _, expr)| (name, expr),
        ),
    )(input)
}

fn parse_bool_literal(input: Span) -> NotLocatedParseResult<Expression> {
    map(alt((tag("true"), tag("false"))), |str: Span| {
        Expression::BoolLiteral(BoolLiteralExpr {
            value: str.fragment() == &"true",
        })
    })(input)
}

fn parse_struct_literal(input: Span) -> NotLocatedParseResult<Expression> {
    fn parse_fields(input: Span) -> NotLocatedParseResult<Vec<(String, LocatedExpr)>> {
        let mut fields = Vec::new();
        let mut rest = input;
        loop {
            (rest, _) = skip0(rest)?;
            if rest.starts_with('}') {
                break;
            }
            let field;
            (rest, field) = parse_field(rest)?;
            fields.push(field);
        }
        (rest, _) = opt(comma)(rest)?;
        Ok((rest, fields))
    }
    map(
        tuple((
            parse_identifier,
            opt(parse_generic_arguments),
            delimited(lbracket, cut(parse_fields), rbracket),
        )),
        |(name, generic_args, fields)| {
            Expression::StructLiteral(StructLiteralExpr {
                name,
                generic_args,
                fields,
            })
        },
    )(input)
}

#[test]
fn test_parse_struct_literal() {
    let result = parse_boxed_expression(Span::new("Point { x: 1, y: 2, }"));
    assert!(result.is_ok());
    let (rest, expr) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    if let Expression::StructLiteral(struct_literal) = *expr.value {
        assert_eq!(struct_literal.name, "Point");
        assert!(struct_literal.generic_args.is_none());
        assert_eq!(struct_literal.fields.len(), 2);
        assert_eq!(struct_literal.fields[0].0, "x");
        assert_eq!(
            *struct_literal.fields[0].1.value,
            Expression::NumberLiteral(NumberLiteralExpr {
                value: "1".to_string()
            })
        );
        assert_eq!(struct_literal.fields[1].0, "y");
        assert_eq!(
            *struct_literal.fields[1].1.value,
            Expression::NumberLiteral(NumberLiteralExpr {
                value: "2".to_string()
            })
        );
    } else {
        panic!();
    }

    let result = parse_struct_literal(Span::new("Vec<T> { buf: a, size: 4 }"));
    assert!(result.is_ok());
    let (rest, expr) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), "");
    if let Expression::StructLiteral(struct_literal) = expr {
        assert_eq!(&struct_literal.name, "Vec");
        let generic_args = struct_literal.generic_args.unwrap();
        assert_eq!(generic_args.len(), 1);
        if let UnresolvedType::TypeRef(typeref) = &generic_args[0].value {
            assert_eq!(typeref.name, "T");
        } else {
            panic!();
        }
        assert_eq!(struct_literal.fields.len(), 2);
        assert_eq!(struct_literal.fields[0].0, "buf");
        assert_eq!(struct_literal.fields[1].0, "size");
    } else {
        panic!();
    }
}

fn parse_sizeof(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(lparen, preceded(sizeof_token, cut(parse_type)), rparen),
        |ty| Expression::SizeOf(SizeOfExpr { ty }),
    )(input)
}

fn parse_asignment(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            lparen,
            tuple((
                assign_token,
                many0(asterisk),
                parse_identifier,
                opt(index_access),
                parse_boxed_expression,
            )),
            rparen,
        ),
        |(_, asterisks, name, index_access, value_expr)| {
            Expression::Assignment(AssignExpr {
                deref_count: asterisks.len() as u32,
                index_access,
                name,
                value: value_expr,
            })
        },
    )(input)
}

#[test]
fn test_parse_assignment() {
    assert!(parse_asignment(Span::new("(:=< a 1)")).is_ok());
    assert!(parse_asignment(Span::new("(:=< buf[index] value)")).is_ok());
}

fn parse_variable_decl(input: Span) -> NotLocatedParseResult<Expression> {
    delimited(
        lparen,
        preceded(
            var_decl_token,
            cut(map(
                many1(located(map(
                    tuple((
                        parse_identifier,
                        opt(context(
                            "type_annotation",
                            map(
                                tuple((skip0, colon, skip0, cut(parse_type))),
                                |(_, _, _, ty)| ty,
                            ),
                        )),
                        preceded(skip0, parse_boxed_expression),
                    )),
                    |(name, ty, expression)| VariableDecl {
                        ty,
                        name,
                        value: expression,
                    },
                ))),
                |decls| Expression::VariableDecl(VariableDeclsExpr { decls }),
            )),
        ),
        rparen,
    )(input)
}

pub(super) fn parse_boxed_expression(input: Span) -> ParseResult<Box<Expression>> {
    let (rest, expr) = located(map(
        alt((
            context("sizeof", parse_sizeof),
            context("deref", parse_deref_expression),
            context("string_literal", parse_string_literal),
            context("number_literal", parse_number_literal),
            context("bool_literal", parse_bool_literal),
            context("struct_literal", parse_struct_literal),
            context("variable_ref", parse_variable_ref),
            context("if", parse_if_expression),
            context("when", parse_when_expression),
            context("assignment", parse_asignment),
            context("variable_decl", parse_variable_decl),
            context("unary_op", parse_intrinsic_unary_op_expression),
            context("binop", parse_intrinsic_binop_expression),
            context("multi_op", parse_intrinsic_multi_op_expression),
            context("call", parse_function_call_expression),
        )),
        Box::new,
    ))(input)?;

    {
        let (rest, opt_index_expr) = opt(located(index_access))(rest)?;
        if let Some(index_expr) = opt_index_expr {
            return Ok((
                rest,
                Located {
                    range: index_expr.range,
                    value: Box::new(Expression::IndexAccess(IndexAccessExpr {
                        target: expr,
                        index: index_expr.value,
                    })),
                },
            ));
        }
    }
    {
        let (rest, opt_field_access) = opt(located(field_access))(rest)?;
        if let Some(field_access) = opt_field_access {
            return Ok((
                rest,
                Located {
                    range: field_access.range,
                    value: Box::new(Expression::FieldAccess(FieldAccessExpr {
                        target: expr,
                        field_name: field_access.value,
                    })),
                },
            ));
        }
    }

    Ok((rest, expr))
}
