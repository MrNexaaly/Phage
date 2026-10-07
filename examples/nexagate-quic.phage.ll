; ModuleID = 'nexagate_quic.6c36603968ae4683-cgu.0'
source_filename = "nexagate_quic.6c36603968ae4683-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

; Function Attrs: nofree norecurse nosync nounwind nonlazybind memory(none) uwtable
define noundef zeroext i1 @phage_target(i8 noundef %a, i8 noundef %b, i8 noundef %c, i8 noundef %d, i8 noundef %e, i8 noundef %f, i8 noundef %g, i8 noundef %h, i8 noundef %len) unnamed_addr #0 !dbg !7 {
start:
  %data = alloca [8 x i8], align 1
  %_10 = icmp ugt i8 %len, 8, !dbg !12
  br i1 %_10, label %bb21, label %bb2, !dbg !12

bb2:                                              ; preds = %start
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %data), !dbg !13
  store i8 %a, ptr %data, align 1, !dbg !14
  %0 = getelementptr inbounds nuw i8, ptr %data, i64 1, !dbg !14
  store i8 %b, ptr %0, align 1, !dbg !14
  %1 = getelementptr inbounds nuw i8, ptr %data, i64 2, !dbg !14
  store i8 %c, ptr %1, align 1, !dbg !14
  %2 = getelementptr inbounds nuw i8, ptr %data, i64 3, !dbg !14
  store i8 %d, ptr %2, align 1, !dbg !14
  %3 = getelementptr inbounds nuw i8, ptr %data, i64 4, !dbg !14
  store i8 %e, ptr %3, align 1, !dbg !14
  %4 = getelementptr inbounds nuw i8, ptr %data, i64 5, !dbg !14
  store i8 %f, ptr %4, align 1, !dbg !14
  %5 = getelementptr inbounds nuw i8, ptr %data, i64 6, !dbg !14
  store i8 %g, ptr %5, align 1, !dbg !14
  %6 = getelementptr inbounds nuw i8, ptr %data, i64 7, !dbg !14
  store i8 %h, ptr %6, align 1, !dbg !14
  %_0.i2 = zext nneg i8 %len to i64, !dbg !15
  %_3.not.i.i = icmp eq i8 %len, 0, !dbg !24
  br i1 %_3.not.i.i, label %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit, label %bb4.i, !dbg !35

bb4.i:                                            ; preds = %bb2
  %_8.i = lshr i8 %a, 6, !dbg !36
  %7 = zext nneg i8 %_8.i to i64, !dbg !38
  %len.i = shl nuw nsw i64 1, %7, !dbg !38
  %_7.not.i.i.i.i = icmp samesign ugt i64 %len.i, %_0.i2, !dbg !39
  br i1 %_7.not.i.i.i.i, label %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit, label %bb10.i, !dbg !51

bb10.i:                                           ; preds = %bb4.i
  %_5.i.i = getelementptr i8, ptr %data, i64 %len.i, !dbg !52
  %_6.i.not6.i = icmp ult i8 %a, 64, !dbg !73
  br i1 %_6.i.not6.i, label %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit, label %bb18.i.preheader, !dbg !86

bb18.i.preheader:                                 ; preds = %bb10.i
  %_16.i = and i8 %a, 63, !dbg !87
  %_0.i3.i = zext nneg i8 %_16.i to i64, !dbg !88
  %8 = add nuw nsw i64 %len.i, 7, !dbg !86
  %9 = add nsw i64 %len.i, -2, !dbg !86
  %xtraiter = and i64 %8, 7, !dbg !86
  %10 = and i64 %len.i, 7, !dbg !86
  %lcmp.mod.not = icmp eq i64 %10, 1, !dbg !86
  br i1 %lcmp.mod.not, label %bb18.i.prol.loopexit, label %bb18.i.prol, !dbg !86

