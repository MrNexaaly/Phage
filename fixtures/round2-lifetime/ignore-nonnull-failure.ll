target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare void @llvm.assume(i1)
define i1 @phage_target() {
entry:
 call void @llvm.assume(i1 true) ["ignore"(i8 poison), "nonnull"(ptr null)]
 ret i1 true
}
