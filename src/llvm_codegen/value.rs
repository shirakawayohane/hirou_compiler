use inkwell::values::IntValue;

#[derive(Debug, Clone, Copy)]
pub(super) enum Value<'a> {
    IntValue(IntValue<'a>),
    Void,
}
