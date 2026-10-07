target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare ptr @__rust_alloc(i64, i64)
declare void @__rust_dealloc(ptr, i64, i64)
declare void @llvm.lifetime.start.p0(i64, ptr)
define i1 @phage_target() {
start:
 %p = call ptr @__rust_alloc(i64 1, i64 1)
 call void @__rust_dealloc(ptr %p, i64 1, i64 1)
 call void @llvm.lifetime.start.p0(i64 -1, ptr %p)
 store i8 7, ptr %p
 ret i1 true
}
