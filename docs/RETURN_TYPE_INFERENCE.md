# Generic Return Type Inference from Context

## Overview

This feature allows calling generic functions without explicit type arguments when the return type can be inferred from the annotation context.

## Feature Description

### Before
```lisp
(:= v : Vec<i32> (Vec::new<i32>))  // Had to specify <i32> explicitly
```

### After
```lisp
(:= v : Vec<i32> (Vec::new))       // T=i32 inferred from annotation
```

## How It Works

The compiler now attempts to infer generic type parameters in three stages (in order):

1. **Explicit generic arguments** - If provided, use them directly
2. **Annotation-based inference** (NEW) - Infer from the variable type annotation
3. **Argument-based inference** - Infer from the function call arguments

### Inference from Annotation

When a generic function is called without type arguments:
- The compiler checks if there's a type annotation on the variable
- It matches the function's return type against the annotation
- Generic parameters are inferred from the match

Example:
```
fn Vec::new<T>(): Vec<T>
```

When called as:
```lisp
(:= v : Vec<i32> (Vec::new))
```

The compiler:
1. Sees return type is `Vec<T>`
2. Sees annotation is `Vec<i32>`
3. Matches `Vec<T>` against `Vec<i32>`
4. Infers `T = i32`

## Use Cases

### 1. Creating Generic Containers
```lisp
// Create vectors of different types
(:= v_int : Vec<i32> (Vec::new))
(:= v_bool : Vec<bool> (Vec::new))
(:= v_float : Vec<f32> (Vec::new))
```

### 2. Chained Operations
```lisp
(:= v : Vec<i32> (Vec::new))
(:=< v (Vec::push v 10))    // T inferred from v (argument inference)
(:=< v (Vec::push v 20))
(:=< v (Vec::pop v))
```

### 3. Multiple Vectors
```lisp
fn process(): void {
  (:= vi : Vec<i32> (Vec::new))
  (:= vb : Vec<bool> (Vec::new))

  (:=< vi (Vec::push vi 42))
  (:=< vb (Vec::push vb true))
}
```

## Implementation Details

### Location
`src/resolver/expression/call.rs`

### Key Functions

#### `infer_generic_args_recursively`
Recursively matches the function's return type against the annotation to infer generic parameters.

**Fixed Bug**: Line 101 was incorrectly indexing `resolved_generic_ty[i]` when it should just use `resolved_generic_ty` directly. Also needed to properly access `generic_args[i].value`.

#### `resolve_infer_generic_from_annotation`
Entry point for annotation-based inference. Called in the resolution flow when:
- No explicit generic arguments are provided
- The function has generic parameters
- An annotation is available

### Resolution Flow

In `resolve_function_call_expr`:

```rust
if callee.decl.generic_args.is_some() {
    // 1. Try explicit generic arguments
    let explicit_resolved = resolve_call_with_generic_args(...)?;

    // 2. Try annotation inference (NEW)
    let annotation_inferred = if !explicit_resolved {
        !resolve_infer_generic_from_annotation(...)?
            .is_empty()
    } else {
        false
    };

    // 3. Try argument inference (fallback)
    if !explicit_resolved && !annotation_inferred {
        let (inferred_indices, inferred_types) =
            resolve_infer_generic_from_arguments(...)?;
        ...
    }
}
```

## Test Coverage

### Test Files
- `sample/test_return_type_inference.hr` - Basic return type inference
- `sample/test_return_type_inference_comprehensive.hr` - Comprehensive test suite

### Test Cases
1. **Basic inference**: Infer T from annotation for i32, bool, u32
2. **Operations**: Vec methods work after annotation-based creation
3. **Reassignment**: Type persists across reassignments
4. **Multiple vectors**: Different types in same scope
5. **Chained operations**: Push, pop, etc. all work together

### Running Tests
```bash
cargo run -- sample/test_return_type_inference.hr
cargo run -- sample/test_return_type_inference_comprehensive.hr
cargo run -- sample/test_inference.hr
```

## Backward Compatibility

The feature is fully backward compatible:
- Explicit type arguments still work: `(Vec::new<i32>)`
- Argument inference still works: `(Vec::push v 10)` where v is Vec<i32>
- All existing tests pass

## Limitations

1. Requires an annotation on the variable declaration
2. Cannot infer from nested expression contexts (e.g., inside if expressions)
3. Only works for return type matching (not parameter types)

## Future Enhancements

Possible future improvements:
- Bidirectional type inference
- Inference from usage context beyond just annotations
- Support for more complex type patterns
