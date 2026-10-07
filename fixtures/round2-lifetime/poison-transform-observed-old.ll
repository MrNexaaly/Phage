target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.lifetime.start.p0(i64, ptr)
declare void @llvm.lifetime.end.p0(i64, ptr)
define i1 @phage_target() {
entry:
 %a = alloca i64, align 8
 call void @llvm.lifetime.start.p0(i64 8, ptr %a)
 call void @llvm.lifetime.end.p0(i64 8, ptr %a)
 %v = load i64, ptr %a, align 8
 %p = inttoptr i64 %v to ptr
 %i = ptrtoint ptr %p to i64
 %j = add i64 %i, 1
 %x = load i8, ptr %p
 ret i1 true
}
