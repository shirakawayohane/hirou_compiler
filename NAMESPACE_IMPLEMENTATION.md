# Namespace System Implementation

## Overview
This document describes the implementation of the namespace system for the Hirou compiler, which allows organizing functions into namespaces and calling them using the `::` operator (e.g., `Vec::new`, `Vec::push`).

## Design Decisions

### Syntax Choice: `::` vs `.`
- **Chosen**: `::` (double colon) for namespace separator
- **Rationale**:
  - Aligns with Rust and C++ conventions
  - Clearly distinguishes from field access (`.`)
  - Already used in type paths (`Vec<T>::new`)

### Function Call Syntax
Functions can now be called with namespaced names:
```lisp
;; Old style (no longer supported)
(vec_new<i32>)
(vec_get<i32> v 0)

;; New style (namespaced)
(Vec::new<i32>)
(Vec::get<i32> v 0)
```

### Function Definition Syntax
Functions can be defined with namespaced names:
```lisp
;; Define a namespaced function
fn Vec::new<T>(): Vec<T> {
  ...
}

fn Vec::get<T>(vec: Vec<T>, index: i32): T {
  ...
}
```

## Implementation Details

### 1. AST Changes (`src/ast.rs`)

#### NamespacePath
New type to represent namespace paths:
```rust
pub struct NamespacePath {
    pub segments: Vec<String>,
}

impl NamespacePath {
    pub fn simple(name: String) -> Self {
        Self { segments: vec![name] }
    }

    pub fn to_string(&self) -> String {
        self.segments.join("::")
    }

    pub fn is_namespaced(&self) -> bool {
        self.segments.len() > 1
    }
}
```

#### CallExpr Update
Modified to use `NamespacePath` instead of plain `String`:
```rust
pub struct CallExpr {
    pub name: NamespacePath,  // Changed from String
    pub generic_args: Option<Vec<Located<UnresolvedType>>>,
    pub args: Vec<LocatedExpr>,
}
```

#### UseStatement
Added support for `use` statements (parsing only, resolution not yet implemented):
```rust
pub struct UseStatement {
    pub path: NamespacePath,
    pub wildcard: bool, // true for `use Vec::*`
}

pub enum TopLevel {
    Function(Function),
    Implementation(Implementation),
    TypeDef(TypeDef),
    Interface(Interface),
    Use(UseStatement),  // New variant
}
```

### 2. Parser Changes

#### Token Parser (`src/parser/token.rs`)
Added new tokens:
```rust
token_tag!(use_token, "use");
token_tag!(double_colon, "::");
```

Added namespace path parser:
```rust
pub(super) fn parse_namespace_path(input: Span) -> NotLocatedParseResult<NamespacePath> {
    use nom::multi::separated_list1;
    map(
        separated_list1(double_colon, parse_identifier),
        |segments| NamespacePath { segments }
    )(input)
}
```

#### Expression Parser (`src/parser/expression.rs`)
Updated function call parser to use namespace paths:
```rust
pub(super) fn parse_function_call_expression(input: Span) -> NotLocatedParseResult<Expression> {
    map(
        delimited(
            lparen,
            tuple((
                parse_namespace_path,  // Changed from parse_identifier
                opt(parse_generic_arguments),
                parse_arguments,
            )),
            rparen,
        ),
        |(name, generic_args, args)| {
            Expression::Call(CallExpr {
                name,
                generic_args,
                args,
            })
        },
    )(input)
}
```

#### Top-level Parser (`src/parser/toplevel.rs`)
Updated function declaration parser:
```rust
fn parse_function_decl(input: Span) -> ParseResult<FunctionDecl> {
    context(
        "function_decl",
        located(map(
            tuple((
                opt(parse_alloc_mode),
                fn_token,
                parse_namespace_path,  // Changed from parse_identifier
                opt(parse_generic_argument_decls),
                parse_arguments,
                map(tuple((colon, parse_type)), |(_, ty)| ty),
            )),
            |(alloc_mode, _, name_path, generic_args, params, ty)| FunctionDecl {
                alloc_mode,
                name: name_path.to_string(),  // Convert to string for storage
                generic_args,
                args: params,
                return_type: ty,
                is_intrinsic: false,
            },
        )),
    )(input)
}
```

Added use statement parser:
```rust
fn parse_use_statement(input: Span) -> ParseResult<TopLevel> {
    let (s, _) = peek(use_token)(input)?;
    cut(located(context(
        "use_statement",
        map(
            tuple((
                use_token,
                parse_namespace_path,
                opt(tuple((double_colon, asterisk))),
            )),
            |(_, path, wildcard_opt)| {
                TopLevel::Use(UseStatement {
                    path,
                    wildcard: wildcard_opt.is_some(),
                })
            },
        ),
    )))(s)
}
```

### 3. Resolver Changes (`src/resolver/`)

