use super::value::Value;
use super::*;
use crate::ast::*;

use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

impl LLVMCodegenerator<'_> {
    pub(super) fn eval_expression(&self, expr: Expression) -> Result<Value, CompileError> {
        match expr {
            Expression::VariableRef { name } => {
                if let Ok(ptr) = self.get_variable(&name) {
                    let value: BasicValueEnum<'_> = self.llvm_builder.build_load(ptr, &name);
                    Ok(match value {
                        BasicValueEnum::ArrayValue(_) => todo!(),
                        BasicValueEnum::IntValue(v) => Value::IntValue(v),
                        BasicValueEnum::FloatValue(_) => todo!(),
                        BasicValueEnum::PointerValue(_) => todo!(),
                        BasicValueEnum::StructValue(_) => todo!(),
                        BasicValueEnum::VectorValue(_) => todo!(),
                    })
                } else {
                    Err(CompileError::VariableNotFound {
                        name: name.to_string(),
                    })
                }
            }
            Expression::IntValue { value } => {
                let literal = self.llvm_context.i32_type().const_int(value as u64, true);
                Ok(Value::IntValue(literal))
            }
            Expression::BinaryExpr { op, lhs, rhs } => {
                let lhs_value = self.eval_expression(*lhs)?;
                let rhs_value = self.eval_expression(*rhs)?;
                match op {
                    BinaryOp::Add => match lhs_value {
                        Value::IntValue(lhs_int_value) => match rhs_value {
                            Value::IntValue(rhs_int_value) => {
                                Ok(Value::IntValue(self.llvm_builder.build_int_add(
                                    lhs_int_value,
                                    rhs_int_value,
                                    "add_int_int",
                                )))
                            }
                            Value::Void => return Err(CompileError::InvalidOperand),
                        },
                        Value::Void => return Err(CompileError::InvalidOperand),
                    },
                    BinaryOp::Sub => match lhs_value {
                        Value::IntValue(lhs_int_value) => match rhs_value {
                            Value::IntValue(rhs_int_value) => {
                                Ok(Value::IntValue(self.llvm_builder.build_int_sub(
                                    lhs_int_value,
                                    rhs_int_value,
                                    "sub_int_int",
                                )))
                            }
                            Value::Void => return Err(CompileError::InvalidOperand),
                        },
                        Value::Void => return Err(CompileError::InvalidOperand),
                    },
                    BinaryOp::Mul => match lhs_value {
                        Value::IntValue(lhs_int_value) => match rhs_value {
                            Value::IntValue(rhs_int_value) => {
                                Ok(Value::IntValue(self.llvm_builder.build_int_mul(
                                    lhs_int_value,
                                    rhs_int_value,
                                    "mul_int_int",
                                )))
                            }
                            Value::Void => return Err(CompileError::InvalidOperand),
                        },
                        Value::Void => return Err(CompileError::InvalidOperand),
                    },
                    BinaryOp::Div => match lhs_value {
                        Value::IntValue(lhs_int_value) => match rhs_value {
                            Value::IntValue(rhs_int_value) => {
                                Ok(Value::IntValue(self.llvm_builder.build_int_signed_div(
                                    lhs_int_value,
                                    rhs_int_value,
                                    "div_int_int",
                                )))
                            }
                            Value::Void => return Err(CompileError::InvalidOperand),
                        },
                        Value::Void => return Err(CompileError::InvalidOperand),
                    },
                }
            }
            Expression::CallExpr { name, args } => {
                if let Some(func) = self.llvm_module.get_function(&name) {
                    let mut evaluated_args: Vec<BasicMetadataValueEnum> = Vec::new();
                    for arg_expr in args {
                        let evaluated_arg = self.eval_expression(arg_expr)?;
                        evaluated_args.push(match evaluated_arg {
                            Value::IntValue(v) => BasicMetadataValueEnum::IntValue(v),
                            Value::Void => return Err(CompileError::InvalidArgument),
                        });
                    }
                    Ok(
                        match self
                            .llvm_builder
                            .build_call(func, &evaluated_args, "function_call")
                            .try_as_basic_value()
                            .left()
                        {
                            Some(returned_value) => match returned_value {
                                BasicValueEnum::ArrayValue(_) => todo!(),
                                BasicValueEnum::IntValue(int_value) => Value::IntValue(int_value),
                                BasicValueEnum::FloatValue(_) => todo!(),
                                BasicValueEnum::PointerValue(_) => todo!(),
                                BasicValueEnum::StructValue(_) => todo!(),
                                BasicValueEnum::VectorValue(_) => todo!(),
                            },
                            None => Value::Void,
                        },
                    )
                } else {
                    if self.context.borrow().find_variable(&name).is_some() {
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
        }
    }
}