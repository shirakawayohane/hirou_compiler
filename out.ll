; ModuleID = 'main'
source_filename = "main"

@digit_format_string = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(i8*, ...)

define void @instrinsic_print_u8(i8 %0) {
entry:
  %call = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @digit_format_string, i32 0, i32 0), i8 %0)
  ret void
}

declare i8* @malloc(i64)

define i8* @__malloc(i64 %0) {
entry:
  %call = call i8* @malloc(i64 %0)
  ret i8* %call
}
