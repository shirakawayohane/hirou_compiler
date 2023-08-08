use super::error::{CompileErrorKind, ContextType};
use super::*;
use super::{error::CompileError, result_value::Value};
use crate::util::unbox_located_expression;
use crate::{ast::*, error_context};
use clap::error::Result;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

impl<'a> LLVMCodegenerator<'a> {
    fn eval_u8(&self, value_str: &str) -> Result<(ResolvedType, Value), CompileError> {
        let n = value_str.parse::<u8>().unwrap();
        let int_value = self.i8_type.const_int(n as u64, true);
        Ok((ResolvedType::U8, Value::U8Value(int_value)))
    }
    fn eval_i32(&self, value_str: &str) -> Result<(ResolvedType, Value), CompileError> {
        let n = value_str.parse::<i32>().unwrap();
        let int_value = self.i32_type.const_int(n as u64, true);
        Ok((ResolvedType::I32, Value::I32Value(int_value)))
    }
    fn eval_u32(&self, value_str: &str) -> Result<(ResolvedType, Value), CompileError> {
        let n = value_str.parse::<u32>().unwrap();
        let int_value = self.i32_type.const_int(n as u64, false);
        Ok((ResolvedType::U32, Value::U32Value(int_value)))
    }
    fn eval_u64(&self, value_str: &str) -> Result<(ResolvedType, Value), CompileError> {
        let n = value_str.parse::<u64>().unwrap();
        let int_value = self.i64_type.const_int(n as u64, false);
        Ok((ResolvedType::U64, Value::U64Value(int_value)))
    }
    fn eval_usize(&self, value_str: &str) -> Result<(ResolvedType, Value), CompileError> {
        match self.pointer_size {
            PointerSize::SixteenFour => {
                let n = value_str.parse::<u64>().unwrap();
                let int_value = self.i64_type.const_int(n as u64, false);
                Ok((ResolvedType::USize, Value::USizeValue(int_value)))
            }
        }
    }

