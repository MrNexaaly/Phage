target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
define i1 @phage_target(i64 %input) {
entry:
 br label %loop
loop:
 %x = phi i64 [ %input, %entry ], [ %next, %loop ]
 %slot = alloca i64
 store i64 %x, ptr %slot
 %next = add i64 %x, 1
 br label %loop
}
