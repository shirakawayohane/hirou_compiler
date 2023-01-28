use crate::ast::{BinaryOp, Expression, FunctionDecl, Module, Statement, TopLevel};
use inkwell::builder::Builder as LLVMBuilder;
use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, IntValue, PointerValue};
use inkwell::AddressSpace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Variable `{name:?}` is not found in this scope.")]
    VariableNotFound { name: String },
    #[error("Function `{name:?}` is not found.")]
    FunctionNotFound { name: String },
    #[error("`{name:?}` is not a function")]
    CallNotFunctionValue { name: String },
    #[error("Invalid operand.")]
    InvalidOperand,
    #[error("Invalid operand.")]
    InvalidArgument,
}

pub struct Context<'a> {
    scopes: Vec<HashMap<String, PointerValue<'a>>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context { scopes: Vec::new() }
    }
    pub fn find_variable(&'a self, name: &str) -> Option<PointerValue<'a>> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(*v);
            }
        }
        None
    }
    pub fn set_variable(&mut self, name: String, value: PointerValue<'a>) {
        self.scopes.last_mut().unwrap().insert(name, value);
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}

pub struct LLVMCodegenerator<'a> {
    pub context: Rc<RefCell<Context<'a>>>,
    pub llvm_module: LLVMModule<'a>,
    pub llvm_builder: LLVMBuilder<'a>,
    pub llvm_context: &'a LLVMContext,
}

impl<'a> LLVMCodegenerator<'a> {
    pub fn new(llvm_context: &'a LLVMContext) -> LLVMCodegenerator<'a> {
        let llvm_module = llvm_context.create_module("main");
        let llvm_builder = llvm_context.create_builder();
        Self {
            context: Rc::new(RefCell::new(Context::new())),
            llvm_module,
            llvm_builder,
            llvm_context,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Value<'a> {
    IntValue(IntValue<'a>),
    Void,
}

impl LLVMCodegenerator<'_> {
    fn get_variable(&self, name: &str) -> Result<PointerValue, CompileError> {
        for scope in self.context.borrow().scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Ok(*v);
            }
        }
        Err(CompileError::VariableNotFound {
            name: name.to_string(),
        })
    }
    fn eval_expression(&self, expr: Expression) -> Result<Value, CompileError> {
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
    pub fn gen_statement(&self, statement: Statement) -> Result<(), CompileError> {
        match statement {
            Statement::VariableDecl { name, value } => {
                let variable_pointer = self
                    .llvm_builder
                    .build_alloca(self.llvm_context.i32_type(), &name);

                // Contextに登録
                self.context
                    .borrow_mut()
                    .set_variable(name, variable_pointer);

                match self.eval_expression(value)? {
                    Value::IntValue(v) => {
                        self.llvm_builder.build_store(variable_pointer, v);
                    }
                    Value::Void => {}
                }
            }
            Statement::Return { expression } => {
                if let Some(exp) = expression {
                    let value = self.eval_expression(exp)?;
                    self.llvm_builder.build_return(match &value {
                        Value::IntValue(v) => Some(v),
                        Value::Void => None,
                    });
                } else {
                    self.llvm_builder.build_return(None);
                }
            }
            Statement::Asignment { name, expression } => {
                if let Some(pointer) = self.context.borrow().find_variable(&name) {
                    let value = self.eval_expression(expression)?;
                    self.llvm_builder.build_store(
                        pointer,
                        match value {
                            Value::IntValue(v) => v,
                            Value::Void => return Ok(()),
                        },
                    );
                } else {
                    return Err(CompileError::VariableNotFound { name });
                }
            }
            Statement::DiscardedExpression { expression } => {
                self.eval_expression(expression)?;
            }
        };
        Ok(())
    }
    fn gen_function(&self, decl: FunctionDecl, body: Vec<Statement>) -> Result<(), CompileError> {
        // TODO: int以外の型にも対応する
        let params = decl
            .params
            .iter()
            .map(|_| self.llvm_context.i32_type().into())
            .collect::<Vec<_>>();
        let fn_type = self.llvm_context.i32_type().fn_type(&params, true);
        let function = self.llvm_module.add_function(&decl.name, fn_type, None);
        let entry_basic_block = self.llvm_context.append_basic_block(function, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        // パラメーターをFunctionBodyにallocし、Contextにも登録する
        self.context.borrow_mut().push_scope();
        // Set parameters in function body
        for (i, parameter) in function.get_param_iter().enumerate() {
            let parameter_name = &decl.params[i];
            parameter.set_name(parameter_name.as_str());
            let alloca = self
                .llvm_builder
                .build_alloca(parameter.get_type(), &parameter_name);
            self.llvm_builder.build_store(alloca, parameter);
            self.context
                .borrow_mut()
                .set_variable(parameter_name.clone(), alloca);
        }

        for statement in body {
            self.gen_statement(statement)?;
        }

        self.context.borrow_mut().pop_scope();
        Ok(())
    }
    pub fn gen_intrinsic_functions(&self) {
        // printf
        let i32_type = self.llvm_context.i32_type();
        let i8_ptr_type = self
            .llvm_context
            .i8_type()
            .ptr_type(AddressSpace::default());
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf_function = self.llvm_module.add_function("printf", printf_type, None);

        // gen printi32 function
        let void_type = self.llvm_context.void_type();
        let print_i32_type = void_type.fn_type(&[i32_type.into()], false);
        let print_i32_function = self
            .llvm_module
            .add_function("printi32", print_i32_type, None);
        let entry_basic_block = self
            .llvm_context
            .append_basic_block(print_i32_function, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        let digit_format_string_ptr = self
            .llvm_builder
            .build_global_string_ptr("%d", "digit_format_string");
        let argument = print_i32_function.get_first_param().unwrap();
        self.llvm_builder.build_call(
            printf_function,
            &[
                digit_format_string_ptr.as_pointer_value().into(),
                argument.into(),
            ],
            "call",
        );
        // main関数は0を返す
        self.llvm_builder.build_return(None);
    }
    pub fn gen_module(&self, module: Module) -> Result<(), CompileError> {
        self.gen_intrinsic_functions();
        for top in module.toplevels {
            match top {
                TopLevel::Function { decl, body } => self.gen_function(decl, body)?,
            }
        }
        Ok(())
    }
}
