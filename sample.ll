; ModuleID = 'main'
source_filename = "main"

@digit_format_string = private unnamed_addr constant [3 x i8] c"%d\00", align 1

declare i32 @printf(i8*, ...)

define void @printi32(i32 %0) {
entry:
  %call = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @digit_format_string, i32 0, i32 0), i32 %0)
  ret void
}

define i32 @test(i32 %j, ...) {
entry:
  %j1 = alloca i32, align 4
  store i32 %j, i32* %j1, align 4
  %i = alloca i32, align 4
  %j2 = load i32, i32* %j1, align 4
  %mul_int_int = mul i32 %j2, 10
  store i32 %mul_int_int, i32* %i, align 4
  %i3 = load i32, i32* %i, align 4
  ret i32 %i3
}

define i32 @main(...) {
entry:
  %i = alloca i32, align 4
  store i32 10, i32* %i, align 4
  %v = alloca i32, align 4
  %i1 = load i32, i32* %i, align 4
  %function_call = call i32 (i32, ...) @test(i32 %i1)
  store i32 %function_call, i32* %v, align 4
  %v2 = load i32, i32* %v, align 4
  call void @printi32(i32 %v2)
  ret i32 0
}