#### Call Expression Resolution (`src/resolver/expression/call.rs`)
Updated to convert `NamespacePath` to string for function lookup:
```rust
pub fn resolve_call_expr(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    annotation: Option<&ResolvedType>,
) -> Result<ResolvedExpression, FaitalError> {
    // Convert namespace path to string for lookup
    let function_name = call_expr.name.to_string();

    let function_by_name = context.function_by_name.borrow();
    let interface_by_name = context.interface_by_name.borrow();
    let impls_by_name = context.impls_by_name.borrow();

    if let Some(callee) = function_by_name.get(&function_name) {
        resolve_function_call_expr(context, call_expr, callee, annotation)
    } else if let Some(interface) = interface_by_name.get(&function_name) {
        // ... interface resolution
    } else {
        context.errors.borrow_mut().push(CompileError::new(
            call_expr.range,
            CompileErrorKind::FunctionNotFound {
                name: function_name,
            },
        ));
        Ok(ResolvedExpression {
            ty: ResolvedType::Unknown,
            kind: ExpressionKind::Unknown,
        })
    }
}
```

#### Module Resolution (`src/resolver/mod.rs`)
Added handling for `Use` top-level items:
```rust
for toplevel in &module.toplevels {
    match &toplevel.value {
        TopLevel::Function(func) => { /* ... */ }
        TopLevel::TypeDef(typedef) => { /* ... */ }
        TopLevel::Interface(interface) => { /* ... */ }
        TopLevel::Implementation(_) => ()
        TopLevel::Use(_) => (), // Use statements are processed separately
    }
}
```

### 4. Standard Library Changes (`src/resolver/stdlib.rs`)

Updated all Vec functions to use namespaced names:
```rust
const STDLIB_DEFINITIONS: &str = r#"
struct Vec<T> {
    capacity: i32,
    size: i32,
    buf: *T,
}

fn Vec::new<T>(): Vec<T> {
    (:= alloc_size (sizeof T))
    (:= buf: *T (malloc alloc_size))
    Vec<T> {
        capacity: 1,
        size: 0,
        buf: buf
    }
}

fn Vec::get<T>(vec: Vec<T>, index: i32): T {
    (:= buf vec.buf)
    (:= idx : usize index)
    buf[idx]
}

// ... other Vec functions with Vec:: prefix
"#;
```

## Testing

### Test Files
Created comprehensive test files:

1. **`sample/test_namespace.hr`** - Basic namespace functionality
   - Tests `Vec::new`, `Vec::get`, `Vec::set`, etc.
   - Verifies namespace syntax works correctly
   - All tests pass ✅

2. **`sample/test_namespace_use.hr`** - Use statement documentation
   - Documents intended use statement syntax
   - Tests qualified names
   - All tests pass ✅

3. **`sample/test_stdlib_vec.hr`** - Updated stdlib tests
   - Updated from old `vec_new` style to `Vec::new` style
   - Comprehensive Vec operation tests
   - All tests pass ✅

### Test Results
```bash
$ cargo run -- sample/test_namespace.hr
=== Test Namespaced Vec Functions ===
Vec from Vec::new: size=0, is_empty=1
Vec from array literal: size=3
v2[0]=1, v2[1]=2, v2[2]=3
first=1, last=3
After set: v2[1]=999
=== All namespace tests passed! ===
```

## Future Work

### Use Statement Name Resolution
Currently, `use` statements are parsed but not resolved. Future implementation should:

1. Track imported names in scope
2. When resolving calls, check imports first
3. Support both qualified (`Vec::push`) and unqualified (`push` after `use Vec::*`) calls

Example of desired functionality:
```lisp
use Vec::*

fn example(): void {
  (:= v (new<i32>))        ;; Calls Vec::new
  (set<i32> v 0 42)        ;; Calls Vec::set
  (:= x (get<i32> v 0))    ;; Calls Vec::get
}
```

### Module System
Extend namespaces to support multi-level hierarchies:
```lisp
use std::collections::Vec::*
(Vec::new<i32>)
```

### Aliasing
Support renaming imports:
```lisp
use Vec::new as create
(:= v (create<i32>))
```

## Breaking Changes

### Old Code Migration
Old code using flat function names (e.g., `vec_new`, `vec_get`) will need to be updated to use the new namespaced syntax (e.g., `Vec::new`, `Vec::get`).

**Migration Guide:**
- `vec_new<T>` → `Vec::new<T>`
- `vec_get<T>` → `Vec::get<T>`
- `vec_set<T>` → `Vec::set<T>`
- `vec_len<T>` → `Vec::len<T>`
- `vec_capacity<T>` → `Vec::capacity<T>`
- `vec_is_empty<T>` → `Vec::is_empty<T>`
- `vec_first<T>` → `Vec::first<T>`
- `vec_last<T>` → `Vec::last<T>`
- `vec_free<T>` → `Vec::free<T>`

## Implementation Summary

The namespace system was implemented with minimal changes to the codebase:
- **AST**: Added `NamespacePath` type and updated `CallExpr`
- **Parser**: Added `::` token and namespace path parsing
- **Resolver**: Convert namespace paths to strings for lookup
- **Stdlib**: Updated function names to use namespaces

The implementation maintains backward compatibility at the AST level (functions are still stored with string names) while providing a clean namespace syntax at the source code level.

## References

- Issue/Feature Request: Namespace system for Hirou compiler
- Design Discussion: `::` vs `.` for namespace separator
- Implementation PR: [To be added]
