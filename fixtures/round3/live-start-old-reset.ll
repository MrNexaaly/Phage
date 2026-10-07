target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.lifetime.start.p0(i64, ptr)
define i1 @phage_target() {
entry:
 %p = alloca i8
 call void @llvm.lifetime.start.p0(i64 1, ptr %p)
 store i8 7, ptr %p
 call void @llvm.lifetime.start.p0(i64 1, ptr %p)
 %v = load i8, ptr %p
 %ok = icmp eq i8 %v, 7
 ret i1 %ok
}
