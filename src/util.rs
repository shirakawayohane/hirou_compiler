use crate::ast::{Expression, Located};

pub fn unbox_located_expression<'a>(
    expr: Located<'a, Box<Expression<'a>>>,
) -> Located<Expression<'a>> {
    let range = expr.range;
    Located {
        range,
        value: *expr.value,
    }
}

pub fn box_located_expression<'a>(
    expr: Located<'a, Expression<'a>>,
) -> Located<Box<Expression<'a>>> {
    let range = expr.range;
    Located {
        range,
        value: Box::new(expr.value),
    }
}
