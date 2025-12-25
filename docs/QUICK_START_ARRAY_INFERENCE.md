# Quick Start: Array Literal Type Inference

## The Feature

You can now write array literals **without type annotations**:

```
// Before (still works)
(:= v : Vec<i32> [1, 2, 3])

// After (new, cleaner syntax)
(:= v [1, 2, 3])
```

## Examples

### Integer Arrays
```
fn example1(): void {
  (:= nums [1, 2, 3, 4, 5])
  (printf "Length: %d\n" (Vec::len nums))
  (printf "First: %d\n" (Vec::first nums))
  (printf "Last: %d\n" (Vec::last nums))
}
```

### Boolean Arrays
```
fn example2(): void {
  (:= flags [true, false, true])
  (printf "flags[0]: %d\n" (Vec::get flags 0))
}
```

### Float Arrays
```
fn example3(): void {
  (:= values [1.5, 2.5, 3.14])
  (printf "Length: %d\n" (Vec::len values))
}
```

### Multiple Arrays
```
fn example4(): void {
  (:= a [1, 2]
      b [3, 4, 5]
      c [10, 20, 30, 40])
  // Each array gets its type inferred independently
}
```

### Using Vec Methods
```
fn example5(): void {
  (:= nums [100, 200, 300])

  // All Vec methods work
  (printf "len=%d\n" (Vec::len nums))
  (printf "first=%d\n" (Vec::first nums))
  (printf "last=%d\n" (Vec::last nums))

  // Modify the array
  (Vec::set nums 1 999)
  (printf "nums[1]=%d\n" (Vec::get nums 1))
}
```

## Important Notes

### Empty Arrays Still Need Type Annotation
```
// This works:
(:= empty : Vec<i32> [])

// This would fail (no elements to infer from):
(:= empty [])  // Error!
```

### All Elements Must Have the Same Type
```
// This works:
(:= nums [1, 2, 3])

// This fails:
(:= mixed [1, true, 3])  // Error: Type mismatch!
```

## Error Messages

The compiler provides clear error messages:

```
error: Type does not match. expected `i32`, but got `bool`
  in sample/test.hr:5:13
   5 |  (:= v [1, true, 3])
```

## Testing

Try it yourself:

```bash
# Create a test file
cat > test.hr << 'EOF'
fn test(): void {
  (:= v [1, 2, 3])
  (printf "len=%d\n" (Vec::len v))
  (printf "v[0]=%d\n" (Vec::get v 0))
}

alloc fn main(): void {
  (test)
}
EOF

# Run it
cargo run -- test.hr
```

## Supported Types

All primitive types work:
- `i32`, `i64`, `u8`, `u32`, `u64`, `usize` (integers)
- `f32`, `f64` (floats)
- `bool` (boolean)

## How It Works

1. Compiler looks at the first array element
2. Infers the element type (e.g., `i32` from `1`)
3. Constructs `Vec<i32>` type automatically
4. Validates all other elements match the inferred type

## More Examples

See comprehensive test files:
- `sample/test_task_requirement.hr` - Basic example from task
- `sample/test_array_inference_comprehensive.hr` - All features
- `sample/test_array_inference_summary.hr` - Feature demonstration
- `sample/test_array_inference_validation.hr` - Advanced usage

## Full Documentation

See `ARRAY_LITERAL_INFERENCE.md` for complete documentation.
