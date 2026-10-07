target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)
define i1 @phage_target() {
entry:
 %p = alloca i16, align 2
 %q = getelementptr i8, ptr %p, i64 1
 call void @llvm.memcpy.p0.p0.i64(ptr noundef align 2 %q, ptr noundef align 2 %p, i64 0, i1 false)
 ret i1 true
}
