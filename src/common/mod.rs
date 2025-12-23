pub mod binary;
pub mod target;
pub mod typename;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StructKind {
    Struct,
    Record,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AllocMode {
    Stack,
}
