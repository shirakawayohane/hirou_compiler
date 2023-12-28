; ModuleID = 'main'
source_filename = "main"

%"Vec<i32>" = type { i64, i64, ptr }

declare ptr @malloc(i64)

define void @"vec()->Vec<i32>"(ptr noalias sret(%"Vec<i32>") %0) {
entry:
  %1 = alloca i64, align 8
  store i64 4, ptr %1, align 4
  %2 = load i64, ptr %1, align 4
  %3 = load i64, ptr %1, align 4
  %4 = call ptr @"generic_malloc(usize)->[i32]"(i64 %3)
  store %"Vec<i32>" { i64 0, i64 %2, ptr %4 }, ptr %0, align 8
  ret void
}

define ptr @"generic_malloc(usize)->[i32]"(i64 %size) {
entry:
  %size1 = alloca i64, align 8
  store i64 %size, ptr %size1, align 4
  %0 = alloca i32, align 4
  %1 = load i64, ptr %0, align 4
  %2 = mul i64 %1, ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64)
  store i64 %2, ptr %0, align 4
  %3 = alloca ptr, align 8
  %4 = load i64, ptr %0, align 4
  %5 = call ptr @malloc(i64 %4)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  ret ptr %6
}

define i32 @main() {
entry:
  call void @"vec()->Vec<i32>"()
  ret i32 0
}
