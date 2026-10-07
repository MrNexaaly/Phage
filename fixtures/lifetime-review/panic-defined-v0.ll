target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"
define i1 @_RNvNtCs123_4core9panicking5panic() {
entry:
 ret i1 true
}
define i1 @phage_target() {
entry:
 %ok = call i1 @_RNvNtCs123_4core9panicking5panic()
 ret i1 %ok
}
