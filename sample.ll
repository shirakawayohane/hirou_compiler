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

declare i8* @malloc(i64)

define i32 @main(...) {
entry:
  %buf = alloca i8*, align 8
  %function_call = call i8* @malloc(i64 2)
  store i8* %function_call, i8** %buf, align 8
  %buf1 = load i8*, i8** %buf, align 8
  call void @print-u8-ptr(i8* %buf1)
  ret i32 0
}
