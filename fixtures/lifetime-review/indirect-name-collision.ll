target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.lifetime.start.p0(i64, ptr)
define void @f(ptr %p) {
entry:
 call void @llvm.lifetime.start.p0(i64 1, ptr %p)
 ret void
}
define void @present(ptr %p) {
entry:
 ret void
}
define i1 @phage_target() {
entry:
 %a = alloca i8
 store i8 7, ptr %a
 %f = select i1 true, ptr @present, ptr @present
 call void %f(ptr %a)
 ret i1 true
}
