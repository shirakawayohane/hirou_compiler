# Generic Return Type Inference - Implementation Report

## Overview

The Hirou compiler successfully implements **generic return type inference from context**. This allows calling generic functions without explicit type arguments when the generic types can be inferred from the expected return type annotation.

## Feature Description

### What Works

You can now write:
```lisp
(:= v : Vec<i32> (Vec::new))
```

Instead of requiring:
```lisp
(:= v : Vec<i32> (Vec::new<i32>))
```

The compiler infers `T = i32` by matching:
- Function signature: `fn Vec::new<T>(): Vec<T>`
- Expected return type: `Vec<i32>`
- Inference: `Vec<T>` matches `Vec<i32>`, therefore `T = i32`

## Implementation Details

### Key Components

The implementation is located in `/Users/mikuto/source/shirakawayohane/hirou_compiler/src/resolver/expression/call.rs`:

#### 1. Main Resolution Flow (`resolve_function_call_expr`, lines 317-470)

For generic functions, the resolution happens in this order:

1. **Explicit generic arguments** (line 345)
   - If the call has explicit type args like `(Vec::new<i32>)`, use them

2. **Return type inference** (lines 348-353)
   - If no explicit args, try to infer from the expected return type (annotation)
   - Calls `resolve_infer_generic_from_annotation`

3. **Argument type inference** (lines 356-400)
   - If return type inference didn't work, try to infer from argument types
   - This is the existing "Category 1" inference

#### 2. Return Type Inference Function (`resolve_infer_generic_from_annotation`, lines 260-296)

This function:
- Takes the expected return type (annotation) from the variable declaration
- Compares it against the function's declared return type
- Calls `infer_generic_args_recursively` to perform the matching

Key behavior:
- Returns a vector of inferred generic argument indices
- Works in global scope to properly register the inferred types
- Only runs if no explicit type args are provided

#### 3. Recursive Matching (`infer_generic_args_recursively`, lines 64-139)

This is the core matching algorithm that recursively compares:
- The function's return type (unresolved, contains generic parameters like `T`)
- The expected type from annotation (resolved, like `Vec<i32>`)

The algorithm handles:

**Direct generic parameter matching** (lines 74-86):
```rust
// If return type is just "T" and annotation is "i32"
// Then T = i32
if return_ty_typeref.generic_args.is_none() {
    if let Some(generic_arg) = callee_generic_args.iter()
        .find(|x| x.value.name == return_ty_typeref.name)
    {
        context.types.borrow_mut()
            .add(generic_arg.value.name.clone(), current_annotation.clone());
        return Ok(true);
    }
}
```

**Nested generic matching** (lines 88-119):
```rust
// If return type is Vec<T> and annotation is Vec<i32>
// Recursively match: T vs i32
if let Some(generic_args) = &return_ty_typeref.generic_args {
    match current_annotation {
        ResolvedType::StructLike(resolved_struct) => {
            if resolved_struct.non_generic_name == return_ty_typeref.name {
                for (i, resolved_generic_ty) in resolved_generic_args.iter().enumerate() {
                    if infer_generic_args_recursively(..., &generic_args[i].value, resolved_generic_ty)? {
                        generic_arg_inferred = true;
                    }
                }
            }
        }
    }
}
```

**Pointer type matching** (lines 121-132):
```rust
// If return type is *T and annotation is *i32
// Recursively match: T vs i32
UnresolvedType::Ptr(return_ty_pointer_ty) => {
    if let ResolvedType::Ptr(inner) = current_annotation {
        if infer_generic_args_recursively(..., return_ty_pointer_ty, inner)? {
            return Ok(true);
        }
    }
}
```

### Execution Flow Example

For the call `(:= v : Vec<i32> (Vec::new))`:

1. **Parser**: Creates CallExpr for `Vec::new` with no generic_args
2. **Resolver**: Calls `resolve_call_expr` with annotation = `Vec<i32>`
3. **resolve_function_call_expr**: Detects generic function (`Vec::new<T>`)
4. **resolve_infer_generic_from_annotation**:
   - Gets function return type: `Vec<T>` (unresolved)
   - Gets annotation: `Vec<i32>` (resolved)
   - Calls recursive matcher
5. **infer_generic_args_recursively**:
   - Sees `Vec<T>` vs `Vec<i32>`
   - Names match (`Vec` == `Vec`)
   - Recursively compares generic args: `T` vs `i32`
   - Finds `T` is a generic parameter
   - Registers `T = i32` in type context
6. **resolve_function**: Resolves the function body with `T = i32`
7. **Result**: Call is resolved with correct type

## Test Coverage

Comprehensive tests are in `/Users/mikuto/source/shirakawayohane/hirou_compiler/sample/test_return_type_inference_final.hr`:

### Test Scenarios

1. **Basic inference**: `(:= v : Vec<i32> (Vec::new))`
2. **With operations**: Combining return type inference with argument inference
3. **Multiple types**: Different generic instantiations in same function
4. **Mixed with array literals**: Both inference mechanisms working together
5. **Chained operations**: Multiple operations on inferred types
6. **All Vec operations**: Comprehensive coverage of all Vec methods

All tests pass successfully.

## Comparison with Argument Inference

The compiler now supports **three types of generic inference**:

### Category 1: Argument Type Inference (Already Implemented)
```lisp
(:= v : Vec<i32> [1, 2, 3])
(:=< v (Vec::push v 10))  ; Infers T=i32 from v's type
```

### Category 2: Array Literal Inference (Already Implemented)
```lisp
(:= v : Vec<i32> [1, 2, 3])  ; Infers T=i32 from expected type
```

### Category 3: Return Type Inference (This Feature)
```lisp
(:= v : Vec<i32> (Vec::new))  ; Infers T=i32 from expected type
```

## Implementation Quality

### Strengths

1. **Prioritization**: Return type inference runs BEFORE argument inference, which is correct
2. **Scope handling**: Uses global scope correctly for type registration
3. **Recursive matching**: Handles nested generics like `Vec<Vec<T>>`
4. **Pointer support**: Handles pointer types `*T` correctly
5. **Error handling**: Temporary errors are collected and only added if inference succeeds

### Design Decisions

1. **Early return**: If explicit type args are provided, skip all inference
2. **Fallback chain**: Try return type → try arguments → fail if neither works
3. **Global scope**: Generic type bindings are registered in global scope for proper visibility

## Edge Cases Handled

1. **No annotation**: If no type annotation, skip return type inference
2. **Partial inference**: If some generics are inferred but not all, still succeeds
3. **Multiple calls**: Each call independently infers its own types
4. **Type mismatches**: If inference produces incompatible types, type checking catches it later

## Limitations

Currently does NOT handle:
- Deeply nested generics (e.g., `Vec<Vec<Vec<T>>>`) - would need more recursion depth
- Generic constraints/bounds during inference (checked after inference)
- Inter-procedural inference (each call is independent)

These are acceptable limitations for the current implementation.

## Conclusion

The return type inference feature is **fully implemented and working correctly**. It integrates seamlessly with existing argument-based inference and provides a natural, ergonomic way to work with generic functions in Hirou.

The implementation follows good software engineering practices:
- Clear separation of concerns
- Recursive algorithm for complex type matching
- Proper scope management
- Comprehensive error handling
- Well-tested with multiple scenarios

No additional implementation is needed - the feature is complete and production-ready.
