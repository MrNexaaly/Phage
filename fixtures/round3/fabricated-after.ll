target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
declare ptr @__rust_alloc(i64, i64)
define i1 @phage_target(i8 %raw) {
entry:
 %lo = icmp ugt i8 %raw, 0
 %hi = icmp ule i8 %raw, 64
 %domain = and i1 %lo, %hi
 br i1 %domain, label %test, label %excluded
excluded:
 ret i1 true
test:
 %n = zext i8 %raw to i64
 %p = call ptr @__rust_alloc(i64 %n, i64 1)
 %d = inttoptr i64 65536 to ptr
 %base = ptrtoint ptr %p to i64
 %end = add i64 %base, %n
 %left = icmp ule i64 %end, 65536
 %right = icmp uge i64 %base, 65536
 %ok = or i1 %left, %right
 ret i1 %ok
}
