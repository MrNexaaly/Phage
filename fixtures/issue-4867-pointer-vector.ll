; Restricted-subset control inspired by Kani #4867 (closed upstream).
; Phage does not support pointer SIMD and must report unknown, not crash.
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
define i1 @phage_target(<2 x ptr> %v) {
start:
  ret i1 true
}
