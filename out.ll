; ModuleID = 'main'
source_filename = "main"

%Vector2 = type { i32, i32 }
%"Vec<Vector2>" = type { i32, i32, ptr }

@string_literal = private unnamed_addr constant [8 x i8] c"%d, %d\0A\00", align 1

declare ptr @malloc(i64)

define ptr @"g_malloc(usize)->[Vector2]"(i64 %size) {
entry:
  %size1 = alloca i64, align 8
  store i64 %size, ptr %size1, align 4
  %0 = load i64, ptr %size1, align 4
  %1 = mul i64 %0, ptrtoint (ptr getelementptr (%Vector2, ptr null, i32 1) to i64)
  %2 = alloca i32, align 4
  store i64 %1, ptr %2, align 4
  %3 = load i64, ptr %2, align 4
  %4 = call ptr @malloc(i64 %3)
  %5 = alloca ptr, align 8
  store ptr %4, ptr %5, align 8
  %6 = load ptr, ptr %5, align 8
  %7 = alloca ptr, align 8
  store ptr %6, ptr %7, align 8
  ret ptr %6
}

define void @"vec()->Vec<Vector2>"(ptr noalias sret(%"Vec<Vector2>") %sret_ptr) {
entry:
  %0 = alloca %"Vec<Vector2>", align 8
  %1 = getelementptr inbounds %"Vec<Vector2>", ptr %0, i32 0, i32 0
  store i32 100, ptr %1, align 4
  %2 = getelementptr inbounds %"Vec<Vector2>", ptr %0, i32 0, i32 1
  store i32 50, ptr %2, align 4
  %3 = call ptr @"g_malloc(usize)->[Vector2]"(i64 4)
  %4 = getelementptr inbounds %"Vec<Vector2>", ptr %0, i32 0, i32 2
  store ptr %3, ptr %4, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %sret_ptr, ptr align 8 %0, i64 ptrtoint (ptr getelementptr (%"Vec<Vector2>", ptr null, i32 1) to i64), i1 false)
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
  %0 = getelementptr inbounds %"Vec<Vector2>", ptr %vec1, i32 0, i32 2
  %1 = load ptr, ptr %0, align 8
  %2 = alloca ptr, align 8
  store ptr %1, ptr %2, align 8
  %3 = load i64, ptr %index2, align 4
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds ptr, ptr %4, i64 %3
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %5, ptr align 8 %value3, i64 ptrtoint (ptr getelementptr (ptr, ptr null, i32 1) to i64), i1 false)
  ret void
}

define void @"get(Vec<Vector2>,usize)->Vector2"(ptr noalias sret(%Vector2) %sret_ptr, %"Vec<Vector2>" %vec, i64 %index) {
entry:
  %vec1 = alloca %"Vec<Vector2>", align 8
  store %"Vec<Vector2>" %vec, ptr %vec1, align 8
  %index2 = alloca i64, align 8
  store i64 %index, ptr %index2, align 4
  %0 = getelementptr inbounds %"Vec<Vector2>", ptr %vec1, i32 0, i32 2
  %1 = load ptr, ptr %0, align 8
  %2 = alloca ptr, align 8
  store ptr %1, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load i64, ptr %index2, align 4
  %5 = getelementptr inbounds %Vector2, ptr %3, i64 %4
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %sret_ptr, ptr align 8 %5, i64 ptrtoint (ptr getelementptr (%Vector2, ptr null, i32 1) to i64), i1 false)
  ret void
}

define void @main() {
entry:
  call void @test_struct_vector()
  ret void
}

declare i32 @printf(ptr, ...)

define void @test_struct_vector() {
entry:
  %0 = alloca %Vector2, align 8
  %1 = getelementptr inbounds %Vector2, ptr %0, i32 0, i32 0
  store i32 10, ptr %1, align 4
  %2 = getelementptr inbounds %Vector2, ptr %0, i32 0, i32 1
  store i32 20, ptr %2, align 4
  %3 = alloca %Vector2, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %3, ptr align 8 %0, i64 ptrtoint (ptr getelementptr (%Vector2, ptr null, i32 1) to i64), i1 false)
  %4 = alloca %"Vec<Vector2>", align 8
  call void @"vec()->Vec<Vector2>"(ptr %4)
  %5 = alloca %"Vec<Vector2>", align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %5, ptr align 8 %4, i64 ptrtoint (ptr getelementptr (%"Vec<Vector2>", ptr null, i32 1) to i64), i1 false)
  %6 = load %"Vec<Vector2>", ptr %5, align 8
  %7 = load %Vector2, ptr %3, align 4
  call void @"set(Vec<Vector2>,usize,Vector2)->void"(%"Vec<Vector2>" %6, i64 0, %Vector2 %7)
  %8 = load %"Vec<Vector2>", ptr %5, align 8
  %9 = alloca %Vector2, align 8
  call void @"get(Vec<Vector2>,usize)->Vector2"(ptr %9, %"Vec<Vector2>" %8, i64 0)
  %10 = alloca %Vector2, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %10, ptr align 8 %9, i64 ptrtoint (ptr getelementptr (%Vector2, ptr null, i32 1) to i64), i1 false)
  %11 = getelementptr inbounds %Vector2, ptr %10, i32 0, i32 0
  %12 = load i32, ptr %11, align 4
  %13 = getelementptr inbounds %Vector2, ptr %10, i32 0, i32 1
  %14 = load i32, ptr %13, align 4
  %15 = call i32 (ptr, ...) @printf(ptr @string_literal, i32 %12, i32 %14)
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #0

attributes #0 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
