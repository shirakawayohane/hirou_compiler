use crate::{
    concrete_ast::{self, ConcreteExpression, ConcreteType, Function},
    resolved_ast::{self, ResolvedType},
};

use super::ConcretizerContext;

fn concretize_type(ctx: &ConcretizerContext, ty: &ResolvedType) -> ConcreteType {
    match ty {
        ResolvedType::I32 => ConcreteType::I32,
        ResolvedType::I64 => ConcreteType::I64,
        ResolvedType::U32 => ConcreteType::U32,
        ResolvedType::U64 => ConcreteType::U64,
        ResolvedType::USize => {
            if ctx.is_64_bit() {
                ConcreteType::U64
            } else {
                ConcreteType::U32
            }
        }
        ResolvedType::U8 => ConcreteType::U8,
        ResolvedType::Bool => ConcreteType::Bool,
        ResolvedType::Ptr(inner) => ConcreteType::Ptr(Box::new(concretize_type(ctx, inner))),
        ResolvedType::Void => ConcreteType::Void,
        ResolvedType::Unknown => panic!("Unknown type should not reach concretizer"),
        ResolvedType::StructLike(struct_ty) => {
            let fields = struct_ty
                .fields
                .iter()
                .map(|(name, ty)| (name.clone(), concretize_type(ctx, ty)))
                .collect();
            ConcreteType::StructLike(concrete_ast::ConcreteStructType {
                name: struct_ty.name.clone(),
                non_generic_name: struct_ty.non_generic_name.clone(),
                fields,
            })
        }
        ResolvedType::Generics(_) => {
            panic!("Generic type should be resolved before concretizer")
        }
    }
}

fn concretize_expression(
    ctx: &ConcretizerContext,
    expr: &resolved_ast::ResolvedExpression,
) -> ConcreteExpression {
    let ty = concretize_type(ctx, &expr.ty);
    let kind = match &expr.kind {
        resolved_ast::ExpressionKind::SizeOf(resolved_ty) => {
            concrete_ast::ExpressionKind::SizeOf(concretize_type(ctx, resolved_ty))
        }
        resolved_ast::ExpressionKind::VariableRef(var_ref) => {
            concrete_ast::ExpressionKind::VariableRef(concrete_ast::VariableRefExpr {
                name: var_ref.name.clone(),
            })
        }
        resolved_ast::ExpressionKind::NumberLiteral(num) => {
            concrete_ast::ExpressionKind::NumberLiteral(concrete_ast::NumberLiteral {
                value: num.value.clone(),
            })
        }
        resolved_ast::ExpressionKind::StringLiteral(str_lit) => {
            concrete_ast::ExpressionKind::StringLiteral(concrete_ast::StringLiteral {
                value: str_lit.value.clone(),
            })
        }
        resolved_ast::ExpressionKind::StructLiteral(struct_lit) => {
            let fields = struct_lit
                .fields
                .iter()
                .map(|(name, expr)| (name.clone(), concretize_expression(ctx, expr)))
                .collect();
            concrete_ast::ExpressionKind::StructLiteral(concrete_ast::StructLiteral { fields })
        }
        resolved_ast::ExpressionKind::BoolLiteral(bool_lit) => {
            concrete_ast::ExpressionKind::BoolLiteral(concrete_ast::BoolLiteral {
                value: bool_lit.value,
            })
        }
        resolved_ast::ExpressionKind::Binary(bin_expr) => {
            concrete_ast::ExpressionKind::Binary(concrete_ast::BinaryExpr {
                op: bin_expr.op,
                lhs: Box::new(concretize_expression(ctx, &bin_expr.lhs)),
                rhs: Box::new(concretize_expression(ctx, &bin_expr.rhs)),
            })
        }
        resolved_ast::ExpressionKind::Unary(unary_expr) => {
            concrete_ast::ExpressionKind::Unary(concrete_ast::UnaryExpr {
                op: unary_expr.op,
                operand: Box::new(concretize_expression(ctx, &unary_expr.operand)),
            })
        }
        resolved_ast::ExpressionKind::Multi(multi_expr) => {
            concrete_ast::ExpressionKind::Multi(concrete_ast::MultiExpr {
                op: multi_expr.op,
                operands: multi_expr
                    .operands
                    .iter()
                    .map(|e| concretize_expression(ctx, e))
                    .collect(),
            })
        }
        resolved_ast::ExpressionKind::CallExpr(call_expr) => {
            concrete_ast::ExpressionKind::CallExpr(concrete_ast::CallExpr {
                callee: call_expr.callee.clone(),
                args: call_expr
                    .args
                    .iter()
                    .map(|e| concretize_expression(ctx, e))
                    .collect(),
            })
        }
        resolved_ast::ExpressionKind::Deref(deref_expr) => {
            concrete_ast::ExpressionKind::Deref(concrete_ast::DerefExpr {
                target: Box::new(concretize_expression(ctx, &deref_expr.target)),
            })
        }
        resolved_ast::ExpressionKind::IndexAccess(idx_expr) => {
            concrete_ast::ExpressionKind::IndexAccess(concrete_ast::IndexAccessExpr {
                target: Box::new(concretize_expression(ctx, &idx_expr.target)),
                index: Box::new(concretize_expression(ctx, &idx_expr.index)),
            })
        }
        resolved_ast::ExpressionKind::FieldAccess(field_expr) => {
            concrete_ast::ExpressionKind::FieldAccess(concrete_ast::FieldAccessExpr {
                target: Box::new(concretize_expression(ctx, &field_expr.target)),
                field_name: field_expr.field_name.clone(),
            })
        }
        resolved_ast::ExpressionKind::If(if_expr) => {
            concrete_ast::ExpressionKind::If(concrete_ast::IfExpr {
                cond: Box::new(concretize_expression(ctx, &if_expr.cond)),
                then: Box::new(concretize_expression(ctx, &if_expr.then)),
                els: Box::new(concretize_expression(ctx, &if_expr.els)),
            })
        }
        resolved_ast::ExpressionKind::When(when_expr) => {
            concrete_ast::ExpressionKind::When(concrete_ast::WhenExpr {
                cond: Box::new(concretize_expression(ctx, &when_expr.cond)),
                then: Box::new(concretize_expression(ctx, &when_expr.then)),
            })
        }
        resolved_ast::ExpressionKind::VariableDecls(decls) => {
            concrete_ast::ExpressionKind::VariableDecls(concrete_ast::VariableDecls {
                decls: decls
                    .decls
                    .iter()
                    .map(|d| concrete_ast::VariableDecl {
                        name: d.name.clone(),
                        value: Box::new(concretize_expression(ctx, &d.value)),
                    })
                    .collect(),
            })
        }
        resolved_ast::ExpressionKind::Assignment(assign) => {
            concrete_ast::ExpressionKind::Assignment(concrete_ast::Assignment {
                name: assign.name.clone(),
                value: Box::new(concretize_expression(ctx, &assign.value)),
                deref_count: assign.deref_count,
                index_access: assign
                    .index_access
                    .as_ref()
                    .map(|e| Box::new(concretize_expression(ctx, e))),
            })
        }
        resolved_ast::ExpressionKind::Unknown => concrete_ast::ExpressionKind::Unknown,
    };
    ConcreteExpression { ty, kind }
}

