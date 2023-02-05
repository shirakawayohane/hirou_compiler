use super::error::{CompileErrorKind, ContextType};
use super::*;
use super::{error::CompileError, value::Value};
use crate::util::unbox_located_expression;
use crate::{ast::*, error_context};
use clap::error::Result;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

impl<'a> LLVMCodegenerator<'a> {
    fn eval_u8(&self, value_str: &str) -> Result<Value, CompileError> {
        let n = value_str.parse::<u8>().unwrap();
        let int_value = self.i8_type.const_int(n as u64, true);
        Ok(Value::U8Value(int_value))
    }
    fn eval_i32(&self, value_str: &str) -> Result<Value, CompileError> {
        let n = value_str.parse::<i32>().unwrap();
        let int_value = self.i32_type.const_int(n as u64, true);
        Ok(Value::I32Value(int_value))
    }
    fn eval_u32(&self, value_str: &str) -> Result<Value, CompileError> {
        let n = value_str.parse::<u32>().unwrap();
        let int_value = self.i32_type.const_int(n as u64, false);
        Ok(Value::U32Value(int_value))
    }
    fn eval_u64(&self, value_str: &str) -> Result<Value, CompileError> {
        let n = value_str.parse::<u64>().unwrap();
        let int_value = self.i64_type.const_int(n as u64, false);
        Ok(Value::U64Value(int_value))
    }
    fn eval_usize(&self, value_str: &str) -> Result<Value, CompileError> {
        match self.pointer_size {
            PointerSize::SixteenFour => {
                let n = value_str.parse::<u64>().unwrap();
                let int_value = self.i64_type.const_int(n as u64, false);
                Ok(Value::USizeValue(int_value))
            }
        }
    }
    fn eval_integer_literal(
        &self,
        value_str: &str,
        annotation: Option<&Type>,
    ) -> Result<Value, CompileError> {
        if let Some(annotation) = annotation {
            match annotation {
                Type::U8 => self.eval_u8(value_str),
                Type::U32 => self.eval_u32(value_str),
                Type::I32 => self.eval_i32(value_str),
                Type::U64 => self.eval_u64(value_str),
                Type::USize => self.eval_usize(value_str),
                Type::Ptr(_) => unreachable!(),
                Type::Void => unreachable!(),
            }
        } else {
            self.eval_i32(value_str)
        }
    }
    fn eval_variable_ref(
        &self,
        deref_count: u32,
        index_access: Option<Located<Expression>>,
        name: &str,
        _annotation: Option<&Type>,
    ) -> Result<Value, CompileError> {
        if let Some((ty, ptr)) = self
            .context
            .borrow()
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
        {
            let mut value = self.gen_load(&ty, *ptr)?;

            if let Some(index_expr) = index_access {
                if !ty.is_pointer_type() {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::CannotIndexAccess {
                            name: name.to_string(),
                            ty: ty.clone(),
                        },
                    ));
                }
                match &value {
                    Value::PointerValue(pointer_of, first_elelement_ptr) => {
                        let index_value =
                            self.eval_expression(index_expr.value, Some(&Type::USize))?;
                        let element_ptr = unsafe {
                            self.llvm_builder.build_gep(
                                *first_elelement_ptr,
                                &[index_value.unwrap_int_value()],
                                "index_access",
                            )
                        };
                        value = self.gen_load(pointer_of, element_ptr)?;
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

            for _ in 0..deref_count {
                match value {
                    Value::PointerValue(ty, ptr) => {
                        value = self.gen_load(&ty, ptr)?;
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

            Ok(value)
        } else {
            return Err(CompileError::from_error_kind(
                CompileErrorKind::VariableNotFound {
                    name: name.to_string(),
                },
            ));
        }
    }

    fn eval_binary_expr<'ctx>(
        &'ctx self,
        op: BinaryOp,
        lhs: Value<'ctx>,
        rhs: Value<'ctx>,
    ) -> Result<Value<'ctx>, CompileError>
    where
        'a: 'ctx,
    {
        pub fn get_cast_type_with_other_operand_of_bin_op<'a>(
            pointer_size: PointerSize,
            ty: &'a Type,
            other: &'a Type,
        ) -> Option<Type> {
            match ty {
                Type::U8 => match other {
                    Type::U8 => None,
                    Type::I32 => Some(Type::I32),
                    Type::U32 => Some(Type::U32),
                    Type::U64 => Some(Type::U64),
                    Type::USize => Some(Type::USize),
                    Type::Void => None,
                    Type::Ptr(_) => None,
                },
                Type::I32 => match other {
                    Type::U8 => None,
                    Type::I32 => None,
                    Type::USize => Some(Type::USize),
                    Type::U32 => Some(Type::U32),
                    Type::U64 => Some(Type::U64),
                    Type::Ptr(_) => None,
                    Type::Void => None,
                },
                Type::U32 => match other {
                    Type::I32 => None,
                    Type::U32 => None,
                    Type::U64 => Some(Type::U64),
                    Type::USize => match pointer_size {
                        PointerSize::SixteenFour => Some(Type::U64),
                    },
                    Type::U8 => None,
                    Type::Ptr(_) => None,
                    Type::Void => None,
                },
                Type::U64 => None,
                Type::USize => match other {
                    Type::U8 => None,
                    Type::I32 => match pointer_size {
                        PointerSize::SixteenFour => Some(Type::U64),
                    },
                    Type::U32 => match pointer_size {
                        PointerSize::SixteenFour => Some(Type::U64),
                    },
                    Type::U64 => None,
                    Type::USize => None,
                    Type::Ptr(_) => None,
                    Type::Void => None,
                },
                Type::Ptr(_) => None,
                Type::Void => None,
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
            match op {
                BinaryOp::Add => {
                    let result = self.llvm_builder.build_int_add(
                        lhs_integer_value.unwrap_int_value(),
                        rhs_integer_value.unwrap_int_value(),
                        "int+int",
                    );
                    Ok(match result_type {
                        Type::U8 => Value::U8Value(result),
                        Type::I32 => Value::I32Value(result),
                        Type::U32 => Value::U32Value(result),
                        Type::U64 => Value::U64Value(result),
                        Type::USize => Value::U64Value(result),
                        Type::Ptr(_) => unreachable!(),
                        Type::Void => unreachable!(),
                    })
                }
                BinaryOp::Sub => {
                    let result = self.llvm_builder.build_int_sub(
                        lhs_integer_value.unwrap_int_value(),
                        rhs_integer_value.unwrap_int_value(),
                        "int-int",
                    );
                    Ok(match result_type {
                        Type::U8 => Value::U8Value(result),
                        Type::I32 => Value::I32Value(result),
                        Type::U32 => Value::U32Value(result),
                        Type::U64 => Value::U64Value(result),
                        Type::USize => Value::U64Value(result),
                        Type::Ptr(_) => unreachable!(),
                        Type::Void => unreachable!(),
                    })
                }
                BinaryOp::Mul => {
                    let result = self.llvm_builder.build_int_mul(
                        lhs_integer_value.unwrap_int_value(),
                        rhs_integer_value.unwrap_int_value(),
                        "int*int",
                    );
                    Ok(match result_type {
                        Type::U8 => Value::U8Value(result),
                        Type::I32 => Value::I32Value(result),
                        Type::U32 => Value::U32Value(result),
                        Type::U64 => Value::U64Value(result),
                        Type::USize => Value::U64Value(result),
                        Type::Ptr(_) => unreachable!(),
                        Type::Void => unreachable!(),
                    })
                }
                BinaryOp::Div => {
                    let result = match result_type {
                        Type::I32 => self.llvm_builder.build_int_signed_div(
                            lhs_integer_value.unwrap_int_value(),
                            rhs_integer_value.unwrap_int_value(),
                            "int/int(signed)",
                        ),
                        Type::U8 | Type::U32 | Type::U64 | Type::USize => {
                            self.llvm_builder.build_int_unsigned_div(
                                lhs_integer_value.unwrap_int_value(),
                                rhs_integer_value.unwrap_int_value(),
                                "int/int(unsigned)",
                            )
                        }
                        Type::Ptr(_) => unreachable!(),
                        Type::Void => unreachable!(),
                    };
                    Ok(match result_type {
                        Type::U8 => Value::U8Value(result),
                        Type::I32 => Value::I32Value(result),
                        Type::U32 => Value::U32Value(result),
                        Type::U64 => Value::U64Value(result),
                        Type::USize => Value::U64Value(result),
                        Type::Ptr(_) => unreachable!(),
                        Type::Void => unreachable!(),
                    })
                }
            }
        } else {
            unimplemented!()
        }
    }

    fn eval_binary_exprs(
        &self,
        op: BinaryOp,
        mut args: Vec<Located<Expression>>,
    ) -> Result<Value, CompileError> {
        // let hoge = args.into_iter().fold(Value::Void, |val, rhs| {
        //     let rhs_value = match self.eval_expression(rhs.value, None) {
        //         Ok(_) => todo!(),
        //         Err(err) => return Err(err)
        //     }
        //     self.eval_binary_expr(op, value, rhs_value)?
        // });
        // todo!()
        let lhs = args.remove(0);
        let mut lhs_value = self.eval_expression(lhs.value, None)?;
        while !args.is_empty() {
            let rhs_value = self.eval_expression(args.remove(0).value, None)?;
            lhs_value = self.eval_binary_expr(op, lhs_value, rhs_value)?;
        }
        return Ok(lhs_value);
    }

    fn eval_call_expr(
        &self,
        name: &str,
        arg_exprs: Vec<Located<Expression>>,
        _annotation: Option<&Type>,
    ) -> Result<Value, CompileError> {
        let context = self.context.borrow();
        let result = context.find_function(&name);
        if let Some((return_type, arg_types, func)) = result {
            let mut evaluated_args: Vec<BasicMetadataValueEnum> = Vec::new();
            assert_eq!(arg_exprs.len(), arg_types.len());
            for (i, arg_expr) in arg_exprs.into_iter().enumerate() {
                let arg_type = arg_types.get(i).unwrap();
                let evaluated_arg = self.eval_expression(arg_expr.value, Some(&arg_type))?;
                evaluated_args.push(match evaluated_arg {
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
                });
            }
            Ok(
                match self
                    .llvm_builder
                    .build_call(*func, &evaluated_args, "function_call")
                    .try_as_basic_value()
                    .left()
                {
                    Some(returned_value) => match returned_value {
                        BasicValueEnum::ArrayValue(_) => todo!(),
                        BasicValueEnum::IntValue(int_value) => Value::I32Value(int_value),
                        BasicValueEnum::FloatValue(_) => todo!(),
                        BasicValueEnum::PointerValue(pointer_value) => {
                            let pointer_of = match return_type {
                                Type::Ptr(pointer_of) => pointer_of,
                                _ => unreachable!(),
                            };
                            Value::PointerValue(pointer_of.clone(), pointer_value)
                        }
                        BasicValueEnum::StructValue(_) => todo!(),
                        BasicValueEnum::VectorValue(_) => todo!(),
                    },
                    None => Value::Void,
                },
            )
        } else {
            if context.find_variable(&name).is_some() {
                Err(CompileError::from_error_kind(
                    CompileErrorKind::CallNotFunctionValue {
                        name: name.to_string(),
                    },
                ))
            } else {
                Err(CompileError::from_error_kind(
                    CompileErrorKind::FunctionNotFound {
                        name: name.to_string(),
                    },
                ))
            }
        }
    }
    pub(super) fn eval_expression(
        &self,
        expr: Expression,
        annotation: Option<&Type>,
    ) -> Result<Value, CompileError> {
        match expr {
            Expression::VariableRef {
                deref_count,
                index_access,
                name,
            } => {
                error_context!(ContextType::VariableRefExpression, {
                    self.eval_variable_ref(
                        deref_count,
                        index_access.map(unbox_located_expression),
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
                self.eval_binary_exprs(op, args)
            ),
            Expression::CallExpr { name, args } => error_context!(
                ContextType::CallExpression,
                self.eval_call_expr(&name, args, annotation)
            ),
        }
    }
}
