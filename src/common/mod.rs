pub mod binary;
pub mod target;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StructKind {
    Struct,
    Record,
}
