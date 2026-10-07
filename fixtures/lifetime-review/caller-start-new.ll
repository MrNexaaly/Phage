target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.lifetime.start.p0(ptr)
declare void @llvm.lifetime.end.p0(ptr)
define void @start(ptr %p) {
entry:
 call void @llvm.lifetime.start.p0(ptr %p)
 ret void
}
define i1 @phage_target() {
entry:
 %a = alloca i8
 call void @start(ptr %a)
 store i8 7, ptr %a
 %v = load i8, ptr %a
 %ok = icmp eq i8 %v, 7
 ret i1 %ok
}
