; ModuleID = 'unsupported.8383d3de02cacdc6-cgu.0'
source_filename = "unsupported.8383d3de02cacdc6-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

; Function Attrs: nounwind nonlazybind uwtable
define noundef zeroext i1 @phage_target(i8 noundef %x) unnamed_addr #0 !dbg !7 {
start:
  %_0 = tail call noundef zeroext i1 @external_result(i8 noundef zeroext %x) #1, !dbg !12
  ret i1 %_0, !dbg !13
}

; Function Attrs: nounwind nonlazybind uwtable
declare noundef zeroext i1 @external_result(i8 noundef zeroext) unnamed_addr #0

attributes #0 = { nounwind nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #1 = { nounwind }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}
!llvm.dbg.cu = !{!5}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{i32 7, !"Dwarf Version", i32 4}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{!"rustc version 1.94.1 (e408947bf 2026-03-25)"}
!5 = distinct !DICompileUnit(language: DW_LANG_Rust, file: !6, producer: "clang LLVM (rustc version 1.94.1 (e408947bf 2026-03-25))", isOptimized: true, runtimeVersion: 0, emissionKind: LineTablesOnly, splitDebugInlining: false, nameTableKind: None)
!6 = !DIFile(filename: "examples/unsupported.rs/@/unsupported.8383d3de02cacdc6-cgu.0", directory: "/home/zero/Dev/Nexaaly/RuHealth")
!7 = distinct !DISubprogram(name: "phage_target", scope: !9, file: !8, line: 6, type: !10, scopeLine: 6, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!8 = !DIFile(filename: "examples/unsupported.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "addb507fdc023712b2b3504da96bcf42")
!9 = !DINamespace(name: "unsupported", scope: null)
!10 = !DISubroutineType(types: !11)
!11 = !{}
!12 = !DILocation(line: 8, column: 14, scope: !7)
!13 = !DILocation(line: 9, column: 2, scope: !7)
