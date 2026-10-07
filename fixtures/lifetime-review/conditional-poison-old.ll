target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.lifetime.start.p0(i64, ptr)
declare void @llvm.lifetime.end.p0(i64, ptr)
define i1 @phage_target(i1 %pick) {
entry:
 %a = alloca i8
 %p = select i1 %pick, ptr poison, ptr %a
 call void @llvm.lifetime.start.p0(i64 1, ptr %p)
 ret i1 true
}
