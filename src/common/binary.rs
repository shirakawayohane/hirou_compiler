use core::panic;

use crate::concrete_ast::ConcreteType;

use super::target::PointerSizedIntWidth;

pub(crate) fn get_cast_type(
    ptr_sized_int_type: PointerSizedIntWidth,
    lhs: &ConcreteType,
    rhs: &ConcreteType,
) -> (Option<ConcreteType>, Option<ConcreteType>) {
    match lhs {
        ConcreteType::I32 => match rhs {
            ConcreteType::I32 => (None, None),
            ConcreteType::I64 => (Some(ConcreteType::I64), None),
            ConcreteType::U32 => (None, Some(ConcreteType::I32)),
            ConcreteType::U64 => (Some(ConcreteType::I64), Some(ConcreteType::I64)),
            ConcreteType::U8 => (None, Some(ConcreteType::I32)),
            _ => panic!("Invalid type for binary expression"),
        },
        ConcreteType::I64 => match rhs {
            ConcreteType::I32 => (None, Some(ConcreteType::I64)),
            ConcreteType::I64 => (None, None),
            ConcreteType::U32 => (None, Some(ConcreteType::I64)),
            ConcreteType::U64 => (None, Some(ConcreteType::I64)),
            ConcreteType::U8 => (None, Some(ConcreteType::I64)),
            _ => panic!("Invalid type for binary expression"),
        },
        ConcreteType::U32 => match rhs {
            ConcreteType::I32 => (Some(ConcreteType::I32), None),
            ConcreteType::I64 => (Some(ConcreteType::I64), None),
            ConcreteType::U32 => (None, None),
            ConcreteType::U64 => (Some(ConcreteType::U64), None),
            ConcreteType::U8 => (Some(ConcreteType::U32), None),
            _ => panic!("Invalid type for binary expression"),
        },
        ConcreteType::U64 => match rhs {
            ConcreteType::I32 => (Some(ConcreteType::I64), None),
            ConcreteType::I64 => (Some(ConcreteType::I64), None),
            ConcreteType::U32 => (None, Some(ConcreteType::U64)),
            ConcreteType::U64 => (None, None),
            _ => panic!("Invalid type for binary expression"),
        },
        ConcreteType::U8 => match rhs {
            ConcreteType::I32 => (Some(ConcreteType::I32), None),
            ConcreteType::I64 => (Some(ConcreteType::I64), None),
            ConcreteType::U32 => (Some(ConcreteType::U32), None),
            ConcreteType::U64 => (Some(ConcreteType::U64), None),
            ConcreteType::U8 => (None, None),
            _ => panic!("Invalid type for binary expression"),
        },
        ConcreteType::Bool => match rhs {
            ConcreteType::Bool => (None, None),
            _ => panic!("Invalid type for binary expression"),
        },
        ConcreteType::Ptr(_) => panic!("Invalid type for binary expression"),
        ConcreteType::Void => panic!("Invalid type for binary expression"),
        ConcreteType::StructLike(_) => panic!("Invalid type for binary expression"),
    }
}
