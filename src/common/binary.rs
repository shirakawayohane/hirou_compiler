use crate::resolved_ast::ResolvedType;

use super::target::PointerSizedIntWidth;

pub(crate) fn get_cast_type(
    ptr_sized_int_type: PointerSizedIntWidth,
    lhs: &ResolvedType,
    rhs: &ResolvedType,
) -> (Option<ResolvedType>, Option<ResolvedType>) {
    match lhs {
        ResolvedType::I32 => match rhs {
            ResolvedType::I32 => (None, None),
            ResolvedType::I64 => (Some(ResolvedType::I64), None),
            ResolvedType::U32 => (None, Some(ResolvedType::I32)),
            ResolvedType::U64 => (Some(ResolvedType::I64), Some(ResolvedType::I64)),
            ResolvedType::USize => (Some(ResolvedType::I64), Some(ResolvedType::I64)),
            ResolvedType::U8 => (None, Some(ResolvedType::I32)),
            _ => panic!("Invalid type for binary expression"),
        },
        ResolvedType::I64 => match rhs {
            ResolvedType::I32 => (None, Some(ResolvedType::I64)),
            ResolvedType::I64 => (None, None),
            ResolvedType::U32 => (None, Some(ResolvedType::I64)),
            ResolvedType::U64 => (None, Some(ResolvedType::I64)),
            ResolvedType::USize => (None, Some(ResolvedType::I64)),
            ResolvedType::U8 => (None, Some(ResolvedType::I64)),
            _ => panic!("Invalid type for binary expression"),
        },
        ResolvedType::U32 => match rhs {
            ResolvedType::I32 => (Some(ResolvedType::I32), None),
            ResolvedType::I64 => (Some(ResolvedType::I64), None),
            ResolvedType::U32 => (None, None),
            ResolvedType::U64 => (Some(ResolvedType::U64), None),
            ResolvedType::USize => {
                if matches!(ptr_sized_int_type, PointerSizedIntWidth::ThirtyTwo) {
                    (None, None)
                } else {
                    (Some(ResolvedType::USize), None)
                }
            }
            ResolvedType::U8 => (Some(ResolvedType::U32), None),
            _ => panic!("Invalid type for binary expression"),
        },
        ResolvedType::U64 => match rhs {
            ResolvedType::I32 => (Some(ResolvedType::I64), None),
            ResolvedType::I64 => (Some(ResolvedType::I64), None),
            ResolvedType::U32 => (None, Some(ResolvedType::U64)),
            ResolvedType::U64 => (None, None),
            ResolvedType::USize => {
                if matches!(ptr_sized_int_type, PointerSizedIntWidth::ThirtyTwo) {
                    (None, Some(ResolvedType::U64))
                } else {
                    (None, None)
                }
            }
            _ => panic!("Invalid type for binary expression"),
        },
        ResolvedType::USize => match rhs {
            ResolvedType::I32 => (Some(ResolvedType::I32), None),
            ResolvedType::I64 => (Some(ResolvedType::I64), None),
            ResolvedType::U32 => {
                if matches!(ptr_sized_int_type, PointerSizedIntWidth::ThirtyTwo) {
                    (None, None)
                } else {
                    (Some(ResolvedType::U64), None)
                }
            }
            ResolvedType::U64 => {
                if matches!(ptr_sized_int_type, PointerSizedIntWidth::ThirtyTwo) {
                    (Some(ResolvedType::U64), None)
                } else {
                    (None, None)
                }
            }
            ResolvedType::USize => (None, None),
            ResolvedType::U8 => {
                if matches!(ptr_sized_int_type, PointerSizedIntWidth::ThirtyTwo) {
                    (Some(ResolvedType::U32), None)
                } else {
                    (Some(ResolvedType::U64), None)
                }
            }
            _ => panic!("Invalid type for binary expression"),
        },
        ResolvedType::U8 => match rhs {
            ResolvedType::I32 => (Some(ResolvedType::I32), None),
            ResolvedType::I64 => (Some(ResolvedType::I64), None),
            ResolvedType::U32 => (Some(ResolvedType::U32), None),
            ResolvedType::U64 => (Some(ResolvedType::U64), None),
            ResolvedType::USize => {
                if matches!(ptr_sized_int_type, PointerSizedIntWidth::ThirtyTwo) {
                    (Some(ResolvedType::U32), None)
                } else {
                    (Some(ResolvedType::U64), None)
                }
            }
            ResolvedType::U8 => (None, None),
            _ => panic!("Invalid type for binary expression"),
        },
        ResolvedType::Bool => match rhs {
            ResolvedType::Bool => (None, None),
            _ => panic!("Invalid type for binary expression"),
        },
        ResolvedType::Ptr(_) => panic!("Invalid type for binary expression"),
        ResolvedType::Void => panic!("Invalid type for binary expression"),
        ResolvedType::Unknown => panic!("Invalid type for binary expression"),
        ResolvedType::StructLike(_) => panic!("Invalid type for binary expression"),
    }
}