bb18.i.prol:                                      ; preds = %bb18.i.preheader, %bb18.i.prol
  %value.sroa.0.08.i.prol = phi i64 [ %11, %bb18.i.prol ], [ %_0.i3.i, %bb18.i.preheader ]
  %iter.sroa.0.07.i.prol = phi ptr [ %spec.select.i.prol, %bb18.i.prol ], [ %0, %bb18.i.preheader ]
  %prol.iter = phi i64 [ %prol.iter.next, %bb18.i.prol ], [ 0, %bb18.i.preheader ]
  %spec.select.i.prol = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i.prol, i64 1, !dbg !92
  %b.i.prol = load i8, ptr %iter.sroa.0.07.i.prol, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %_24.i.prol = shl i64 %value.sroa.0.08.i.prol, 8, !dbg !99
  %_0.i.i.prol = zext i8 %b.i.prol to i64, !dbg !101
  %11 = or disjoint i64 %_24.i.prol, %_0.i.i.prol, !dbg !103
  %prol.iter.next = add i64 %prol.iter, 1, !dbg !86
  %prol.iter.cmp.not = icmp eq i64 %prol.iter.next, %xtraiter, !dbg !86
  br i1 %prol.iter.cmp.not, label %bb18.i.prol.loopexit, label %bb18.i.prol, !dbg !86, !llvm.loop !104

bb18.i.prol.loopexit:                             ; preds = %bb18.i.prol, %bb18.i.preheader
  %_24.i.lcssa.unr = phi i64 [ poison, %bb18.i.preheader ], [ %_24.i.prol, %bb18.i.prol ]
  %value.sroa.0.08.i.unr = phi i64 [ %_0.i3.i, %bb18.i.preheader ], [ %11, %bb18.i.prol ]
  %iter.sroa.0.07.i.unr = phi ptr [ %0, %bb18.i.preheader ], [ %spec.select.i.prol, %bb18.i.prol ]
  %12 = icmp ult i64 %9, 7, !dbg !86
  br i1 %12, label %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit.loopexit, label %bb18.i, !dbg !86

bb18.i:                                           ; preds = %bb18.i.prol.loopexit, %bb18.i
  %value.sroa.0.08.i = phi i64 [ %24, %bb18.i ], [ %value.sroa.0.08.i.unr, %bb18.i.prol.loopexit ]
  %iter.sroa.0.07.i = phi ptr [ %spec.select.i.7, %bb18.i ], [ %iter.sroa.0.07.i.unr, %bb18.i.prol.loopexit ]
  %spec.select.i = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i, i64 1, !dbg !92
  %b.i = load i8, ptr %iter.sroa.0.07.i, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %_0.i.i = zext i8 %b.i to i64, !dbg !101
  %spec.select.i.1 = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i, i64 2, !dbg !92
  %b.i.1 = load i8, ptr %spec.select.i, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %13 = shl i64 %value.sroa.0.08.i, 16, !dbg !99
  %14 = shl nuw nsw i64 %_0.i.i, 8, !dbg !99
  %_24.i.1 = or disjoint i64 %13, %14, !dbg !99
  %_0.i.i.1 = zext i8 %b.i.1 to i64, !dbg !101
  %15 = or disjoint i64 %_24.i.1, %_0.i.i.1, !dbg !103
  %spec.select.i.2 = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i, i64 3, !dbg !92
  %b.i.2 = load i8, ptr %spec.select.i.1, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %_0.i.i.2 = zext i8 %b.i.2 to i64, !dbg !101
  %spec.select.i.3 = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i, i64 4, !dbg !92
  %b.i.3 = load i8, ptr %spec.select.i.2, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %16 = shl i64 %15, 16, !dbg !99
  %17 = shl nuw nsw i64 %_0.i.i.2, 8, !dbg !99
  %_24.i.3 = or disjoint i64 %16, %17, !dbg !99
  %_0.i.i.3 = zext i8 %b.i.3 to i64, !dbg !101
  %18 = or disjoint i64 %_24.i.3, %_0.i.i.3, !dbg !103
  %spec.select.i.4 = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i, i64 5, !dbg !92
  %b.i.4 = load i8, ptr %spec.select.i.3, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %_0.i.i.4 = zext i8 %b.i.4 to i64, !dbg !101
  %spec.select.i.5 = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i, i64 6, !dbg !92
  %b.i.5 = load i8, ptr %spec.select.i.4, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %19 = shl i64 %18, 16, !dbg !99
  %20 = shl nuw nsw i64 %_0.i.i.4, 8, !dbg !99
  %_24.i.5 = or disjoint i64 %19, %20, !dbg !99
  %_0.i.i.5 = zext i8 %b.i.5 to i64, !dbg !101
  %21 = or disjoint i64 %_24.i.5, %_0.i.i.5, !dbg !103
  %spec.select.i.6 = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i, i64 7, !dbg !92
  %b.i.6 = load i8, ptr %spec.select.i.5, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %_0.i.i.6 = zext i8 %b.i.6 to i64, !dbg !101
  %spec.select.i.7 = getelementptr inbounds nuw i8, ptr %iter.sroa.0.07.i, i64 8, !dbg !92
  %b.i.7 = load i8, ptr %spec.select.i.6, align 1, !dbg !93, !alias.scope !94, !noalias !97, !noundef !11
  %22 = shl i64 %21, 16, !dbg !99
  %23 = shl nuw nsw i64 %_0.i.i.6, 8, !dbg !99
  %_24.i.7 = or disjoint i64 %22, %23, !dbg !99
  %_0.i.i.7 = zext i8 %b.i.7 to i64, !dbg !101
  %24 = or disjoint i64 %_24.i.7, %_0.i.i.7, !dbg !103
  %_6.i.not.i.7 = icmp eq ptr %spec.select.i.7, %_5.i.i, !dbg !73
  br i1 %_6.i.not.i.7, label %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit.loopexit, label %bb18.i, !dbg !86

