; ModuleID = 'main'
source_filename = "main"

declare i32 @main()

declare i32 @printf(i8*, ...)

declare i32* @"malloc(usize,i32,)->[i32]"(i64, i32)

declare void @print-i32(i8*, i32)
