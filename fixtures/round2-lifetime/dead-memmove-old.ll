target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.lifetime.start.p0(i64, ptr)
declare void @llvm.lifetime.end.p0(i64, ptr)
declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1)
define i1 @phage_target() {
entry:
 %a = alloca i64, align 8
 %b = alloca i64, align 8
 call void @llvm.lifetime.start.p0(i64 8, ptr %a)
 call void @llvm.lifetime.end.p0(i64 8, ptr %a)
 call void @llvm.memmove.p0.p0.i64(ptr %b, ptr %a, i64 1, i1 false)
 ret i1 true
}
