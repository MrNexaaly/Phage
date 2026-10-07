; ModuleID = 'safe_add.e454b8243aabf92a-cgu.0'
source_filename = "safe_add.e454b8243aabf92a-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(none) uwtable
define noundef zeroext i1 @phage_target(i8 noundef %x) unnamed_addr #0 !dbg !7 {
start:
  ret i1 true, !dbg !12
}

attributes #0 = { mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(none) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}
!llvm.dbg.cu = !{!5}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{i32 7, !"Dwarf Version", i32 4}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{!"rustc version 1.94.1 (e408947bf 2026-03-25)"}
!5 = distinct !DICompileUnit(language: DW_LANG_Rust, file: !6, producer: "clang LLVM (rustc version 1.94.1 (e408947bf 2026-03-25))", isOptimized: true, runtimeVersion: 0, emissionKind: LineTablesOnly, splitDebugInlining: false, nameTableKind: None)
!6 = !DIFile(filename: "examples/safe-add.rs/@/safe_add.e454b8243aabf92a-cgu.0", directory: "/home/zero/Dev/Nexaaly/RuHealth")
!7 = distinct !DISubprogram(name: "phage_target", scope: !9, file: !8, line: 3, type: !10, scopeLine: 3, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!8 = !DIFile(filename: "examples/safe-add.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "ab62639b99f315e6f1904d72b669f292")
!9 = !DINamespace(name: "safe_add", scope: null)
!10 = !DISubroutineType(types: !11)
!11 = !{}
!12 = !DILocation(line: 6, column: 2, scope: !7)