    fn eval_integer_literal(
        &self,
        value_str: &str,
        annotation: Option<&ResolvedType>,
    ) -> Result<(ResolvedType, Value), CompileError> {
        if let Some(annotation) = annotation {
            match annotation {
                ResolvedType::U8 => self.eval_u8(value_str),
                ResolvedType::U32 => self.eval_u32(value_str),
                ResolvedType::I32 => self.eval_i32(value_str),
                ResolvedType::U64 => self.eval_u64(value_str),
                ResolvedType::USize => self.eval_usize(value_str),
                ResolvedType::Ptr(_) => unreachable!(),
                ResolvedType::Void => unreachable!(),
            }
        } else {
            self.eval_i32(value_str)
        }
    }
    fn eval_variable_ref(
        &self,
        deref_count: u32,
        index_access: Option<Located<Box<Expression>>>,
        name: &str,
        _annotation: Option<&ResolvedType>,
    ) -> Result<(ResolvedType, Value), CompileError> {
        let (ty, ptr) = self.find_variable(name)?;
        let mut result_ty = self.resolve_type(&ty)?;
        let mut result_value = self.gen_load(&result_ty, ptr)?;

        if let Some(index_expr) = index_access {
            if !result_ty.is_pointer_type() {
                return Err(CompileError::from_error_kind(
                    CompileErrorKind::CannotIndexAccess {
                        name: name.to_string(),
                        ty: self.resolve_type(&ty)?.clone(),
                    },
                ));
            }
            match result_value {
                Value::PointerValue(pointer_of, first_elelement_ptr) => {
                    let (_, index_value) =
                        self.eval_expression(&index_expr.value, Some(&ResolvedType::USize))?;
                    let element_ptr = unsafe {
                        self.llvm_builder.build_gep(
                            first_elelement_ptr,
                            &[index_value.unwrap_int_value()],
                            "index_access",
                        )
                    };
                    result_value = self.gen_load(&pointer_of, element_ptr)?;
                    result_ty = *pointer_of;
                }
                _ => unreachable!(),
            }
        }

        for _ in 0..deref_count {
            match result_value {
                Value::PointerValue(ty, ptr) => {
                    result_value = self.gen_load(&ty, ptr)?;
                }
                _ => {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::CannotDeref {
                            name: name.to_string(),
                            deref_count,
                        },
                    ))
                }
            }
        }

        Ok((result_ty, result_value))
    }

    fn eval_binary_expr<'ctx>(
        &'ctx self,
        op: BinaryOp,
        lhs: Value<'ctx>,
        rhs: Value<'ctx>,
    ) -> Result<(ResolvedType, Value<'ctx>), CompileError>
    where
        'a: 'ctx,
    {
        pub fn get_cast_type_with_other_operand_of_bin_op<'a>(
            pointer_size: PointerSize,
            ty: &'a ResolvedType,
            other: &'a ResolvedType,
        ) -> Option<ResolvedType> {
            match ty {
                ResolvedType::U8 => match other {
                    ResolvedType::U8 => None,
                    ResolvedType::I32 => Some(ResolvedType::I32),
                    ResolvedType::U32 => Some(ResolvedType::U32),
                    ResolvedType::U64 => Some(ResolvedType::U64),
                    ResolvedType::USize => Some(ResolvedType::USize),
                    ResolvedType::Void => None,
                    ResolvedType::Ptr(_) => None,
                },
                ResolvedType::I32 => match other {
                    ResolvedType::U8 => None,
                    ResolvedType::I32 => None,
                    ResolvedType::USize => Some(ResolvedType::USize),
                    ResolvedType::U32 => Some(ResolvedType::U32),
                    ResolvedType::U64 => Some(ResolvedType::U64),
                    ResolvedType::Ptr(_) => None,
                    ResolvedType::Void => None,
                },
                ResolvedType::U32 => match other {
                    ResolvedType::I32 => None,
                    ResolvedType::U32 => None,
                    ResolvedType::U64 => Some(ResolvedType::U64),
                    ResolvedType::USize => match pointer_size {
                        PointerSize::SixteenFour => Some(ResolvedType::U64),
                    },
                    ResolvedType::U8 => None,
                    ResolvedType::Ptr(_) => None,
                    ResolvedType::Void => None,
                },
                ResolvedType::U64 => None,
                ResolvedType::USize => match other {
                    ResolvedType::U8 => None,
                    ResolvedType::I32 => match pointer_size {
                        PointerSize::SixteenFour => Some(ResolvedType::U64),
                    },
                    ResolvedType::U32 => match pointer_size {
                        PointerSize::SixteenFour => Some(ResolvedType::U64),
                    },
                    ResolvedType::U64 => None,
                    ResolvedType::USize => None,
                    ResolvedType::Ptr(_) => None,
                    ResolvedType::Void => None,
                },
                ResolvedType::Ptr(_) => None,
                ResolvedType::Void => None,
            }
        }

        let lhs_type = lhs.get_type();
        let rhs_type = rhs.get_type();

        if !lhs_type.is_valid_as_operand() {
            return Err(CompileError::from_error_kind(
                CompileErrorKind::InvalidOperand(Box::new(lhs_type)),
            ));
        }
        if !rhs_type.is_valid_as_operand() {
            return Err(CompileError::from_error_kind(
                CompileErrorKind::InvalidOperand(Box::new(rhs_type)),
            ));
        }

        if lhs_type.is_integer_type() && rhs_type.is_integer_type() {
            let lhs_cast_type =
                get_cast_type_with_other_operand_of_bin_op(self.pointer_size, &lhs_type, &rhs_type);
            let rhs_cast_type =
                get_cast_type_with_other_operand_of_bin_op(self.pointer_size, &rhs_type, &lhs_type);
            let (lhs_integer_value, lhs_type) = if let Some(cast_type) = lhs_cast_type {
                (self.gen_try_cast(lhs, &cast_type)?, Some(cast_type))
            } else {
                (lhs, None)
            };
            let (rhs_integer_value, rhs_type) = if let Some(cast_type) = rhs_cast_type {
                (self.gen_try_cast(rhs, &cast_type)?, Some(cast_type))
            } else {
                (rhs, Some(rhs_type))
            };
            let result_type = lhs_type.unwrap_or(rhs_type.unwrap());
            let value = match op {
                BinaryOp::Add => {
                    let result = self.llvm_builder.build_int_add(
                        lhs_integer_value.unwrap_int_value(),
                        rhs_integer_value.unwrap_int_value(),
                        "int+int",
                    );
                    match result_type {
                        ResolvedType::U8 => Value::U8Value(result),
                        ResolvedType::I32 => Value::I32Value(result),
                        ResolvedType::U32 => Value::U32Value(result),
                        ResolvedType::U64 => Value::U64Value(result),
                        ResolvedType::USize => Value::U64Value(result),
                        ResolvedType::Ptr(_) => unreachable!(),
                        ResolvedType::Void => unreachable!(),
                    }
                }
                BinaryOp::Sub => {
                    let result = self.llvm_builder.build_int_sub(
                        lhs_integer_value.unwrap_int_value(),
                        rhs_integer_value.unwrap_int_value(),
                        "int-int",
                    );
                    match result_type {
                        ResolvedType::U8 => Value::U8Value(result),
                        ResolvedType::I32 => Value::I32Value(result),
                        ResolvedType::U32 => Value::U32Value(result),
                        ResolvedType::U64 => Value::U64Value(result),
                        ResolvedType::USize => Value::U64Value(result),
                        ResolvedType::Ptr(_) => unreachable!(),
                        ResolvedType::Void => unreachable!(),
                    }
                }
                BinaryOp::Mul => {
                    let result = self.llvm_builder.build_int_mul(
                        lhs_integer_value.unwrap_int_value(),
                        rhs_integer_value.unwrap_int_value(),
                        "int*int",
                    );
                    match result_type {
                        ResolvedType::U8 => Value::U8Value(result),
                        ResolvedType::I32 => Value::I32Value(result),
                        ResolvedType::U32 => Value::U32Value(result),
                        ResolvedType::U64 => Value::U64Value(result),
                        ResolvedType::USize => Value::U64Value(result),
                        ResolvedType::Ptr(_) => unreachable!(),
                        ResolvedType::Void => unreachable!(),
                    }
                }
                BinaryOp::Div => {
                    let result = match result_type {
                        ResolvedType::I32 => self.llvm_builder.build_int_signed_div(
                            lhs_integer_value.unwrap_int_value(),
                            rhs_integer_value.unwrap_int_value(),
                            "int/int(signed)",
                        ),
                        ResolvedType::U8
                        | ResolvedType::U32
                        | ResolvedType::U64
                        | ResolvedType::USize => self.llvm_builder.build_int_unsigned_div(
                            lhs_integer_value.unwrap_int_value(),
                            rhs_integer_value.unwrap_int_value(),
                            "int/int(unsigned)",
                        ),
                        ResolvedType::Ptr(_) => unreachable!(),
                        ResolvedType::Void => unreachable!(),
                    };
                    match result_type {
                        ResolvedType::U8 => Value::U8Value(result),
                        ResolvedType::I32 => Value::I32Value(result),
                        ResolvedType::U32 => Value::U32Value(result),
                        ResolvedType::U64 => Value::U64Value(result),
                        ResolvedType::USize => Value::U64Value(result),
                        ResolvedType::Ptr(_) => unreachable!(),
                        ResolvedType::Void => unreachable!(),
                    }
                }
            };

            match &value {
                Value::U8Value(_) => Ok((ResolvedType::U8, value)),
                Value::I32Value(_) => Ok((ResolvedType::I32, value)),
                Value::U64Value(_) => Ok((ResolvedType::U64, value)),
                Value::U32Value(_) => Ok((ResolvedType::U32, value)),
                Value::USizeValue(_) => Ok((ResolvedType::USize, value)),
                Value::PointerValue(pointer_of, _) => Ok((ResolvedType::Ptr(pointer_of.clone()), value)),
                Value::Void => Ok((ResolvedType::Void, value)),
            }
        } else {
            unimplemented!()
        }
    }

    fn eval_binary_exprs(
        &self,
        op: BinaryOp,
        args: &[Located<Box<Expression>>],
    ) -> Result<(ResolvedType, Value), CompileError> {
        let mut lhs = &args[0];
        let mut rest = &args[1..];
        let (mut lhs_ty, mut lhs_value) = self.eval_expression(&lhs.value, None)?;
        while !args.is_empty() {
            (lhs, rest) = rest.split_first().unwrap();
            let (_rhs_ty, rhs_value) = self.eval_expression(&lhs.value, None)?;
            (lhs_ty, lhs_value) = self.eval_binary_expr(op, lhs_value, rhs_value)?;
        }
        return Ok((lhs_ty, lhs_value));
    }

    fn eval_call_expr(
        &self,
        name: &str,
        arg_exprs: &[Located<Box<Expression>>],
        _annotation: Option<&ResolvedType>,
    ) -> Result<(ResolvedType, Value), CompileError> {
        let mut evaluated_args: Vec<BasicMetadataValueEnum> = Vec::new();
        let mut evaluated_arg_tys: Vec<ResolvedType> = Vec::new();
        for (i, arg_expr) in arg_exprs.into_iter().enumerate() {
            let (ty, evaluated_arg) = self.eval_expression(&arg_expr.value, None)?;
            let value_enum = match evaluated_arg {
                Value::U8Value(v) => BasicMetadataValueEnum::IntValue(v),
                Value::I32Value(v) => BasicMetadataValueEnum::IntValue(v),
                Value::U32Value(v) => BasicMetadataValueEnum::IntValue(v),
                Value::U64Value(v) => BasicMetadataValueEnum::IntValue(v),
                Value::USizeValue(v) => BasicMetadataValueEnum::IntValue(v),
                Value::PointerValue(_, pointer_value) => {
                    BasicMetadataValueEnum::PointerValue(pointer_value)
                }
                Value::Void => {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::InvalidArgument,
                    ))
                }
            };
            evaluated_args.push(value_enum);
            evaluated_arg_tys.push(ty);
        }
        let func = self.find_function_impl(name, &evaluated_arg_tys)?;
        Ok(
            match self
                .llvm_builder
                .build_call(func.function_value, &evaluated_args, "function_call")
                .try_as_basic_value()
                .left()
            {
                Some(returned_value) => match returned_value {
                    BasicValueEnum::ArrayValue(_) => todo!(),
                    BasicValueEnum::IntValue(int_value) => {
                        (ResolvedType::I32, Value::I32Value(int_value))
                    }
                    BasicValueEnum::FloatValue(_) => todo!(),
                    BasicValueEnum::PointerValue(pointer_value) => {
                        let pointer_of = match &func.return_type {
                            ResolvedType::Ptr(pointer_of) => pointer_of,
                            _ => unreachable!(),
                        };
                        (
                            ResolvedType::Ptr(pointer_of.clone()),
                            Value::PointerValue(pointer_of.clone(), pointer_value),
                        )
                    }
                    BasicValueEnum::StructValue(_) => todo!(),
                    BasicValueEnum::VectorValue(_) => todo!(),
                },
                None => (ResolvedType::Void, Value::Void),
            },
        )
    }
    pub(super) fn eval_expression(
        &self,
        expr: &Expression,
        annotation: Option<&ResolvedType>,
    ) -> Result<(ResolvedType, Value), CompileError> {
        match expr {
            Expression::VariableRef {
                deref_count,
                index_access,
                name,
            } => {
                error_context!(ContextType::VariableRefExpression, {
                    self.eval_variable_ref(
                        *deref_count,
                        index_access.clone(),
                        &name,
                        annotation,
                    )
                })
            }
            Expression::NumberLiteral { value } => error_context!(
                ContextType::NumberLiteralExpression,
                self.eval_integer_literal(&value, annotation)
            ),
            Expression::BinaryExpr { op, args } => error_context!(
                ContextType::BinaryExpression,
                self.eval_binary_exprs(*op, args)
            ),
            Expression::CallExpr { name, args } => error_context!(
                ContextType::CallExpression,
                self.eval_call_expr(&name, args, annotation)
            ),
        }
    }
}
