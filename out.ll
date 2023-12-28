; ModuleID = 'main'
source_filename = "main"

%"Vec<i32>" = type { i32, i32, ptr }

@string_literal = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@string_literal.1 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

define ptr @"g_malloc(usize)->[i32]"(i64 %size) {
entry:
  %size1 = alloca i64, align 8
  store i64 %size, ptr %size1, align 4
  %0 = alloca i32, align 4
  %1 = load i64, ptr %size1, align 4
  %2 = mul i64 %1, ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64)
  store i64 %2, ptr %0, align 4
  %3 = alloca ptr, align 8
  %4 = load i64, ptr %0, align 4
  %5 = call ptr @malloc(i64 %4)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  ret ptr %6
}

declare ptr @malloc(i64)

define i32 @"get(Vec<i32>,usize)->i32"(%"Vec<i32>" %vec, i64 %index) {
entry:
  %vec1 = alloca %"Vec<i32>", align 8
  store %"Vec<i32>" %vec, ptr %vec1, align 8
  %index2 = alloca i64, align 8
  store i64 %index, ptr %index2, align 4
  %0 = alloca ptr, align 8
  %1 = alloca ptr, align 8
  %2 = load %"Vec<i32>", ptr %vec1, align 8
  store %"Vec<i32>" %2, ptr %1, align 8
  %3 = getelementptr inbounds %"Vec<i32>", ptr %1, i32 0, i32 2
  %4 = load ptr, ptr %3, align 8
  store ptr %4, ptr %0, align 8
  %5 = load ptr, ptr %0, align 8
  %6 = load i64, ptr %index2, align 4
  %7 = getelementptr inbounds i32, ptr %5, i64 %6
  %8 = load i32, ptr %7, align 4
  ret i32 %8
}

declare i32 @printf(ptr, ...)

define void @"set(Vec<i32>,usize,i32)->void"(%"Vec<i32>" %vec, i64 %index, i32 %value) {
entry:
  %vec1 = alloca %"Vec<i32>", align 8
  store %"Vec<i32>" %vec, ptr %vec1, align 8
  %index2 = alloca i64, align 8
  store i64 %index, ptr %index2, align 4
  %value3 = alloca i32, align 4
  store i32 %value, ptr %value3, align 4
  %0 = alloca ptr, align 8
  %1 = alloca ptr, align 8
  %2 = load %"Vec<i32>", ptr %vec1, align 8
  store %"Vec<i32>" %2, ptr %1, align 8
  %3 = getelementptr inbounds %"Vec<i32>", ptr %1, i32 0, i32 2
  %4 = load ptr, ptr %3, align 8
  store ptr %4, ptr %0, align 8
  %5 = load i32, ptr %value3, align 4
  %6 = load i64, ptr %index2, align 4
  %7 = load ptr, ptr %0, align 8
  %8 = getelementptr inbounds i32, ptr %7, i64 %6
  store i32 %5, ptr %8, align 4
  ret void
}

define i32 @add(i32 %l, i32 %r) {
entry:
  %l1 = alloca i32, align 4
  store i32 %l, ptr %l1, align 4
  %r2 = alloca i32, align 4
  store i32 %r, ptr %r2, align 4
  %0 = load i32, ptr %l1, align 4
  %1 = load i32, ptr %r2, align 4
  %2 = add i32 %0, %1
  ret i32 %2
}

define void @"vec()->Vec<i32>"(ptr noalias sret(%"Vec<i32>") %0) {
entry:
  %1 = call ptr @"g_malloc(usize)->[i32]"(i64 4)
  store %"Vec<i32>" { i32 100, i32 50, ptr %1 }, ptr %0, align 8
  ret void
}

define i32 @main() {
entry:
  %0 = alloca %"Vec<i32>", align 8
  %1 = alloca %"Vec<i32>", align 8
  call void @"vec()->Vec<i32>"(ptr %1)
  %2 = load %"Vec<i32>", ptr %1, align 8
  store %"Vec<i32>" %2, ptr %0, align 8
  %3 = load %"Vec<i32>", ptr %0, align 8
  call void @"set(Vec<i32>,usize,i32)->void"(%"Vec<i32>" %3, i64 1, i32 123)
  %4 = alloca i32, align 4
  %5 = load %"Vec<i32>", ptr %0, align 8
  %6 = call i32 @"get(Vec<i32>,usize)->i32"(%"Vec<i32>" %5, i64 1)
  store i32 %6, ptr %4, align 4
  %7 = load i32, ptr %4, align 4
  %8 = call i32 (ptr, ...) @printf(ptr @string_literal, i32 %7)
  %9 = call i32 @add(i32 1, i32 2)
  %10 = call i32 (ptr, ...) @printf(ptr @string_literal.1, i32 %9)
  ret i32 0
}