fn concretize_statement(
    ctx: &ConcretizerContext,
    stmt: &resolved_ast::Statement,
) -> ConcreteExpression {
    match stmt {
        resolved_ast::Statement::Return(ret) => {
            let ty = ret
                .expression
                .as_ref()
                .map(|e| concretize_type(ctx, &e.ty))
                .unwrap_or(ConcreteType::Void);
            ConcreteExpression {
                ty,
                kind: concrete_ast::ExpressionKind::Return(concrete_ast::Return {
                    expression: ret
                        .expression
                        .as_ref()
                        .map(|e| Box::new(concretize_expression(ctx, e))),
                }),
            }
        }
        resolved_ast::Statement::Effect(effect) => concretize_expression(ctx, &effect.expression),
    }
}

fn concretize_argument(
    ctx: &ConcretizerContext,
    arg: &resolved_ast::Argument,
) -> concrete_ast::Argument {
    match arg {
        resolved_ast::Argument::VarArgs => concrete_ast::Argument::VarArgs,
        resolved_ast::Argument::Normal(ty, name) => {
            concrete_ast::Argument::Normal(concretize_type(ctx, ty), name.clone())
        }
    }
}

fn concretize_function(ctx: &ConcretizerContext, func: &resolved_ast::Function) -> Function {
    let args = func
        .decl
        .args
        .iter()
        .map(|a| concretize_argument(ctx, a))
        .collect();
    let return_type = concretize_type(ctx, &func.decl.return_type);
    let body = func
        .body
        .iter()
        .map(|stmt| concretize_statement(ctx, stmt))
        .collect();

    Function {
        decl: concrete_ast::FunctionDecl {
            name: func.decl.name.clone(),
            args,
            return_type,
        },
        body,
    }
}

pub fn concretize_toplevel(
    ctx: &ConcretizerContext,
    toplevel: &resolved_ast::TopLevel,
) -> Option<Vec<concrete_ast::TopLevel>> {
    match toplevel {
        resolved_ast::TopLevel::Function(func) => {
            let concretized = concretize_function(ctx, func);
            Some(vec![concrete_ast::TopLevel::Function(concretized)])
        }
    }
}