_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit.loopexit: ; preds = %bb18.i, %bb18.i.prol.loopexit
  %_24.i.lcssa = phi i64 [ %_24.i.lcssa.unr, %bb18.i.prol.loopexit ], [ %_24.i.7, %bb18.i ], !dbg !99
  %25 = icmp ult i64 %_24.i.lcssa, 4611686018427387904
  br label %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit, !dbg !106

_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit: ; preds = %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit.loopexit, %bb10.i, %bb4.i, %bb2
  %result.sroa.10.0 = phi i64 [ undef, %bb2 ], [ undef, %bb4.i ], [ %len.i, %bb10.i ], [ %len.i, %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit.loopexit ]
  %result.sroa.8.0 = phi i1 [ false, %bb2 ], [ false, %bb4.i ], [ true, %bb10.i ], [ %25, %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit.loopexit ]
  %_2.not.i = phi i1 [ true, %bb2 ], [ true, %bb4.i ], [ false, %bb10.i ], [ false, %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit.loopexit ], !dbg !108
  %result.sroa.0.0 = phi i1 [ false, %bb2 ], [ false, %bb4.i ], [ true, %bb10.i ], [ true, %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit.loopexit ], !dbg !108
  %26 = icmp eq i8 %len, 0, !dbg !106
  br i1 %26, label %bb21.sink.split, label %bb9, !dbg !106

bb21.sink.split:                                  ; preds = %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit, %bb11, %bb12, %bb13
  %_0.sroa.0.0.shrunk.ph = phi i1 [ %28, %bb11 ], [ false, %bb12 ], [ %spec.select, %bb13 ], [ %_2.not.i, %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit ]
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %data), !dbg !109
  br label %bb21, !dbg !110

bb21:                                             ; preds = %bb21.sink.split, %start
  %_0.sroa.0.0.shrunk = phi i1 [ true, %start ], [ %_0.sroa.0.0.shrunk.ph, %bb21.sink.split ]
  ret i1 %_0.sroa.0.0.shrunk, !dbg !110

bb9:                                              ; preds = %_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E.exit
  %_19 = lshr i8 %a, 6, !dbg !111
  %27 = zext nneg i8 %_19 to i64, !dbg !112
  %required = shl nuw nsw i64 1, %27, !dbg !112
  br i1 %result.sroa.0.0, label %bb12, label %bb11, !dbg !113

bb12:                                             ; preds = %bb9
  %_24 = icmp eq i64 %result.sroa.10.0, %required, !dbg !115
  br i1 %_24, label %bb13, label %bb21.sink.split, !dbg !115

bb11:                                             ; preds = %bb9
  %28 = icmp samesign ugt i64 %required, %_0.i2, !dbg !117
  br label %bb21.sink.split, !dbg !118

bb13:                                             ; preds = %bb12
  %_25.not = icmp ule i64 %result.sroa.10.0, %_0.i2, !dbg !119
  %spec.select = select i1 %_25.not, i1 %result.sroa.8.0, i1 false, !dbg !119
  br label %bb21.sink.split, !dbg !119
}

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.start.p0(i64 immarg, ptr captures(none)) #1

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.end.p0(i64 immarg, ptr captures(none)) #1

