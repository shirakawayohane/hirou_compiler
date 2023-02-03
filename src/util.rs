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
