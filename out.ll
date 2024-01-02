; ModuleID = 'main'
source_filename = "main"

%"Vec<i32>" = type { i32, i32, ptr }

@string_literal = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare ptr @malloc(i64)

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

define i32 @main() {
entry:
  %0 = alloca %"Vec<i32>", align 8
  %1 = alloca %"Vec<i32>", align 8
  call void @"vec()->Vec<i32>"(ptr %1)
  %2 = load %"Vec<i32>", ptr %1, align 8
  store %"Vec<i32>" %2, ptr %0, align 8
  %3 = alloca i32, align 4
  %4 = load %"Vec<i32>", ptr %0, align 8
  store %"Vec<i32>" %4, ptr %3, align 8
  %5 = getelementptr inbounds %"Vec<i32>", ptr %3, i32 0, i32 1
  %6 = load i32, ptr %5, align 4
  %7 = call i32 (ptr, ...) @printf(ptr @string_literal, i32 %6)
  %8 = load %"Vec<i32>", ptr %0, align 8
  call void @"set(Vec<i32>,usize,i32)->void"(%"Vec<i32>" %8, i64 1, i32 10)
  ret i32 0
}

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

define void @"vec()->Vec<i32>"(ptr noalias sret(%"Vec<i32>") %0) {
entry:
  %1 = call ptr @"g_malloc(usize)->[i32]"(i64 4)
  store %"Vec<i32>" { i32 100, i32 50, ptr %1 }, ptr %0, align 8
  ret void
}

declare i32 @printf(ptr, ...)
