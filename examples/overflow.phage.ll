; ModuleID = 'overflow.f1d5e78198cb53b8-cgu.0'
source_filename = "overflow.f1d5e78198cb53b8-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_b4f57ecc86d33306283012fbcae4eccd = private unnamed_addr constant [21 x i8] c"examples/overflow.rs\00", align 1
@alloc_8879c53042566e43bed2b426d80dcb6b = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_b4f57ecc86d33306283012fbcae4eccd, [16 x i8] c"\14\00\00\00\00\00\00\00\04\00\00\00\10\00\00\00" }>, align 8

; Function Attrs: nounwind nonlazybind uwtable
define noundef zeroext i1 @phage_target(i8 noundef %x) unnamed_addr #0 !dbg !7 {
start:
  %_3.1 = icmp eq i8 %x, -1, !dbg !12
  br i1 %_3.1, label %panic, label %bb1, !dbg !12

bb1:                                              ; preds = %start
  ret i1 true, !dbg !13

panic:                                            ; preds = %start
; call core::panicking::panic_const::panic_const_add_overflow
  tail call void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h26fd19dc5797cca9E(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24) @alloc_8879c53042566e43bed2b426d80dcb6b) #2, !dbg !12
  unreachable, !dbg !12
}

; core::panicking::panic_const::panic_const_add_overflow
; Function Attrs: cold noinline noreturn nounwind nonlazybind uwtable
declare void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h26fd19dc5797cca9E(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24)) unnamed_addr #1

attributes #0 = { nounwind nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #1 = { cold noinline noreturn nounwind nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #2 = { noinline noreturn nounwind }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}
!llvm.dbg.cu = !{!5}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{i32 7, !"Dwarf Version", i32 4}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{!"rustc version 1.94.1 (e408947bf 2026-03-25)"}
!5 = distinct !DICompileUnit(language: DW_LANG_Rust, file: !6, producer: "clang LLVM (rustc version 1.94.1 (e408947bf 2026-03-25))", isOptimized: true, runtimeVersion: 0, emissionKind: LineTablesOnly, splitDebugInlining: false, nameTableKind: None)
!6 = !DIFile(filename: "examples/overflow.rs/@/overflow.f1d5e78198cb53b8-cgu.0", directory: "/home/zero/Dev/Nexaaly/RuHealth")
!7 = distinct !DISubprogram(name: "phage_target", scope: !9, file: !8, line: 3, type: !10, scopeLine: 3, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!8 = !DIFile(filename: "examples/overflow.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "056b6c4604e7f56e91802f63514ec07b")
!9 = !DINamespace(name: "overflow", scope: null)
!10 = !DISubroutineType(types: !11)
!11 = !{}
!12 = !DILocation(line: 4, column: 16, scope: !7)
!13 = !DILocation(line: 6, column: 2, scope: !7)
