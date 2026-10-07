target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.lifetime.start.p0(ptr)
declare void @llvm.lifetime.end.p0(ptr)
define void @observe(i8 noundef %v) {
entry:
 ret void
}
define i1 @phage_target() {
entry:
 %a = alloca i8
 call void @llvm.lifetime.start.p0(ptr %a)
 call void @llvm.lifetime.end.p0(ptr %a)
 %v = load i8, ptr %a
 call void @observe(i8 noundef %v)
 ret i1 true
}