attributes #0 = { nofree norecurse nosync nounwind nonlazybind memory(none) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #1 = { mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}
!llvm.dbg.cu = !{!5}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{i32 7, !"Dwarf Version", i32 4}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{!"rustc version 1.94.1 (e408947bf 2026-03-25)"}
!5 = distinct !DICompileUnit(language: DW_LANG_Rust, file: !6, producer: "clang LLVM (rustc version 1.94.1 (e408947bf 2026-03-25))", isOptimized: true, runtimeVersion: 0, emissionKind: LineTablesOnly, splitDebugInlining: false, nameTableKind: None)
!6 = !DIFile(filename: "examples/nexagate-quic.rs/@/nexagate_quic.6c36603968ae4683-cgu.0", directory: "/home/zero/Dev/Nexaaly/RuHealth")
!7 = distinct !DISubprogram(name: "phage_target", scope: !9, file: !8, line: 8, type: !10, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!8 = !DIFile(filename: "examples/nexagate-quic.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "75738612ed6d8b05f7594ccb75725e0e")
!9 = !DINamespace(name: "nexagate_quic", scope: null)
!10 = !DISubroutineType(types: !11)
!11 = !{}
!12 = !DILocation(line: 9, column: 8, scope: !7)
!13 = !DILocation(line: 12, column: 9, scope: !7)
!14 = !DILocation(line: 12, column: 16, scope: !7)
!15 = !DILocation(line: 79, column: 17, scope: !16, inlinedAt: !22)
!16 = distinct !DISubprogram(name: "from", linkageName: "_ZN4core7convert3num65_$LT$impl$u20$core..convert..From$LT$u8$GT$$u20$for$u20$usize$GT$4from17h38581d55ae52cc82E", scope: !18, file: !17, line: 78, type: !10, scopeLine: 78, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!17 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/convert/num.rs", directory: "", checksumkind: CSK_MD5, checksum: "87ae55fcbe39b68ccbdcd352f6dbfc81")
!18 = !DINamespace(name: "{impl#68}", scope: !19)
!19 = !DINamespace(name: "num", scope: !20)
!20 = !DINamespace(name: "convert", scope: !21)
!21 = !DINamespace(name: "core", scope: null)
!22 = distinct !DILocation(line: 13, column: 39, scope: !23)
!23 = distinct !DILexicalBlock(scope: !7, file: !8, line: 12, column: 5)
!24 = !DILocation(line: 156, column: 16, scope: !25, inlinedAt: !30)
!25 = distinct !DILexicalBlock(scope: !27, file: !26, line: 156, column: 35)
!26 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "bd4112d6e61cf6a64838143565da7e2d")
!27 = distinct !DISubprogram(name: "first<u8>", linkageName: "_ZN4core5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$5first17h6fd09dd31c7c2da0E", scope: !28, file: !26, line: 155, type: !10, scopeLine: 155, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!28 = !DINamespace(name: "{impl#0}", scope: !29)
!29 = !DINamespace(name: "slice", scope: !21)
!30 = distinct !DILocation(line: 10, column: 24, scope: !31, inlinedAt: !34)
!31 = distinct !DISubprogram(name: "read", linkageName: "_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E", scope: !33, file: !32, line: 9, type: !10, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!32 = !DIFile(filename: "examples/../../Nexagate/crates/gate-quic/src/varint.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "1163804f5bf65f5cd7e61fb8c1363ee4")
!33 = !DINamespace(name: "varint", scope: !9)
!34 = distinct !DILocation(line: 13, column: 18, scope: !23)
!35 = !DILocation(line: 10, column: 18, scope: !31, inlinedAt: !34)
!36 = !DILocation(line: 11, column: 25, scope: !37, inlinedAt: !34)
!37 = distinct !DILexicalBlock(scope: !31, file: !32, line: 10, column: 5)
!38 = !DILocation(line: 11, column: 15, scope: !37, inlinedAt: !34)
!39 = !DILocation(line: 369, column: 16, scope: !40, inlinedAt: !44)
!40 = distinct !DISubprogram(name: "get<u8>", linkageName: "_ZN106_$LT$core..ops..range..Range$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$3get17hbd2f606d57d3e720E", scope: !42, file: !41, line: 366, type: !10, scopeLine: 366, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!41 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs", directory: "", checksumkind: CSK_MD5, checksum: "ea08754c2648ab0c9215e0939d287f09")
!42 = !DINamespace(name: "{impl#4}", scope: !43)
!43 = !DINamespace(name: "index", scope: !29)
!44 = distinct !DILocation(line: 507, column: 23, scope: !45, inlinedAt: !47)
!45 = distinct !DISubprogram(name: "get<u8>", linkageName: "_ZN108_$LT$core..ops..range..RangeTo$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$3get17h0456f4f8bed9cb37E", scope: !46, file: !41, line: 506, type: !10, scopeLine: 506, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!46 = !DINamespace(name: "{impl#6}", scope: !43)
!47 = distinct !DILocation(line: 576, column: 15, scope: !48, inlinedAt: !49)
!48 = distinct !DISubprogram(name: "get<u8, core::ops::range::RangeTo<usize>>", linkageName: "_ZN4core5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$3get17h5c05adbda7465d0cE", scope: !28, file: !26, line: 572, type: !10, scopeLine: 572, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!49 = distinct !DILocation(line: 12, column: 23, scope: !50, inlinedAt: !34)
!50 = distinct !DILexicalBlock(scope: !37, file: !32, line: 11, column: 5)
!51 = !DILocation(line: 12, column: 17, scope: !50, inlinedAt: !34)
!52 = !DILocation(line: 961, column: 18, scope: !53, inlinedAt: !58)
!53 = distinct !DISubprogram(name: "add<u8>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$3add17h614df7203aee16ceE", scope: !55, file: !54, line: 927, type: !10, scopeLine: 927, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!54 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs", directory: "", checksumkind: CSK_MD5, checksum: "741f53a1b46ef31ed27a8da0b10d1ba5")
!55 = !DINamespace(name: "{impl#0}", scope: !56)
!56 = !DINamespace(name: "mut_ptr", scope: !57)
!57 = !DINamespace(name: "ptr", scope: !21)
!58 = distinct !DILocation(line: 102, column: 78, scope: !59, inlinedAt: !65)
!59 = distinct !DILexicalBlock(scope: !61, file: !60, line: 98, column: 9)
!60 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs", directory: "", checksumkind: CSK_MD5, checksum: "f33ab2e22fe09095bf73c41c52bd166c")
!61 = distinct !DILexicalBlock(scope: !62, file: !60, line: 97, column: 9)
!62 = distinct !DISubprogram(name: "new<u8>", linkageName: "_ZN4core5slice4iter13Iter$LT$T$GT$3new17h8c196e1a8b00bb4aE", scope: !63, file: !60, line: 96, type: !10, scopeLine: 96, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!63 = !DINamespace(name: "Iter", scope: !64)
!64 = !DINamespace(name: "iter", scope: !29)
!65 = distinct !DILocation(line: 1041, column: 9, scope: !66, inlinedAt: !67)
!66 = distinct !DISubprogram(name: "iter<u8>", linkageName: "_ZN4core5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$4iter17h7d8256d3d46bc264E", scope: !28, file: !26, line: 1040, type: !10, scopeLine: 1040, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!67 = distinct !DILocation(line: 26, column: 14, scope: !68, inlinedAt: !70)
!68 = distinct !DISubprogram(name: "into_iter<u8>", linkageName: "_ZN4core5slice4iter87_$LT$impl$u20$core..iter..traits..collect..IntoIterator$u20$for$u20$$RF$$u5b$T$u5d$$GT$9into_iter17h91a525d3497474cbE", scope: !69, file: !60, line: 25, type: !10, scopeLine: 25, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!69 = !DINamespace(name: "{impl#1}", scope: !64)
!70 = distinct !DILocation(line: 14, column: 15, scope: !71, inlinedAt: !34)
!71 = distinct !DILexicalBlock(scope: !72, file: !32, line: 13, column: 5)
!72 = distinct !DILexicalBlock(scope: !50, file: !32, line: 12, column: 5)
!73 = !DILocation(line: 1692, column: 9, scope: !74, inlinedAt: !78)
!74 = distinct !DISubprogram(name: "eq<u8>", linkageName: "_ZN78_$LT$core..ptr..non_null..NonNull$LT$T$GT$$u20$as$u20$core..cmp..PartialEq$GT$2eq17h6e46c8d3403a069cE", scope: !76, file: !75, line: 1691, type: !10, scopeLine: 1691, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!75 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs", directory: "", checksumkind: CSK_MD5, checksum: "b9319c401ce0dd8d5662df7fdc4fc942")
!76 = !DINamespace(name: "{impl#16}", scope: !77)
!77 = !DINamespace(name: "non_null", scope: !57)
!78 = distinct !DILocation(line: 180, column: 28, scope: !79, inlinedAt: !84)
!79 = distinct !DILexicalBlock(scope: !81, file: !80, line: 162, column: 17)
!80 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs", directory: "", checksumkind: CSK_MD5, checksum: "87d1f0c2746f51593d75ddf4c9271f14")
!81 = distinct !DILexicalBlock(scope: !82, file: !80, line: 161, column: 17)
!82 = distinct !DISubprogram(name: "next<u8>", linkageName: "_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h2beda646c31fd27fE", scope: !83, file: !80, line: 157, type: !10, scopeLine: 157, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!83 = !DINamespace(name: "{impl#171}", scope: !64)
!84 = distinct !DILocation(line: 14, column: 15, scope: !85, inlinedAt: !34)
!85 = distinct !DILexicalBlock(scope: !71, file: !32, line: 14, column: 5)
!86 = !DILocation(line: 14, column: 15, scope: !85, inlinedAt: !34)
!87 = !DILocation(line: 13, column: 31, scope: !72, inlinedAt: !34)
!88 = !DILocation(line: 79, column: 17, scope: !89, inlinedAt: !91)
!89 = distinct !DISubprogram(name: "from", linkageName: "_ZN4core7convert3num63_$LT$impl$u20$core..convert..From$LT$u8$GT$$u20$for$u20$u64$GT$4from17h13fb8097f6320c2eE", scope: !90, file: !17, line: 78, type: !10, scopeLine: 78, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !11)
!90 = !DINamespace(name: "{impl#66}", scope: !19)
!91 = distinct !DILocation(line: 13, column: 21, scope: !72, inlinedAt: !34)
!92 = !DILocation(line: 180, column: 28, scope: !79, inlinedAt: !84)
!93 = !DILocation(line: 14, column: 10, scope: !85, inlinedAt: !34)
!94 = !{!95}
!95 = distinct !{!95, !96, !"_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E: %input.0"}
!96 = distinct !{!96, !"_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E"}
!97 = !{!98}
!98 = distinct !{!98, !96, !"_ZN13nexagate_quic6varint4read17he6ed322c7f84a9b3E: %_0"}
!99 = !DILocation(line: 15, column: 17, scope: !100, inlinedAt: !34)
!100 = distinct !DILexicalBlock(scope: !85, file: !32, line: 14, column: 27)
!101 = !DILocation(line: 79, column: 17, scope: !89, inlinedAt: !102)
!102 = distinct !DILocation(line: 15, column: 30, scope: !100, inlinedAt: !34)
!103 = !DILocation(line: 15, column: 9, scope: !100, inlinedAt: !34)
!104 = distinct !{!104, !105}
!105 = !{!"llvm.loop.unroll.disable"}
!106 = !DILocation(line: 14, column: 8, scope: !107)
!107 = distinct !DILexicalBlock(scope: !23, file: !8, line: 13, column: 5)
!108 = !DILocation(line: 0, scope: !31, inlinedAt: !34)
!109 = !DILocation(line: 22, column: 1, scope: !7)
!110 = !DILocation(line: 22, column: 2, scope: !7)
!111 = !DILocation(line: 17, column: 30, scope: !107)
!112 = !DILocation(line: 17, column: 20, scope: !107)
!113 = !DILocation(line: 18, column: 5, scope: !114)
!114 = distinct !DILexicalBlock(scope: !107, file: !8, line: 17, column: 5)
!115 = !DILocation(line: 19, column: 32, scope: !116)
!116 = distinct !DILexicalBlock(scope: !114, file: !8, line: 19, column: 9)
!117 = !DILocation(line: 20, column: 17, scope: !114)
!118 = !DILocation(line: 20, column: 43, scope: !114)
!119 = !DILocation(line: 19, column: 52, scope: !116)
