target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
define i1 @phage_target() {
entry:
 %d = inttoptr i64 65536 to ptr
 store i8 1, ptr %d
 ret i1 true
}
