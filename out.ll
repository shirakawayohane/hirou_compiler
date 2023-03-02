; ModuleID = 'main'
source_filename = "main"

@digit_format_string = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@digit_format_string.1 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@digit_format_string.2 = private unnamed_addr constant [5 x i8] c"%zu\0A\00", align 1
@digit_format_string.3 = private unnamed_addr constant [5 x i8] c"%zu\0A\00", align 1

declare i32 @printf(i8*, ...)

define void @print-u8(i8 %0) {
entry:
  %call = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @digit_format_string, i32 0, i32 0), i8 %0)
  ret void
}

define void @print-i32(i32 %0) {
entry:
  %call = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @digit_format_string.1, i32 0, i32 0), i32 %0)
  ret void
}

define void @print-u64(i64 %0) {
entry:
  %call = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @digit_format_string.2, i32 0, i32 0), i64 %0)
  ret void
}

define void @print-u8-ptr(i8* %0) {
entry:
  %call = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @digit_format_string.3, i32 0, i32 0), i8* %0)
  ret void
}

declare i8* @malloc(i32)

define i8* @__malloc(i32 %0) {
entry:
  %call = call i8* @malloc(i32 %0)
  ret i8* %call
}

define i32 @main(...) {
entry:
  %size = alloca i32, align 4
  store i32 4, i32* %size, align 4
  %buf = alloca i32*, align 8
  %load = load i32, i32* %size, align 4
  %function_call = call i8* @__malloc(i32 %load)
  store i8* %function_call, i32** %buf, align 8
  %deref = load i32*, i32** %buf, align 8
  %array_indexing = getelementptr i32, i32* %deref, i64 0
  store i32 1, i32* %array_indexing, align 4
  %deref1 = load i32*, i32** %buf, align 8
  %array_indexing2 = getelementptr i32, i32* %deref1, i64 1
  store i32 2, i32* %array_indexing2, align 4
  %deref3 = load i32*, i32** %buf, align 8
  %array_indexing4 = getelementptr i32, i32* %deref3, i64 2
  store i32 3, i32* %array_indexing4, align 4
  %deref5 = load i32*, i32** %buf, align 8
  %array_indexing6 = getelementptr i32, i32* %deref5, i64 3
  store i32 4, i32* %array_indexing6, align 4
  %load7 = load i32*, i32** %buf, align 8
  %load8 = load i32, i32* %load7, align 4
  call void @print-i32(i32 %load8)
  %load9 = load i32*, i32** %buf, align 8
  %index_access = getelementptr i32, i32* %load9, i64 1
  %load10 = load i32, i32* %index_access, align 4
  call void @print-i32(i32 %load10)
  %load11 = load i32*, i32** %buf, align 8
  %index_access12 = getelementptr i32, i32* %load11, i64 2
  %load13 = load i32, i32* %index_access12, align 4
  call void @print-i32(i32 %load13)
  %load14 = load i32*, i32** %buf, align 8
  %index_access15 = getelementptr i32, i32* %load14, i64 3
  %load16 = load i32, i32* %index_access15, align 4
  call void @print-i32(i32 %load16)
  ret i32 0
}
