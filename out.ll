; ModuleID = 'main'
source_filename = "main"

@string_literal = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare ptr @malloc(i64)

declare i32 @printf(ptr, ...)

define void @printi32(i32 %n) {
entry:
  %n1 = alloca i32, align 4
  store i32 %n, ptr %n1, align 4
  %0 = load i32, ptr %n1, align 4
  %1 = call i32 (ptr, ...) @printf(ptr @string_literal, i32 %0)
  ret void
}

define i32 @main() {
entry:
  %0 = alloca i32, align 4
  store i32 0, ptr %0, align 4
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  %2 = alloca i64, align 8
  store i64 4, ptr %2, align 4
  %3 = alloca ptr, align 8
  %4 = load i64, ptr %2, align 4
  %5 = call ptr @malloc(i64 %4)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds i32, ptr %6, i64 0
  store i32 10, ptr %7, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds i32, ptr %8, i64 1
  store i32 20, ptr %9, align 4
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %10, align 4
  call void @printi32(i32 %11)
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds i32, ptr %12, i64 1
  %14 = load i32, ptr %13, align 4
  call void @printi32(i32 %14)
  ret i32 0
}
