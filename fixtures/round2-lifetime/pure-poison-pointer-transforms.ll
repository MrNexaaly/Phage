target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
define i1 @phage_target() {
entry:
 %p = inttoptr i32 poison to ptr
 %q = bitcast ptr %p to ptr
 %r = getelementptr inbounds i8, ptr %q, i64 1
 %i = ptrtoint ptr %r to i64
 %j = add i64 %i, 1
 ret i1 true
}
