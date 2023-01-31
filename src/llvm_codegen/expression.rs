use super::value::Value;
use super::*;
use crate::ast::*;

use clap::error::Result;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

impl LLVMCodegenerator<'_> {
    fn eval_u8(&self, value_str: &str) -> Result<Value, CompileError> {
        let n = value_str.parse::<u8>().unwrap();
        let int_value = self.i8_type.const_int(n as u64, true);
        Ok(Value::I32Value(int_value))
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
            PointerSize::SixteenFour => self.eval_u64(value_str),
        }
    }
    fn eval_integer_literal(
        &self,
        value_str: &str,
        annotation: Option<Type>,
    ) -> Result<Value, CompileError> {
        if let Some(annotation) = annotation {
            match annotation {
                Type::U8 => self.eval_u8(value_str),
                Type::U32 => self.eval_u32(value_str),
                Type::I32 => self.eval_i32(value_str),
                Type::U64 => self.eval_u64(value_str),
                Type::USize => self.eval_usize(value_str),
                Type::Ptr(_) => todo!(),
                Type::Void => todo!(),
            }
        } else {
            self.eval_i32(value_str)
        }
    }
    fn eval_variable_ref(
        &self,
        name: &str,
        _annotation: Option<Type>,
    ) -> Result<Value, CompileError> {
        self.get_variable(&name)
    }
    fn eval_binary_expr(
        &self,
        op: BinaryOp,
        lhs: &Expression,
        rhs: &Expression,
        // _annotation: Option<Type>,
    ) -> Result<Value, CompileError> {
        pub fn get_cast_type_with_other_operand_of_bin_op(
            pointer_size: PointerSize,
            ty: &Type,
            other: &Type,
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
                Type::Void => None
            }
        }

        let (lhs_value, rhs_value) = (
            self.eval_expression(lhs, None)?,
            self.eval_expression(rhs, None)?,
        );
        let lhs_type_opt = lhs_value.get_primitive_type();
        let rhs_type_opt = rhs_value.get_primitive_type();
        if lhs_type_opt.is_none() || rhs_type_opt.is_none() {
            return Err(CompileError::InvalidOperand);
        }
        let lhs_type = lhs_type_opt.unwrap();
        let rhs_type = rhs_type_opt.unwrap();
        if lhs_type.is_integer_type() && rhs_type.is_integer_type() {
            let lhs_cast_type =
                get_cast_type_with_other_operand_of_bin_op(self.pointer_size, &lhs_type, &rhs_type);
            let rhs_cast_type =
                get_cast_type_with_other_operand_of_bin_op(self.pointer_size, &rhs_type, &lhs_type);
            let lhs_integer_value = if let Some(cast_type) = &lhs_cast_type {
                // lhs_value.cast(&self, &cast_type)
                if lhs_value.is_integer() {
                    let int_value = lhs_value.unwrap_int_value();
                    match cast_type {
                        Type::I32 => Value::I32Value(self.llvm_builder.build_int_cast_sign_flag(
                            int_value,
                            self.i32_type,
                            true,
                            "(i32)",
                        )),
                        Type::U32 => Value::U32Value(self.llvm_builder.build_int_cast_sign_flag(
                            int_value,
                            self.i32_type,
                            false,
                            "(u32)",
                        )),
                        Type::U64 => Value::U64Value(self.llvm_builder.build_int_cast_sign_flag(
                            int_value,
                            self.i64_type,
                            false,
                            "(u64)",
                        )),
                        Type::USize => {
                            Value::USizeValue(self.llvm_builder.build_int_cast_sign_flag(
                                int_value,
                                match self.pointer_size {
                                    super::PointerSize::SixteenFour => self.i64_type,
                                },
                                false,
                                "(u64)",
                            ))
                        }
                        Type::U8 => Value::U8Value(self.llvm_builder.build_int_cast_sign_flag(
                            int_value,
                            self.i8_type,
                            false,
                            "(u8)",
                        )),
                        Type::Ptr(_) => unreachable!(),
                        Type::Void => unreachable!(),
                    }
                } else {
                    unimplemented!()
                }
            } else {
                lhs_value
            };
            let rhs_integer_value = if let Some(cast_type) = &rhs_cast_type {
                if rhs_value.is_integer() {
                    let int_value = lhs_integer_value.clone().unwrap_int_value();
                    match cast_type {
                        Type::I32 => Value::I32Value(self.llvm_builder.build_int_cast_sign_flag(
                            int_value,
                            self.i32_type,
                            true,
                            "(i32)",
                        )),
                        Type::U32 => Value::U32Value(self.llvm_builder.build_int_cast_sign_flag(
                            int_value,
                            self.i32_type,
                            false,
                            "(u32)",
                        )),
                        Type::U64 => Value::U64Value(self.llvm_builder.build_int_cast_sign_flag(
                            int_value,
                            self.i64_type,
                            false,
                            "(u64)",
                        )),
                        Type::USize => {
                            Value::USizeValue(self.llvm_builder.build_int_cast_sign_flag(
                                int_value,
                                match self.pointer_size {
                                    super::PointerSize::SixteenFour => self.i64_type,
                                },
                                false,
                                "(u64)",
                            ))
                        }
                        Type::U8 => Value::U8Value(self.llvm_builder.build_int_cast_sign_flag(
                            int_value,
                            self.i8_type,
                            false,
                            "(u8)",
                        )),
                        Type::Ptr(_) => unreachable!(),
                        Type::Void => unreachable!(),
                    }
                } else {
                    unimplemented!()
                }
            } else {
                rhs_value
            };
            let result_type = lhs_cast_type.unwrap_or(rhs_cast_type.unwrap_or(lhs_type));
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
            todo!("impl float arithmetic");
        }
    }
    fn eval_call_expr<'b>(
        &'b self,
        name: &str,
        arg_exprs: &[Expression],
        _annotation: Option<Type>,
    ) -> Result<Value, CompileError> {
        let context = self.context.borrow();
        let result = context.find_function(&name);
        if let Some((_return_type, arg_types, func)) = result {
            let mut evaluated_args: Vec<BasicMetadataValueEnum> = Vec::new();
            assert_eq!(arg_exprs.len(), arg_types.len());
            for i in 0..arg_exprs.len() {
                let arg_expr = arg_exprs.get(i).unwrap();
                let arg_type = arg_types.get(i).unwrap();
                let evaluated_arg = self.eval_expression(arg_expr, Some(arg_type.clone()))?;
                evaluated_args.push(match evaluated_arg {
                    Value::U8Value(v) => BasicMetadataValueEnum::IntValue(v),
                    Value::I32Value(v) => BasicMetadataValueEnum::IntValue(v),
                    Value::U32Value(v) => BasicMetadataValueEnum::IntValue(v),
                    Value::U64Value(v) => BasicMetadataValueEnum::IntValue(v),
                    Value::USizeValue(v) => BasicMetadataValueEnum::IntValue(v),
                    Value::PointerValue(v) => BasicMetadataValueEnum::PointerValue(v),
                    Value::Void => return Err(CompileError::InvalidArgument),
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
                        BasicValueEnum::PointerValue(v) => Value::PointerValue(v),
                        BasicValueEnum::StructValue(_) => todo!(),
                        BasicValueEnum::VectorValue(_) => todo!(),
                    },
                    None => Value::Void,
                },
            )
        } else {
            if context.find_variable(&name).is_some() {
                Err(CompileError::CallNotFunctionValue {
                    name: name.to_string(),
                })
            } else {
                Err(CompileError::FunctionNotFound {
                    name: name.to_string(),
                })
            }
        }
    }
    pub(super) fn eval_expression(
        &self,
        expr: &Expression,
        annotation: Option<Type>,
    ) -> Result<Value, CompileError> {
        match expr {
            Expression::VariableRef { name } => self.eval_variable_ref(&name, annotation),
            Expression::NumberLiteral { value } => self.eval_integer_literal(&value, annotation),
            Expression::BinaryExpr { op, lhs, rhs } => self.eval_binary_expr(*op, lhs, rhs),
            Expression::CallExpr { name, args } => self.eval_call_expr(&name, args, annotation),
        }
    }
}
