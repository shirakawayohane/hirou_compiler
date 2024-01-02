; ModuleID = 'main'
source_filename = "main"

@string_literal = private unnamed_addr constant [14 x i8] c"Hello, world!\00", align 1

declare i32 @printf(ptr, ...)

define void @main() {
entry:
  %0 = call i32 (ptr, ...) @printf(ptr @string_literal)
  ret void
}
