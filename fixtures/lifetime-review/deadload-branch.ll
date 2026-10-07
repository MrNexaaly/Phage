target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.lifetime.start.p0(ptr)
declare void @llvm.lifetime.end.p0(ptr)
define i1 @phage_target() {
start:
 %a = alloca i8
 call void @llvm.lifetime.start.p0(ptr %a)
 call void @llvm.lifetime.end.p0(ptr %a)
 %v = load i8, ptr %a
 %ok = icmp eq i8 %v, 7
 br i1 %ok, label %yes, label %no
yes:
 ret i1 true
no:
 ret i1 true
}
