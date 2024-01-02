; ModuleID = 'main'
source_filename = "main"

%"Vec<Vector2>" = type { i32, i32, ptr }
%Vector2 = type { i32, i32 }

define void @main() {
entry:
  %0 = alloca %"Vec<Vector2>", align 8
  %1 = alloca %"Vec<Vector2>", align 8
  call void @"vec()->Vec<Vector2>"(ptr %1)
  %2 = load %"Vec<Vector2>", ptr %1, align 8
  store %"Vec<Vector2>" %2, ptr %0, align 8
  %3 = load %"Vec<Vector2>", ptr %0, align 8
  call void @"set(Vec<Vector2>,usize,Vector2)->void"(%"Vec<Vector2>" %3, i64 0, %Vector2 { i32 15, i32 25 })
  ret void
}

define void @"set(Vec<Vector2>,usize,Vector2)->void"(%"Vec<Vector2>" %vec, i64 %index, %Vector2 %value) {
entry:
  %vec1 = alloca %"Vec<Vector2>", align 8
  store %"Vec<Vector2>" %vec, ptr %vec1, align 8
  %index2 = alloca i64, align 8
  store i64 %index, ptr %index2, align 4
  %value3 = alloca %Vector2, align 8
  store %Vector2 %value, ptr %value3, align 4
  %0 = alloca ptr, align 8
  %1 = alloca ptr, align 8
  %2 = load %"Vec<Vector2>", ptr %vec1, align 8
  store %"Vec<Vector2>" %2, ptr %1, align 8
  %3 = getelementptr inbounds %"Vec<Vector2>", ptr %1, i32 0, i32 2
  %4 = load ptr, ptr %3, align 8
  store ptr %4, ptr %0, align 8
  %5 = load %Vector2, ptr %value3, align 4
  %6 = load i64, ptr %index2, align 4
  %7 = load ptr, ptr %0, align 8
  %8 = getelementptr inbounds %Vector2, ptr %7, i64 %6
  store %Vector2 %5, ptr %8, align 4
  ret void
}

declare ptr @malloc(i64)

define void @"vec()->Vec<Vector2>"(ptr noalias sret(%"Vec<Vector2>") %0) {
entry:
  %1 = call ptr @"g_malloc(usize)->[Vector2]"(i64 4)
  %2 = getelementptr inbounds %"Vec<Vector2>", ptr %0, i32 0, i32 0
  %3 = load i32, ptr %2, align 4
  store i32 %3, ptr %2, align 4
  %4 = getelementptr inbounds %"Vec<Vector2>", ptr %0, i32 0, i32 1
  %5 = load i32, ptr %4, align 4
  store i32 %5, ptr %4, align 4
  %6 = getelementptr inbounds %"Vec<Vector2>", ptr %0, i32 0, i32 2
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %6, align 8
  ret void
}

define ptr @"g_malloc(usize)->[Vector2]"(i64 %size) {
entry:
  %size1 = alloca i64, align 8
  store i64 %size, ptr %size1, align 4
  %0 = alloca i32, align 4
  %1 = load i64, ptr %size1, align 4
  %2 = mul i64 %1, ptrtoint (ptr getelementptr (%Vector2, ptr null, i32 1) to i64)
  store i64 %2, ptr %0, align 4
  %3 = alloca ptr, align 8
  %4 = load i64, ptr %0, align 4
  %5 = call ptr @malloc(i64 %4)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = alloca ptr, align 8
  store ptr %6, ptr %7, align 8
  ret ptr %6
}
