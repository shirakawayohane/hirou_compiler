use crate::ast::{Expression, Located};

pub fn unbox_located_expression(expr: Located<Box<Expression>>) -> Located<Expression> {
    let range = expr.range;
    Located {
        range,
        value: *expr.value,
    }
}

pub fn box_located_expression(expr: Located<Expression>) -> Located<Box<Expression>> {
    let range = expr.range;
    Located {
        range,
        value: Box::new(expr.value),
    }
}
