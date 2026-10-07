; ModuleID = 'nexagate_hpack_before.6f0f2380cbf4442f-cgu.0'
source_filename = "nexagate_hpack_before.6f0f2380cbf4442f-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%"core::ops::range::Range<usize>" = type { i64, i64 }
%"alloc::vec::Vec<(&[u8], usize)>" = type { %"alloc::raw_vec::RawVec<(&[u8], usize)>", i64 }
%"alloc::raw_vec::RawVec<(&[u8], usize)>" = type { %"alloc::raw_vec::RawVecInner", %"core::marker::PhantomData<(&[u8], usize)>" }
%"alloc::raw_vec::RawVecInner" = type { i64, ptr, %"alloc::alloc::Global" }
%"alloc::alloc::Global" = type {}
%"core::marker::PhantomData<(&[u8], usize)>" = type {}

@alloc_6e448024d909ecbc6c7184524def04d9 = private unnamed_addr constant [127 x i8] c"/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/enumerate.rs\00", align 1
@alloc_aeb2f4e6771f847856ffd3202edf9987 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_6e448024d909ecbc6c7184524def04d9, [16 x i8] c"~\00\00\00\00\00\00\00R\00\00\00\09\00\00\00" }>, align 8
@alloc_18ff8af99e51a98d72b531fe691d1961 = private unnamed_addr constant [17 x i8] c"truncated integer", align 1
@alloc_c6dcf4b47faac55369efcef85d6c9bf1 = private unnamed_addr constant [50 x i8] c"examples/../fixtures/nexagate-before/hpack/mod.rs\00", align 1
@alloc_db0d3c4eb7c152921f9afd9fce696e08 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_c6dcf4b47faac55369efcef85d6c9bf1, [16 x i8] c"1\00\00\00\00\00\00\00N\00\00\00\12\00\00\00" }>, align 8
@alloc_02127657331c02dbcc0a53a744076139 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_c6dcf4b47faac55369efcef85d6c9bf1, [16 x i8] c"1\00\00\00\00\00\00\00N\00\00\00\09\00\00\00" }>, align 8
@alloc_b9378ea015704e9df6c0470e86732e03 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_c6dcf4b47faac55369efcef85d6c9bf1, [16 x i8] c"1\00\00\00\00\00\00\00S\00\00\00\1F\00\00\00" }>, align 8
@alloc_ea701fff40f262979702ba2c66956fda = private unnamed_addr constant [17 x i8] c"integer too large", align 1
@alloc_c59fa46376452fc07c4824b416899185 = private unnamed_addr constant [27 x i8] c"\18malformed header block: \C0\00", align 1
@alloc_140607d7eb711ca40e4cb0abc623dac6 = private unnamed_addr constant [23 x i8] c"\14bad Huffman string: \C0\00", align 1
@alloc_bf7f5d82af619205829c8b0897e5f61e = private unnamed_addr constant [21 x i8] c"header list too large", align 1

; <alloc::vec::Vec<T,A> as alloc::vec::spec_extend::SpecExtend<&T,core::slice::iter::Iter<T>>>::spec_extend
; Function Attrs: nounwind nonlazybind uwtable
define void @"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h9308aadac9002e8dE"(ptr noalias noundef align 8 dereferenceable(24) %self, ptr noundef nonnull %0, ptr noundef %1) unnamed_addr #0 !dbg !7 {
start:
  %2 = icmp ne ptr %1, null
  tail call void @llvm.assume(i1 %2)
  %3 = ptrtoint ptr %1 to i64, !dbg !15
  %4 = ptrtoint ptr %0 to i64, !dbg !15
  %5 = sub nuw i64 %3, %4, !dbg !15
; call alloc::vec::Vec<T,A>::reserve
  tail call void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h2f692e8ba7e2c189E"(ptr noalias noundef nonnull align 8 dereferenceable(24) %self, i64 noundef %5) #16, !dbg !45
  %6 = getelementptr inbounds nuw i8, ptr %self, i64 16, !dbg !52
  %len.i = load i64, ptr %6, align 8, !dbg !52, !alias.scope !55, !noundef !14
  %_9.i = icmp sgt i64 %len.i, -1, !dbg !58
  tail call void @llvm.assume(i1 %_9.i), !dbg !60
  %7 = getelementptr inbounds nuw i8, ptr %self, i64 8, !dbg !61
  %_10.i = load ptr, ptr %7, align 8, !dbg !61, !alias.scope !55, !nonnull !14, !noundef !14
  %dst.i = getelementptr inbounds nuw i8, ptr %_10.i, i64 %len.i, !dbg !75
  tail call void @llvm.memcpy.p0.p0.i64(ptr nonnull align 1 %dst.i, ptr nonnull readonly align 1 %0, i64 %5, i1 false), !dbg !78
  %8 = load i64, ptr %6, align 8, !dbg !82, !alias.scope !55, !noundef !14
  %9 = add i64 %8, %5, !dbg !82
  store i64 %9, ptr %6, align 8, !dbg !82, !alias.scope !55
  ret void, !dbg !83
}

; nexagate_hpack_before::hpack::decoder::FieldList::len
; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: read, inaccessiblemem: write) uwtable
define noundef range(i64 0, 288230376151711744) i64 @_ZN21nexagate_hpack_before5hpack7decoder9FieldList3len17h41e11c4b526fe49bE(ptr noalias noundef readonly align 8 captures(none) dereferenceable(48) %self) unnamed_addr #1 !dbg !84 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 40, !dbg !90
  %_0.i = load i64, ptr %0, align 8, !dbg !90, !alias.scope !93, !noundef !14
  %_2.i = icmp ult i64 %_0.i, 288230376151711744, !dbg !96
  tail call void @llvm.assume(i1 %_2.i), !dbg !98
  ret i64 %_0.i, !dbg !99
}

; nexagate_hpack_before::hpack::decoder::FieldList::iter
; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: readwrite) uwtable
define void @_ZN21nexagate_hpack_before5hpack7decoder9FieldList4iter17hf9f9e9ede7b62fd9E(ptr dead_on_unwind noalias noundef writable writeonly sret([24 x i8]) align 8 captures(none) dereferenceable(24) initializes((0, 24)) %_0, ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(48) %self) unnamed_addr #2 !dbg !100 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 32, !dbg !101
  %_5.i = load ptr, ptr %0, align 8, !dbg !101, !alias.scope !115, !nonnull !14, !noundef !14
  %1 = getelementptr inbounds nuw i8, ptr %self, i64 40, !dbg !118
  %_4.i = load i64, ptr %1, align 8, !dbg !118, !alias.scope !115, !noundef !14
  %_6.i.i = getelementptr inbounds nuw { %"core::ops::range::Range<usize>", %"core::ops::range::Range<usize>" }, ptr %_5.i, i64 %_4.i, !dbg !119
  store ptr %_5.i, ptr %_0, align 8, !dbg !130, !alias.scope !144, !noalias !147
  %2 = getelementptr inbounds nuw i8, ptr %_0, i64 8, !dbg !130
  store ptr %_6.i.i, ptr %2, align 8, !dbg !130, !alias.scope !144, !noalias !147
  %3 = getelementptr inbounds nuw i8, ptr %_0, i64 16, !dbg !130
  store ptr %self, ptr %3, align 8, !dbg !130, !alias.scope !144, !noalias !147
  ret void, !dbg !149
}

; nexagate_hpack_before::hpack::decoder::FieldList::push
; Function Attrs: nounwind nonlazybind uwtable
define void @_ZN21nexagate_hpack_before5hpack7decoder9FieldList4push17h46cdc8bdfe629df2E(ptr noalias noundef align 8 dereferenceable(48) %self, ptr noalias noundef nonnull readonly align 1 captures(none) %name.0, i64 noundef range(i64 0, -9223372036854775808) %name.1, ptr noalias noundef nonnull readonly align 1 captures(none) %value.0, i64 noundef range(i64 0, -9223372036854775808) %value.1) unnamed_addr #0 personality ptr @rust_eh_personality !dbg !150 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 16, !dbg !151
  %_0.i = load i64, ptr %0, align 8, !dbg !151, !alias.scope !154, !noundef !14
  %_2.i = icmp sgt i64 %_0.i, -1, !dbg !157
  tail call void @llvm.assume(i1 %_2.i), !dbg !159
; call alloc::vec::Vec<T,A>::reserve
  tail call void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h2f692e8ba7e2c189E"(ptr noalias noundef nonnull align 8 dereferenceable(24) %self, i64 noundef range(i64 0, -9223372036854775808) %name.1) #16, !dbg !160, !noalias !166
  %len.i.i.i = load i64, ptr %0, align 8, !dbg !169, !alias.scope !171, !noalias !166, !noundef !14
  %_9.i.i.i = icmp sgt i64 %len.i.i.i, -1, !dbg !177
  tail call void @llvm.assume(i1 %_9.i.i.i), !dbg !178
  %1 = getelementptr inbounds nuw i8, ptr %self, i64 8, !dbg !179
  %_10.i.i.i = load ptr, ptr %1, align 8, !dbg !179, !alias.scope !171, !noalias !166, !nonnull !14, !noundef !14
  %dst.i.i.i = getelementptr inbounds nuw i8, ptr %_10.i.i.i, i64 %len.i.i.i, !dbg !184
  tail call void @llvm.memcpy.p0.p0.i64(ptr nonnull align 1 %dst.i.i.i, ptr nonnull readonly align 1 %name.0, i64 range(i64 0, -9223372036854775808) %name.1, i1 false), !dbg !186
  %2 = load i64, ptr %0, align 8, !dbg !188, !alias.scope !171, !noalias !166, !noundef !14
  %3 = add i64 %2, %name.1, !dbg !188
  store i64 %3, ptr %0, align 8, !dbg !188, !alias.scope !171, !noalias !166
  %_2.i5 = icmp sgt i64 %3, -1, !dbg !189
  tail call void @llvm.assume(i1 %_2.i5), !dbg !191
; call alloc::vec::Vec<T,A>::reserve
  tail call void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h2f692e8ba7e2c189E"(ptr noalias noundef nonnull align 8 dereferenceable(24) %self, i64 noundef range(i64 0, -9223372036854775808) %value.1) #16, !dbg !192, !noalias !197
  %len.i.i.i8 = load i64, ptr %0, align 8, !dbg !200, !alias.scope !202, !noalias !197, !noundef !14
  %_9.i.i.i9 = icmp sgt i64 %len.i.i.i8, -1, !dbg !208
  tail call void @llvm.assume(i1 %_9.i.i.i9), !dbg !209
  %_10.i.i.i10 = load ptr, ptr %1, align 8, !dbg !210, !alias.scope !202, !noalias !197, !nonnull !14, !noundef !14
  %dst.i.i.i11 = getelementptr inbounds nuw i8, ptr %_10.i.i.i10, i64 %len.i.i.i8, !dbg !215
  tail call void @llvm.memcpy.p0.p0.i64(ptr nonnull align 1 %dst.i.i.i11, ptr nonnull readonly align 1 %value.0, i64 range(i64 0, -9223372036854775808) %value.1, i1 false), !dbg !217
  %4 = load i64, ptr %0, align 8, !dbg !219, !alias.scope !202, !noalias !197, !noundef !14
  %5 = add i64 %4, %value.1, !dbg !219
  store i64 %5, ptr %0, align 8, !dbg !219, !alias.scope !202, !noalias !197
  %_25 = getelementptr inbounds nuw i8, ptr %self, i64 24, !dbg !220
  %6 = getelementptr inbounds nuw i8, ptr %self, i64 40, !dbg !221
  %len.i.i = load i64, ptr %6, align 8, !dbg !221, !alias.scope !226, !noalias !231, !noundef !14
  %self1.i.i = load i64, ptr %_25, align 8, !dbg !234, !range !240, !alias.scope !226, !noalias !231, !noundef !14
  %_4.i.i = icmp eq i64 %len.i.i, %self1.i.i, !dbg !241
  br i1 %_4.i.i, label %bb1.i.i, label %"_ZN5alloc3vec16Vec$LT$T$C$A$GT$4push17hb288ea25d35b9a2cE.exit", !dbg !241

bb1.i.i:                                          ; preds = %start
; call alloc::raw_vec::RawVec<T,A>::grow_one
  tail call void @"_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h76175b00c5618055E"(ptr noalias noundef nonnull align 8 dereferenceable(24) %_25) #17, !dbg !242, !noalias !231
  br label %"_ZN5alloc3vec16Vec$LT$T$C$A$GT$4push17hb288ea25d35b9a2cE.exit", !dbg !243

"_ZN5alloc3vec16Vec$LT$T$C$A$GT$4push17hb288ea25d35b9a2cE.exit": ; preds = %start, %bb1.i.i
  %_11.0 = add nuw i64 %_0.i, %name.1, !dbg !244
  %_21.0 = add nuw i64 %3, %value.1, !dbg !245
  %7 = getelementptr inbounds nuw i8, ptr %self, i64 32, !dbg !246
  %_14.i.i = load ptr, ptr %7, align 8, !dbg !246, !alias.scope !226, !noalias !231, !nonnull !14, !noundef !14
  %end.i.i = getelementptr inbounds nuw { %"core::ops::range::Range<usize>", %"core::ops::range::Range<usize>" }, ptr %_14.i.i, i64 %len.i.i, !dbg !255
  store i64 %_0.i, ptr %end.i.i, align 8, !dbg !258
  %_26.sroa.4.0.end.i.i.sroa_idx = getelementptr inbounds nuw i8, ptr %end.i.i, i64 8, !dbg !258
  store i64 %_11.0, ptr %_26.sroa.4.0.end.i.i.sroa_idx, align 8, !dbg !258
  %_26.sroa.5.0.end.i.i.sroa_idx = getelementptr inbounds nuw i8, ptr %end.i.i, i64 16, !dbg !258
  store i64 %3, ptr %_26.sroa.5.0.end.i.i.sroa_idx, align 8, !dbg !258
  %_26.sroa.6.0.end.i.i.sroa_idx = getelementptr inbounds nuw i8, ptr %end.i.i, i64 24, !dbg !258
  store i64 %_21.0, ptr %_26.sroa.6.0.end.i.i.sroa_idx, align 8, !dbg !258
  %8 = add i64 %len.i.i, 1, !dbg !262
  store i64 %8, ptr %6, align 8, !dbg !262, !alias.scope !226, !noalias !231
  ret void, !dbg !263
}

; nexagate_hpack_before::hpack::decoder::FieldList::clear
; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: write) uwtable
define void @_ZN21nexagate_hpack_before5hpack7decoder9FieldList5clear17hef74b301bd19135cE(ptr noalias noundef writeonly align 8 captures(none) dereferenceable(48) initializes((16, 24), (40, 48)) %self) unnamed_addr #3 !dbg !264 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 16, !dbg !265
  store i64 0, ptr %0, align 8, !dbg !270, !alias.scope !272
  %1 = getelementptr inbounds nuw i8, ptr %self, i64 40, !dbg !275
  store i64 0, ptr %1, align 8, !dbg !280, !alias.scope !282
  ret void, !dbg !285
}

; nexagate_hpack_before::hpack::decoder::FieldList::is_empty
; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: read, inaccessiblemem: write) uwtable
define noundef zeroext i1 @_ZN21nexagate_hpack_before5hpack7decoder9FieldList8is_empty17h9e8ba878cb51e8a7E(ptr noalias noundef readonly align 8 captures(none) dereferenceable(48) %self) unnamed_addr #1 !dbg !286 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 40, !dbg !287
  %_2.i = load i64, ptr %0, align 8, !dbg !287, !alias.scope !292, !noundef !14
  %_3.i = icmp ult i64 %_2.i, 288230376151711744, !dbg !295
  tail call void @llvm.assume(i1 %_3.i), !dbg !297
  %_0.i = icmp eq i64 %_2.i, 0, !dbg !298
  ret i1 %_0.i, !dbg !299
}

; <&T as core::fmt::Display>::fmt
; Function Attrs: nounwind nonlazybind uwtable
define noundef zeroext i1 @"_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hf912134bce2d0087E"(ptr noalias noundef readonly align 8 captures(none) dereferenceable(8) %self, ptr noalias noundef align 8 dereferenceable(24) %f) unnamed_addr #0 !dbg !300 {
start:
  %_3 = load ptr, ptr %self, align 8, !dbg !304, !nonnull !14, !align !305, !noundef !14
; call <&T as core::fmt::Display>::fmt
  %_0 = tail call noundef zeroext i1 @"_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h28908b04add44bbcE"(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(16) %_3, ptr noalias noundef align 8 dereferenceable(24) %f) #16, !dbg !306
  ret i1 %_0, !dbg !307
}

; alloc::vec::Vec<T,A>::extend_from_slice
; Function Attrs: nounwind nonlazybind uwtable
define void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$17extend_from_slice17he23345362ef15a58E"(ptr noalias noundef align 8 dereferenceable(24) %self, ptr noalias noundef nonnull readonly align 1 captures(none) %other.0, i64 noundef range(i64 0, -9223372036854775808) %other.1) unnamed_addr #0 !dbg !163 {
start:
; call alloc::vec::Vec<T,A>::reserve
  tail call void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h2f692e8ba7e2c189E"(ptr noalias noundef nonnull align 8 dereferenceable(24) %self, i64 noundef %other.1) #16, !dbg !308
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 16, !dbg !311
  %len.i.i = load i64, ptr %0, align 8, !dbg !311, !alias.scope !313, !noundef !14
  %_9.i.i = icmp sgt i64 %len.i.i, -1, !dbg !318
  tail call void @llvm.assume(i1 %_9.i.i), !dbg !319
  %1 = getelementptr inbounds nuw i8, ptr %self, i64 8, !dbg !320
  %_10.i.i = load ptr, ptr %1, align 8, !dbg !320, !alias.scope !313, !nonnull !14, !noundef !14
  %dst.i.i = getelementptr inbounds nuw i8, ptr %_10.i.i, i64 %len.i.i, !dbg !325
  tail call void @llvm.memcpy.p0.p0.i64(ptr nonnull align 1 %dst.i.i, ptr nonnull readonly align 1 %other.0, i64 %other.1, i1 false), !dbg !327
  %2 = load i64, ptr %0, align 8, !dbg !329, !alias.scope !313, !noundef !14
  %3 = add i64 %2, %other.1, !dbg !329
  store i64 %3, ptr %0, align 8, !dbg !329, !alias.scope !313
  ret void, !dbg !330
}

; alloc::raw_vec::RawVec<T,A>::grow_one
; Function Attrs: noinline nounwind nonlazybind uwtable
define void @"_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h76175b00c5618055E"(ptr noalias noundef align 8 dereferenceable(16) %self) unnamed_addr #4 !dbg !331 {
start:
  %self1 = load i64, ptr %self, align 8, !dbg !332, !range !240, !noundef !14
; call alloc::raw_vec::RawVecInner<A>::grow_amortized
  %0 = tail call { i64, i64 } @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$14grow_amortized17h7368fa7136fd38e3E"(ptr noalias noundef align 8 dereferenceable(16) %self, i64 noundef %self1, i64 noundef 1, i64 noundef 8, i64 noundef 32) #16, !dbg !336
  %1 = extractvalue { i64, i64 } %0, 0, !dbg !336
  %.not = icmp eq i64 %1, -9223372036854775807, !dbg !337
  br i1 %.not, label %bb3, label %bb2, !dbg !338, !prof !339

bb2:                                              ; preds = %start
  %2 = extractvalue { i64, i64 } %0, 1, !dbg !336
; call alloc::raw_vec::handle_error
  tail call void @_ZN5alloc7raw_vec12handle_error17hfa86a3a4628bd209E(i64 noundef %1, i64 %2) #18, !dbg !340
  unreachable, !dbg !340

bb3:                                              ; preds = %start
  ret void, !dbg !341
}

; <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(none) uwtable
define void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h580f52b1ec55ace2E"(ptr noalias noundef readnone align 8 captures(none) dereferenceable(24) %self) unnamed_addr #5 !dbg !342 {
start:
  ret void, !dbg !344
}

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: nounwind nonlazybind uwtable
define void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h1babb42875e99263E"(ptr noalias noundef align 8 dereferenceable(16) %self) unnamed_addr #0 !dbg !345 {
start:
; call alloc::raw_vec::RawVecInner<A>::deallocate
  tail call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h1ab787cdb4e63fb3E"(ptr noalias noundef align 8 dereferenceable(16) %self, i64 noundef 8, i64 noundef 24) #16, !dbg !347
  ret void, !dbg !348
}

; <hashbrown::raw::RawTable<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: nounwind nonlazybind uwtable
define void @"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hb71e566929c0c2b0E"(ptr noalias noundef readonly align 8 captures(none) dereferenceable(32) %self) unnamed_addr #0 !dbg !349 {
start:
; call hashbrown::raw::RawTableInner::drop_inner_table
  tail call void @_ZN9hashbrown3raw13RawTableInner16drop_inner_table17hf55df0bb53f2638dE(ptr noalias noundef align 8 dereferenceable(32) %self, ptr noalias nonnull readonly align 1 captures(address, read_provenance) poison, i64 noundef 48, i64 noundef 16) #16, !dbg !354
  ret void, !dbg !355
}

; <nexagate_hpack_before::hpack::HpackError as core::fmt::Display>::fmt
; Function Attrs: nounwind nonlazybind uwtable
define noundef zeroext i1 @"_ZN79_$LT$nexagate_hpack_before..hpack..HpackError$u20$as$u20$core..fmt..Display$GT$3fmt17h3d558c1ca0f89cedE"(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24) %self, ptr noalias noundef align 8 dereferenceable(24) %f) unnamed_addr #0 !dbg !356 {
start:
  %args2 = alloca [16 x i8], align 8
  %m1 = alloca [8 x i8], align 8
  %args = alloca [16 x i8], align 8
  %m = alloca [8 x i8], align 8
  %_3 = load i64, ptr %self, align 8, !dbg !359, !range !360, !noundef !14
  switch i64 %_3, label %default.unreachable16 [
    i64 0, label %bb4
    i64 1, label %bb3
    i64 2, label %bb2
  ], !dbg !361

default.unreachable16:                            ; preds = %start
  unreachable

bb4:                                              ; preds = %start
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %m), !dbg !362
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 8, !dbg !362
  store ptr %0, ptr %m, align 8, !dbg !362
  call void @llvm.lifetime.start.p0(i64 16, ptr nonnull %args), !dbg !363
  store ptr %m, ptr %args, align 8, !dbg !363
  %_8.sroa.4.0.args.sroa_idx = getelementptr inbounds nuw i8, ptr %args, i64 8, !dbg !363
  store ptr @"_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hf912134bce2d0087E", ptr %_8.sroa.4.0.args.sroa_idx, align 8, !dbg !363
  call void @llvm.experimental.noalias.scope.decl(metadata !368), !dbg !371
  %_8.0.i = load ptr, ptr %f, align 8, !dbg !372, !alias.scope !368, !nonnull !14, !align !376, !noundef !14
  %1 = getelementptr inbounds nuw i8, ptr %f, i64 8, !dbg !372
  %_8.1.i = load ptr, ptr %1, align 8, !dbg !372, !alias.scope !368, !nonnull !14, !align !305, !noundef !14
; call core::fmt::write
  %2 = call noundef zeroext i1 @_ZN4core3fmt5write17h469843d235cc4241E(ptr noundef nonnull align 1 %_8.0.i, ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(48) %_8.1.i, ptr noundef nonnull @alloc_c59fa46376452fc07c4824b416899185, ptr noundef nonnull %args) #16, !dbg !377, !noalias !368
  call void @llvm.lifetime.end.p0(i64 16, ptr nonnull %args), !dbg !378
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %m), !dbg !378
  br label %bb11, !dbg !378

bb3:                                              ; preds = %start
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %m1), !dbg !379
  %3 = getelementptr inbounds nuw i8, ptr %self, i64 8, !dbg !379
  store ptr %3, ptr %m1, align 8, !dbg !379
  call void @llvm.lifetime.start.p0(i64 16, ptr nonnull %args2), !dbg !380
  store ptr %m1, ptr %args2, align 8, !dbg !380
  %_14.sroa.4.0.args2.sroa_idx = getelementptr inbounds nuw i8, ptr %args2, i64 8, !dbg !380
  store ptr @"_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hf912134bce2d0087E", ptr %_14.sroa.4.0.args2.sroa_idx, align 8, !dbg !380
  call void @llvm.experimental.noalias.scope.decl(metadata !384), !dbg !387
  %_8.0.i8 = load ptr, ptr %f, align 8, !dbg !388, !alias.scope !384, !nonnull !14, !align !376, !noundef !14
  %4 = getelementptr inbounds nuw i8, ptr %f, i64 8, !dbg !388
  %_8.1.i9 = load ptr, ptr %4, align 8, !dbg !388, !alias.scope !384, !nonnull !14, !align !305, !noundef !14
; call core::fmt::write
  %5 = call noundef zeroext i1 @_ZN4core3fmt5write17h469843d235cc4241E(ptr noundef nonnull align 1 %_8.0.i8, ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(48) %_8.1.i9, ptr noundef nonnull @alloc_140607d7eb711ca40e4cb0abc623dac6, ptr noundef nonnull %args2) #16, !dbg !390, !noalias !384
  call void @llvm.lifetime.end.p0(i64 16, ptr nonnull %args2), !dbg !391
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %m1), !dbg !391
  br label %bb11, !dbg !391

bb2:                                              ; preds = %start
; call core::fmt::Formatter::write_str
  %6 = tail call noundef zeroext i1 @_ZN4core3fmt9Formatter9write_str17h72189eba35977850E(ptr noalias noundef align 8 dereferenceable(24) %f, ptr noalias noundef nonnull readonly align 1 captures(address, read_provenance) @alloc_bf7f5d82af619205829c8b0897e5f61e, i64 noundef 21) #16, !dbg !392
  br label %bb11, !dbg !392

bb11:                                             ; preds = %bb2, %bb3, %bb4
  %_0.sroa.0.0.in = phi i1 [ %2, %bb4 ], [ %5, %bb3 ], [ %6, %bb2 ]
  ret i1 %_0.sroa.0.0.in, !dbg !393
}

; <nexagate_hpack_before::hpack::encoder::Encoder as core::default::Default>::default
; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: write) uwtable
define void @"_ZN89_$LT$nexagate_hpack_before..hpack..encoder..Encoder$u20$as$u20$core..default..Default$GT$7default17h80dd29de72c76951E"(ptr dead_on_unwind noalias noundef writable writeonly sret([96 x i8]) align 8 captures(none) dereferenceable(96) initializes((0, 8), (24, 96)) %_0) unnamed_addr #3 !dbg !394 {
start:
  %_1.sroa.5.0..sroa_idx.i = getelementptr inbounds nuw i8, ptr %_0, i64 40, !dbg !398
  tail call void @llvm.memset.p0.i64(ptr noundef nonnull align 8 dereferenceable(16) %_1.sroa.5.0..sroa_idx.i, i8 0, i64 16, i1 false), !dbg !402, !alias.scope !420
  %0 = getelementptr inbounds nuw i8, ptr %_0, i64 24, !dbg !398
  store i64 0, ptr %0, align 8, !dbg !398, !alias.scope !420
  %_1.sroa.4.0..sroa_idx.i = getelementptr inbounds nuw i8, ptr %_0, i64 32, !dbg !398
  store ptr inttoptr (i64 8 to ptr), ptr %_1.sroa.4.0..sroa_idx.i, align 8, !dbg !398, !alias.scope !420
  %_1.sroa.6.0..sroa_idx.i = getelementptr inbounds nuw i8, ptr %_0, i64 56, !dbg !398
  store i64 0, ptr %_1.sroa.6.0..sroa_idx.i, align 8, !dbg !398, !alias.scope !420
  %_1.sroa.7.0..sroa_idx.i = getelementptr inbounds nuw i8, ptr %_0, i64 64, !dbg !398
  store i64 4096, ptr %_1.sroa.7.0..sroa_idx.i, align 8, !dbg !398, !alias.scope !420
  store i64 0, ptr %_0, align 8, !dbg !398, !alias.scope !420
  %1 = getelementptr inbounds nuw i8, ptr %_0, i64 72, !dbg !398
  store i64 0, ptr %1, align 8, !dbg !398, !alias.scope !420
  %_3.sroa.4.0..sroa_idx.i = getelementptr inbounds nuw i8, ptr %_0, i64 80, !dbg !398
  store ptr inttoptr (i64 1 to ptr), ptr %_3.sroa.4.0..sroa_idx.i, align 8, !dbg !398, !alias.scope !420
  %_3.sroa.5.0..sroa_idx.i = getelementptr inbounds nuw i8, ptr %_0, i64 88, !dbg !398
  store i64 0, ptr %_3.sroa.5.0..sroa_idx.i, align 8, !dbg !398, !alias.scope !420
  ret void, !dbg !423
}

; hashbrown::raw::RawTableInner::drop_elements
; Function Attrs: nounwind nonlazybind uwtable
define void @_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E(ptr noalias noundef readonly align 8 captures(none) dereferenceable(32) %self) unnamed_addr #0 !dbg !424 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 24, !dbg !426
  %_2 = load i64, ptr %0, align 8, !dbg !426, !noundef !14
  %1 = icmp eq i64 %_2, 0, !dbg !426
  br i1 %1, label %bb8, label %bb2, !dbg !426

bb2:                                              ; preds = %start
  %self3 = load ptr, ptr %self, align 8, !dbg !427, !nonnull !14, !noundef !14
  %2 = load <16 x i8>, ptr %self3, align 16, !dbg !432, !noalias !451
  %3 = icmp slt <16 x i8> %2, zeroinitializer, !dbg !456
  %4 = bitcast <16 x i1> %3 to i16, !dbg !456
  %_23.i = xor i16 %4, -1, !dbg !466
  %next_ctrl.i = getelementptr inbounds nuw i8, ptr %self3, i64 16, !dbg !472
  br label %bb14, !dbg !476

bb8:                                              ; preds = %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit", %start
  ret void, !dbg !482

bb14:                                             ; preds = %bb2, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit"
  %iter.sroa.0.011 = phi ptr [ %self3, %bb2 ], [ %iter.sroa.0.1, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit" ]
  %iter.sroa.6.010 = phi ptr [ %next_ctrl.i, %bb2 ], [ %iter.sroa.6.1, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit" ]
  %iter.sroa.115.09 = phi i64 [ %_2, %bb2 ], [ %13, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit" ]
  %iter.sroa.84.08 = phi i16 [ %_23.i, %bb2 ], [ %_33.i, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit" ]
  %.not14.i = icmp eq i16 %iter.sroa.84.08, 0, !dbg !483
  br i1 %.not14.i, label %bb9.i, label %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit", !dbg !493

bb1.bb8_crit_edge.i:                              ; preds = %bb9.i
  %_55.i = xor i16 %8, -1, !dbg !494
  br label %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit", !dbg !493

bb9.i:                                            ; preds = %bb14, %bb9.i
  %_1717.i = phi ptr [ %_17.i, %bb9.i ], [ %iter.sroa.6.010, %bb14 ], !dbg !499
  %5 = phi ptr [ %9, %bb9.i ], [ %iter.sroa.0.011, %bb14 ]
  %6 = load <16 x i8>, ptr %_1717.i, align 16, !dbg !500, !noalias !504
  %7 = icmp slt <16 x i8> %6, zeroinitializer, !dbg !509
  %8 = bitcast <16 x i1> %7 to i16, !dbg !509
  %9 = getelementptr inbounds i8, ptr %5, i64 -768, !dbg !514
  %_17.i = getelementptr inbounds nuw i8, ptr %_1717.i, i64 16, !dbg !521
  %.not.i = icmp eq i16 %8, -1, !dbg !483
  br i1 %.not.i, label %bb9.i, label %bb1.bb8_crit_edge.i, !dbg !493

"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit": ; preds = %bb14, %bb1.bb8_crit_edge.i
  %iter.sroa.6.1 = phi ptr [ %_17.i, %bb1.bb8_crit_edge.i ], [ %iter.sroa.6.010, %bb14 ], !dbg !524
  %iter.sroa.0.1 = phi ptr [ %9, %bb1.bb8_crit_edge.i ], [ %iter.sroa.0.011, %bb14 ], !dbg !524
  %self3.lcssa.i = phi i16 [ %_55.i, %bb1.bb8_crit_edge.i ], [ %iter.sroa.84.08, %bb14 ], !dbg !525
  %10 = add i16 %self3.lcssa.i, -1, !dbg !526
  %11 = tail call range(i16 0, 17) i16 @llvm.cttz.i16(i16 %self3.lcssa.i, i1 true), !dbg !537
  %_24.i = zext nneg i16 %11 to i64, !dbg !538
  %_33.i = and i16 %10, %self3.lcssa.i, !dbg !539
  %_45.i = sub nsw i64 0, %_24.i, !dbg !543
  %12 = getelementptr inbounds { { ptr, i64 }, { i64, %"alloc::vec::Vec<(&[u8], usize)>" } }, ptr %iter.sroa.0.1, i64 %_45.i, !dbg !546
  %13 = add i64 %iter.sroa.115.09, -1, !dbg !547
  %14 = getelementptr inbounds i8, ptr %12, i64 -24, !dbg !549
; call alloc::raw_vec::RawVecInner<A>::deallocate
  tail call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h1ab787cdb4e63fb3E"(ptr noalias noundef nonnull align 8 dereferenceable(24) %14, i64 noundef 8, i64 noundef 24) #16, !dbg !559
  %15 = icmp eq i64 %13, 0, !dbg !476
  br i1 %15, label %bb8, label %bb14, !dbg !476
}

; hashbrown::raw::RawTableInner::drop_inner_table
; Function Attrs: nounwind nonlazybind uwtable
define void @_ZN9hashbrown3raw13RawTableInner16drop_inner_table17hf55df0bb53f2638dE(ptr noalias noundef readonly align 8 captures(none) dereferenceable(32) %self, ptr noalias nonnull readonly align 1 captures(none) %alloc, i64 noundef %table_layout.0, i64 noundef %table_layout.1) unnamed_addr #0 !dbg !565 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 8, !dbg !566
  %_5 = load i64, ptr %0, align 8, !dbg !566, !noundef !14
  %1 = icmp eq i64 %_5, 0, !dbg !569
  br i1 %1, label %bb4, label %bb2, !dbg !569

bb2:                                              ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !570), !dbg !573
  %2 = getelementptr inbounds nuw i8, ptr %self, i64 24, !dbg !574
  %_2.i = load i64, ptr %2, align 8, !dbg !574, !alias.scope !570, !noundef !14
  %3 = icmp eq i64 %_2.i, 0, !dbg !574
  br i1 %3, label %_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E.exit, label %bb2.i, !dbg !574

bb2.i:                                            ; preds = %bb2
  %self3.i = load ptr, ptr %self, align 8, !dbg !576, !alias.scope !570, !nonnull !14, !noundef !14
  %4 = load <16 x i8>, ptr %self3.i, align 16, !dbg !579, !noalias !583
  %5 = icmp slt <16 x i8> %4, zeroinitializer, !dbg !588
  %6 = bitcast <16 x i1> %5 to i16, !dbg !588
  %_23.i.i = xor i16 %6, -1, !dbg !592
  %next_ctrl.i.i = getelementptr inbounds nuw i8, ptr %self3.i, i64 16, !dbg !594
  br label %bb14.i, !dbg !596

bb14.i:                                           ; preds = %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i", %bb2.i
  %iter.sroa.0.011.i = phi ptr [ %self3.i, %bb2.i ], [ %iter.sroa.0.1.i, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i" ]
  %iter.sroa.6.010.i = phi ptr [ %next_ctrl.i.i, %bb2.i ], [ %iter.sroa.6.1.i, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i" ]
  %iter.sroa.115.09.i = phi i64 [ %_2.i, %bb2.i ], [ %15, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i" ]
  %iter.sroa.84.08.i = phi i16 [ %_23.i.i, %bb2.i ], [ %_33.i.i, %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i" ]
  %.not14.i.i = icmp eq i16 %iter.sroa.84.08.i, 0, !dbg !598
  br i1 %.not14.i.i, label %bb9.i.i, label %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i", !dbg !602

bb1.bb8_crit_edge.i.i:                            ; preds = %bb9.i.i
  %_55.i.i = xor i16 %10, -1, !dbg !603
  br label %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i", !dbg !602

bb9.i.i:                                          ; preds = %bb14.i, %bb9.i.i
  %_1717.i.i = phi ptr [ %_17.i.i, %bb9.i.i ], [ %iter.sroa.6.010.i, %bb14.i ], !dbg !606
  %7 = phi ptr [ %11, %bb9.i.i ], [ %iter.sroa.0.011.i, %bb14.i ]
  %8 = load <16 x i8>, ptr %_1717.i.i, align 16, !dbg !607, !noalias !610
  %9 = icmp slt <16 x i8> %8, zeroinitializer, !dbg !615
  %10 = bitcast <16 x i1> %9 to i16, !dbg !615
  %11 = getelementptr inbounds i8, ptr %7, i64 -768, !dbg !618
  %_17.i.i = getelementptr inbounds nuw i8, ptr %_1717.i.i, i64 16, !dbg !621
  %.not.i.i = icmp eq i16 %10, -1, !dbg !598
  br i1 %.not.i.i, label %bb9.i.i, label %bb1.bb8_crit_edge.i.i, !dbg !602

"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i": ; preds = %bb1.bb8_crit_edge.i.i, %bb14.i
  %iter.sroa.6.1.i = phi ptr [ %_17.i.i, %bb1.bb8_crit_edge.i.i ], [ %iter.sroa.6.010.i, %bb14.i ], !dbg !623
  %iter.sroa.0.1.i = phi ptr [ %11, %bb1.bb8_crit_edge.i.i ], [ %iter.sroa.0.011.i, %bb14.i ], !dbg !623
  %self3.lcssa.i.i = phi i16 [ %_55.i.i, %bb1.bb8_crit_edge.i.i ], [ %iter.sroa.84.08.i, %bb14.i ], !dbg !624
  %12 = add i16 %self3.lcssa.i.i, -1, !dbg !625
  %13 = tail call range(i16 0, 17) i16 @llvm.cttz.i16(i16 %self3.lcssa.i.i, i1 true), !dbg !629
  %_24.i.i = zext nneg i16 %13 to i64, !dbg !630
  %_33.i.i = and i16 %12, %self3.lcssa.i.i, !dbg !631
  %_45.i.i = sub nsw i64 0, %_24.i.i, !dbg !633
  %14 = getelementptr inbounds { { ptr, i64 }, { i64, %"alloc::vec::Vec<(&[u8], usize)>" } }, ptr %iter.sroa.0.1.i, i64 %_45.i.i, !dbg !636
  %15 = add i64 %iter.sroa.115.09.i, -1, !dbg !637
  %16 = getelementptr inbounds i8, ptr %14, i64 -24, !dbg !638
; call alloc::raw_vec::RawVecInner<A>::deallocate
  tail call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h1ab787cdb4e63fb3E"(ptr noalias noundef nonnull align 8 dereferenceable(24) %16, i64 noundef 8, i64 noundef 24) #16, !dbg !643, !noalias !570
  %17 = icmp eq i64 %15, 0, !dbg !596
  br i1 %17, label %_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E.exit, label %bb14.i, !dbg !596

_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E.exit: ; preds = %"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E.exit.i", %bb2
  %_9 = add i64 %_5, 1, !dbg !647
  %18 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %table_layout.0, i64 %_9), !dbg !654
  %_25.1.i = extractvalue { i64, i1 } %18, 1, !dbg !654
  br i1 %_25.1.i, label %_ZN9hashbrown3raw11TableLayout20calculate_layout_for17he05461673a20bd68E.exit, label %bb8.i, !dbg !665, !prof !671

bb8.i:                                            ; preds = %_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E.exit
  %_25.0.i = extractvalue { i64, i1 } %18, 0, !dbg !654
  %rhs.i = add i64 %table_layout.1, -1, !dbg !672
  %_32.0.i = add i64 %_25.0.i, %rhs.i, !dbg !673
  %_32.1.i = icmp ult i64 %_32.0.i, %_25.0.i, !dbg !673
  br i1 %_32.1.i, label %_ZN9hashbrown3raw11TableLayout20calculate_layout_for17he05461673a20bd68E.exit, label %bb11.i, !dbg !676, !prof !671

bb11.i:                                           ; preds = %bb8.i
  %_14.i = sub i64 0, %table_layout.1, !dbg !678
  %ctrl_offset.i = and i64 %_32.0.i, %_14.i, !dbg !679
  %rhs5.i = add i64 %_5, 17, !dbg !680
  %_37.0.i = add i64 %rhs5.i, %ctrl_offset.i, !dbg !682
  %_37.1.i = icmp ult i64 %_37.0.i, %ctrl_offset.i, !dbg !682
  %_20.i = sub i64 -9223372036854775808, %table_layout.1
  %_19.i = icmp ugt i64 %_37.0.i, %_20.i
  %or.cond = or i1 %_37.1.i, %_19.i, !dbg !684
  br i1 %or.cond, label %_ZN9hashbrown3raw11TableLayout20calculate_layout_for17he05461673a20bd68E.exit, label %bb2.i3, !dbg !684, !prof !687

bb2.i3:                                           ; preds = %bb11.i
  %19 = icmp sgt i64 %rhs.i, -1, !dbg !688
  tail call void @llvm.assume(i1 %19), !dbg !688
  br label %_ZN9hashbrown3raw11TableLayout20calculate_layout_for17he05461673a20bd68E.exit, !dbg !696

_ZN9hashbrown3raw11TableLayout20calculate_layout_for17he05461673a20bd68E.exit: ; preds = %_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E.exit, %bb8.i, %bb11.i, %bb2.i3
  %_8.sroa.9.0 = phi i64 [ %ctrl_offset.i, %bb2.i3 ], [ undef, %bb11.i ], [ undef, %bb8.i ], [ undef, %_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E.exit ]
  %_8.sroa.7.0 = phi i64 [ %_37.0.i, %bb2.i3 ], [ undef, %bb11.i ], [ undef, %bb8.i ], [ undef, %_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E.exit ]
  %_8.sroa.0.0 = phi i64 [ %table_layout.1, %bb2.i3 ], [ 0, %bb11.i ], [ 0, %bb8.i ], [ 0, %_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E.exit ], !dbg !697
  %20 = icmp ne i64 %_8.sroa.0.0, 0, !dbg !698
  tail call void @llvm.assume(i1 %20), !dbg !699
  %21 = icmp eq i64 %_8.sroa.7.0, 0, !dbg !700
  br i1 %21, label %bb4, label %bb1.i4, !dbg !700

bb1.i4:                                           ; preds = %_ZN9hashbrown3raw11TableLayout20calculate_layout_for17he05461673a20bd68E.exit
  %self1 = load ptr, ptr %self, align 8, !dbg !712, !nonnull !14, !noundef !14
  %_17 = sub nsw i64 0, %_8.sroa.9.0, !dbg !714
  %ptr = getelementptr inbounds i8, ptr %self1, i64 %_17, !dbg !717
; call __rustc::__rust_dealloc
  tail call void @_RNvCs1Y7DaGC1cwg_7___rustc14___rust_dealloc(ptr noundef nonnull %ptr, i64 noundef %_8.sroa.7.0, i64 noundef range(i64 1, -9223372036854775807) %_8.sroa.0.0) #16, !dbg !718
  br label %bb4, !dbg !721

bb4:                                              ; preds = %bb1.i4, %_ZN9hashbrown3raw11TableLayout20calculate_layout_for17he05461673a20bd68E.exit, %start
  ret void, !dbg !722
}

; hashbrown::raw::RawIterRange<T>::new
; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: readwrite) uwtable
define void @"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17hfd5cca0e9bf5885fE"(ptr dead_on_unwind noalias noundef writable writeonly sret([32 x i8]) align 8 captures(none) dereferenceable(32) initializes((0, 26)) %_0, ptr noundef %ctrl, ptr noundef nonnull %data, i64 noundef %len) unnamed_addr #2 !dbg !447 {
start:
  %end = getelementptr inbounds nuw i8, ptr %ctrl, i64 %len, !dbg !723
  %0 = load <16 x i8>, ptr %ctrl, align 16, !dbg !725, !noalias !728
  %1 = icmp slt <16 x i8> %0, zeroinitializer, !dbg !731
  %2 = bitcast <16 x i1> %1 to i16, !dbg !731
  %_23 = xor i16 %2, -1, !dbg !735
  %next_ctrl = getelementptr inbounds nuw i8, ptr %ctrl, i64 16, !dbg !737
  %3 = getelementptr inbounds nuw i8, ptr %_0, i64 24, !dbg !739
  store i16 %_23, ptr %3, align 8, !dbg !739
  store ptr %data, ptr %_0, align 8, !dbg !739
  %4 = getelementptr inbounds nuw i8, ptr %_0, i64 8, !dbg !739
  store ptr %next_ctrl, ptr %4, align 8, !dbg !739
  %5 = getelementptr inbounds nuw i8, ptr %_0, i64 16, !dbg !739
  store ptr %end, ptr %5, align 8, !dbg !739
  ret void, !dbg !741
}

; hashbrown::raw::RawIterRange<T>::next_impl
; Function Attrs: nofree norecurse nosync nounwind nonlazybind memory(read, argmem: readwrite, inaccessiblemem: none) uwtable
define noundef nonnull ptr @"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E"(ptr noalias noundef align 8 captures(none) dereferenceable(32) %self) unnamed_addr #6 !dbg !491 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i64 24
  %.promoted = load i16, ptr %0, align 8
  %.not14 = icmp eq i16 %.promoted, 0, !dbg !742
  br i1 %.not14, label %bb9.lr.ph, label %bb8, !dbg !745

bb9.lr.ph:                                        ; preds = %start
  %self.promoted = load ptr, ptr %self, align 8
  %1 = getelementptr inbounds nuw i8, ptr %self, i64 8
  %.promoted16 = load ptr, ptr %1, align 8
  br label %bb9, !dbg !745

bb1.bb8_crit_edge:                                ; preds = %bb9
  %_55 = xor i16 %8, -1, !dbg !746
  store ptr %_17, ptr %1, align 8, !dbg !749
  store i16 %_55, ptr %0, align 8, !dbg !750
  store ptr %9, ptr %self, align 8, !dbg !751
  br label %bb8, !dbg !745

bb8:                                              ; preds = %bb1.bb8_crit_edge, %start
  %self3.lcssa = phi i16 [ %_55, %bb1.bb8_crit_edge ], [ %.promoted, %start ], !dbg !752
  %2 = add i16 %self3.lcssa, -1, !dbg !753
  %3 = tail call range(i16 0, 17) i16 @llvm.cttz.i16(i16 %self3.lcssa, i1 true), !dbg !757
  %_24 = zext nneg i16 %3 to i64, !dbg !758
  %_33 = and i16 %2, %self3.lcssa, !dbg !759
  store i16 %_33, ptr %0, align 8, !dbg !761
  %self5 = load ptr, ptr %self, align 8, !dbg !762, !nonnull !14, !noundef !14
  %_45 = sub nsw i64 0, %_24, !dbg !764
  %4 = getelementptr inbounds { { ptr, i64 }, { i64, %"alloc::vec::Vec<(&[u8], usize)>" } }, ptr %self5, i64 %_45, !dbg !766
  ret ptr %4, !dbg !767

bb9:                                              ; preds = %bb9.lr.ph, %bb9
  %_1717 = phi ptr [ %.promoted16, %bb9.lr.ph ], [ %_17, %bb9 ], !dbg !768
  %5 = phi ptr [ %self.promoted, %bb9.lr.ph ], [ %9, %bb9 ]
  %6 = load <16 x i8>, ptr %_1717, align 16, !dbg !769, !noalias !772
  %7 = icmp slt <16 x i8> %6, zeroinitializer, !dbg !775
  %8 = bitcast <16 x i1> %7 to i16, !dbg !775
  %9 = getelementptr inbounds i8, ptr %5, i64 -768, !dbg !778
  %_17 = getelementptr inbounds nuw i8, ptr %_1717, i64 16, !dbg !781
  %.not = icmp eq i16 %8, -1, !dbg !742
  br i1 %.not, label %bb9, label %bb1.bb8_crit_edge, !dbg !745
}

; Function Attrs: nounwind nonlazybind uwtable
define noundef zeroext i1 @phage_target(i8 noundef %a, i8 noundef %b, i8 noundef %c, i8 noundef %d, i8 noundef %e, i8 noundef %f, i8 noundef %g, i8 noundef %h, i8 noundef %i, i8 noundef %j, i8 noundef %k, i8 noundef %l, i8 noundef %len, i8 noundef %prefix) unnamed_addr #0 personality ptr @rust_eh_personality !dbg !783 {
start:
  %data = alloca [12 x i8], align 1
  %_15 = icmp ugt i8 %len, 12, !dbg !785
  %0 = add i8 %prefix, -9, !dbg !785
  %1 = icmp ult i8 %0, -8, !dbg !785
  %or.cond1 = or i1 %_15, %1, !dbg !785
  br i1 %or.cond1, label %bb19, label %bb4, !dbg !785

bb4:                                              ; preds = %start
  call void @llvm.lifetime.start.p0(i64 12, ptr nonnull %data), !dbg !786
  store i8 %a, ptr %data, align 1, !dbg !787
  %2 = getelementptr inbounds nuw i8, ptr %data, i64 1, !dbg !787
  store i8 %b, ptr %2, align 1, !dbg !787
  %3 = getelementptr inbounds nuw i8, ptr %data, i64 2, !dbg !787
  store i8 %c, ptr %3, align 1, !dbg !787
  %4 = getelementptr inbounds nuw i8, ptr %data, i64 3, !dbg !787
  store i8 %d, ptr %4, align 1, !dbg !787
  %5 = getelementptr inbounds nuw i8, ptr %data, i64 4, !dbg !787
  store i8 %e, ptr %5, align 1, !dbg !787
  %6 = getelementptr inbounds nuw i8, ptr %data, i64 5, !dbg !787
  store i8 %f, ptr %6, align 1, !dbg !787
  %7 = getelementptr inbounds nuw i8, ptr %data, i64 6, !dbg !787
  store i8 %g, ptr %7, align 1, !dbg !787
  %8 = getelementptr inbounds nuw i8, ptr %data, i64 7, !dbg !787
  store i8 %h, ptr %8, align 1, !dbg !787
  %9 = getelementptr inbounds nuw i8, ptr %data, i64 8, !dbg !787
  store i8 %i, ptr %9, align 1, !dbg !787
  %10 = getelementptr inbounds nuw i8, ptr %data, i64 9, !dbg !787
  store i8 %j, ptr %10, align 1, !dbg !787
  %11 = getelementptr inbounds nuw i8, ptr %data, i64 10, !dbg !787
  store i8 %k, ptr %11, align 1, !dbg !787
  %12 = getelementptr inbounds nuw i8, ptr %data, i64 11, !dbg !787
  store i8 %l, ptr %12, align 1, !dbg !787
  %_0.i2 = zext nneg i8 %len to i64, !dbg !788
  tail call void @llvm.experimental.noalias.scope.decl(metadata !796), !dbg !799
  %_3.not.i.not.not.i = icmp eq i8 %len, 0, !dbg !800
  %alloc_18ff8af99e51a98d72b531fe691d1961..self.0.i.i = select i1 %_3.not.i.not.not.i, ptr @alloc_18ff8af99e51a98d72b531fe691d1961, ptr %data, !dbg !806
  br i1 %_3.not.i.not.not.i, label %bb6.i, label %bb5.i, !dbg !812

bb6.i:                                            ; preds = %bb4
  %13 = ptrtoint ptr %alloc_18ff8af99e51a98d72b531fe691d1961..self.0.i.i to i64, !dbg !813
  br label %_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit, !dbg !822

bb5.i:                                            ; preds = %bb4
  %first.i = load i8, ptr %alloc_18ff8af99e51a98d72b531fe691d1961..self.0.i.i, align 1, !dbg !824, !noalias !825, !noundef !14
  %14 = zext nneg i8 %prefix to i64, !dbg !827
  %notmask.i = shl nsw i64 -1, %14, !dbg !827
  %_14.0.i = xor i64 %notmask.i, -1, !dbg !827
  %_0.i16.i = zext i8 %first.i to i64, !dbg !829
  %15 = and i64 %_0.i16.i, %_14.0.i, !dbg !834
  %_17.not.i = icmp eq i64 %15, %_14.0.i, !dbg !835
  br i1 %_17.not.i, label %"_ZN110_$LT$core..ops..range..RangeFrom$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17hf5bc6849bc651b13E.exit.i", label %_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit, !dbg !835

"_ZN110_$LT$core..ops..range..RangeFrom$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17hf5bc6849bc651b13E.exit.i": ; preds = %bb5.i
  %_6.i.i.i = getelementptr i8, ptr %data, i64 %_0.i2, !dbg !837
  br label %bb17.i, !dbg !847

bb17.i:                                           ; preds = %bb25.i, %"_ZN110_$LT$core..ops..range..RangeFrom$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17hf5bc6849bc651b13E.exit.i"
  %indvars.iv.i = phi i64 [ %indvars.iv.next.i, %bb25.i ], [ 0, %"_ZN110_$LT$core..ops..range..RangeFrom$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17hf5bc6849bc651b13E.exit.i" ], !dbg !849
  %iter.sroa.0.0.i = phi ptr [ %spec.select.idx.i.sroa.sel.idx.sroa.sel, %bb25.i ], [ %2, %"_ZN110_$LT$core..ops..range..RangeFrom$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17hf5bc6849bc651b13E.exit.i" ], !dbg !849
  %iter.sroa.8.0.i = phi i64 [ %iter.sroa.8.1.i, %bb25.i ], [ 0, %"_ZN110_$LT$core..ops..range..RangeFrom$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17hf5bc6849bc651b13E.exit.i" ], !dbg !849
  %value.sroa.0.0.i = phi i64 [ %_36.0.i, %bb25.i ], [ %15, %"_ZN110_$LT$core..ops..range..RangeFrom$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17hf5bc6849bc651b13E.exit.i" ], !dbg !850
  %_6.i.i20.i.not.not = icmp ne ptr %iter.sroa.0.0.i, %_6.i.i.i, !dbg !851
  %spec.select.idx.i.sroa.sel.idx.sroa.sel.idx = zext i1 %_6.i.i20.i.not.not to i64, !dbg !865
  %spec.select.idx.i.sroa.sel.idx.sroa.sel = getelementptr inbounds nuw i8, ptr %iter.sroa.0.0.i, i64 %spec.select.idx.i.sroa.sel.idx.sroa.sel.idx, !dbg !865
  %spec.select26.i = select i1 %_6.i.i20.i.not.not, ptr %iter.sroa.0.0.i, ptr null, !dbg !865
  %.not.i21.i = icmp eq ptr %spec.select26.i, null, !dbg !866
  br i1 %.not.i21.i, label %"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE.exit.i", label %bb8.i.i, !dbg !870

bb8.i.i:                                          ; preds = %bb17.i
  %_8.1.i.i = icmp eq i64 %iter.sroa.8.0.i, -1, !dbg !871
  br i1 %_8.1.i.i, label %panic.i.i, label %bb3.i22.i, !dbg !871

bb3.i22.i:                                        ; preds = %bb8.i.i
  %_8.0.i.i = add nuw i64 %iter.sroa.8.0.i, 1, !dbg !871
  br label %"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE.exit.i", !dbg !874

panic.i.i:                                        ; preds = %bb8.i.i
; call core::panicking::panic_const::panic_const_add_overflow
  call void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h26fd19dc5797cca9E(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24) @alloc_aeb2f4e6771f847856ffd3202edf9987) #19, !dbg !871, !noalias !875
  unreachable, !dbg !871

"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE.exit.i": ; preds = %bb3.i22.i, %bb17.i
  %iter.sroa.8.1.i = phi i64 [ %iter.sroa.8.0.i, %bb17.i ], [ %_8.0.i.i, %bb3.i22.i ], !dbg !849
  %_0.sroa.2.0.i.i = phi ptr [ null, %bb17.i ], [ %spec.select26.i, %bb3.i22.i ], !dbg !878
  %_0.sroa.0.0.i.i = phi i64 [ undef, %bb17.i ], [ %iter.sroa.8.0.i, %bb3.i22.i ]
  %.not15.i = icmp eq ptr %_0.sroa.2.0.i.i, null, !dbg !879
  br i1 %.not15.i, label %_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit, label %bb19.i, !dbg !879

bb19.i:                                           ; preds = %"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE.exit.i"
  %byte.i = load i8, ptr %_0.sroa.2.0.i.i, align 1, !dbg !880, !alias.scope !796, !noalias !825, !noundef !14
  %_35.i = icmp samesign ult i64 %indvars.iv.i, 64, !dbg !881
  br i1 %_35.i, label %bb22.i, label %panic2.i, !dbg !881

bb22.i:                                           ; preds = %bb19.i
  %_33.i = and i8 %byte.i, 127, !dbg !883
  %_0.i.i = zext nneg i8 %_33.i to i64, !dbg !884
  %_31.i = shl i64 %_0.i.i, %indvars.iv.i, !dbg !881
  %_36.0.i = add i64 %_31.i, %value.sroa.0.0.i, !dbg !886
  %_36.1.i = icmp ult i64 %_36.0.i, %value.sroa.0.0.i, !dbg !886
  br i1 %_36.1.i, label %panic3.i, label %bb23.i, !dbg !886

panic2.i:                                         ; preds = %bb19.i
; call core::panicking::panic_const::panic_const_shl_overflow
  call void @_ZN4core9panicking11panic_const24panic_const_shl_overflow17hc69eab5018f16ee8E(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24) @alloc_db0d3c4eb7c152921f9afd9fce696e08) #19, !dbg !881, !noalias !887
  unreachable, !dbg !881

bb23.i:                                           ; preds = %bb22.i
  %_37.i = icmp ugt i64 %_36.0.i, 4294967295, !dbg !888
  br i1 %_37.i, label %_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit, label %bb25.i, !dbg !888

panic3.i:                                         ; preds = %bb22.i
; call core::panicking::panic_const::panic_const_add_overflow
  call void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h26fd19dc5797cca9E(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24) @alloc_02127657331c02dbcc0a53a744076139) #19, !dbg !886, !noalias !887
  unreachable, !dbg !886

bb25.i:                                           ; preds = %bb23.i
  %16 = icmp sgt i8 %byte.i, -1, !dbg !889
  %indvars.iv.next.i = add nuw nsw i64 %indvars.iv.i, 7, !dbg !890
  br i1 %16, label %bb26.i, label %bb17.i, !dbg !889

bb26.i:                                           ; preds = %bb25.i
  %_43.1.i = icmp ugt i64 %_0.sroa.0.0.i.i, -3, !dbg !891
  br i1 %_43.1.i, label %panic4.i, label %bb28.i, !dbg !891

bb28.i:                                           ; preds = %bb26.i
  %17 = add i64 %_0.sroa.0.0.i.i, 1, !dbg !892
  br label %_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit, !dbg !894

panic4.i:                                         ; preds = %bb26.i
; call core::panicking::panic_const::panic_const_add_overflow
  call void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h26fd19dc5797cca9E(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24) @alloc_b9378ea015704e9df6c0470e86732e03) #19, !dbg !891, !noalias !887
  unreachable, !dbg !891

_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit: ; preds = %bb23.i, %"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE.exit.i", %bb5.i, %bb6.i, %bb28.i
  %_19.sroa.14.0 = phi i64 [ 16, %bb6.i ], [ %17, %bb28.i ], [ 0, %bb5.i ], [ 16, %"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE.exit.i" ], [ 16, %bb23.i ], !dbg !896
  %_19.sroa.8.0 = phi i64 [ %13, %bb6.i ], [ %_36.0.i, %bb28.i ], [ %15, %bb5.i ], [ ptrtoint (ptr @alloc_ea701fff40f262979702ba2c66956fda to i64), %bb23.i ], [ ptrtoint (ptr @alloc_18ff8af99e51a98d72b531fe691d1961 to i64), %"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE.exit.i" ], !dbg !896
  %.not = phi i1 [ false, %bb6.i ], [ true, %bb28.i ], [ true, %bb5.i ], [ false, %"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE.exit.i" ], [ false, %bb23.i ], !dbg !896
  br i1 %.not, label %bb10, label %bb18, !dbg !897

bb10:                                             ; preds = %_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit
  %or.cond.not = icmp ult i64 %_19.sroa.14.0, %_0.i2, !dbg !892
  %18 = icmp ult i64 %_19.sroa.8.0, 4294967296
  %spec.select = and i1 %or.cond.not, %18, !dbg !892
  br label %bb18, !dbg !892

bb18:                                             ; preds = %bb10, %_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit
  %_0.sroa.0.0 = phi i1 [ true, %_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE.exit ], [ %spec.select, %bb10 ], !dbg !898
  call void @llvm.lifetime.end.p0(i64 12, ptr nonnull %data), !dbg !899
  br label %bb19, !dbg !900

bb19:                                             ; preds = %start, %bb18
  %_0.sroa.0.1 = phi i1 [ %_0.sroa.0.0, %bb18 ], [ true, %start ], !dbg !901
  ret i1 %_0.sroa.0.1, !dbg !900
}

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.start.p0(i64 immarg, ptr captures(none)) #7

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.end.p0(i64 immarg, ptr captures(none)) #7

; Function Attrs: nounwind nonlazybind uwtable
declare noundef range(i32 0, 10) i32 @rust_eh_personality(i32 noundef, i32 noundef, i64 noundef, ptr noundef, ptr noundef) unnamed_addr #0

; core::panicking::panic_const::panic_const_add_overflow
; Function Attrs: cold noinline noreturn nounwind nonlazybind uwtable
declare void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h26fd19dc5797cca9E(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24)) unnamed_addr #8

; Function Attrs: mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i64, i1 immarg) #9

; core::panicking::panic_const::panic_const_shl_overflow
; Function Attrs: cold noinline noreturn nounwind nonlazybind uwtable
declare void @_ZN4core9panicking11panic_const24panic_const_shl_overflow17hc69eab5018f16ee8E(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(24)) unnamed_addr #8

; <&T as core::fmt::Display>::fmt
; Function Attrs: nounwind nonlazybind uwtable
declare noundef zeroext i1 @"_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h28908b04add44bbcE"(ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(16), ptr noalias noundef align 8 dereferenceable(24)) unnamed_addr #0

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(inaccessiblemem: write)
declare void @llvm.assume(i1 noundef) #10

; core::fmt::write
; Function Attrs: nounwind nonlazybind uwtable
declare noundef zeroext i1 @_ZN4core3fmt5write17h469843d235cc4241E(ptr noundef nonnull align 1, ptr noalias noundef readonly align 8 captures(address, read_provenance) dereferenceable(48), ptr noundef nonnull, ptr noundef nonnull) unnamed_addr #0

; alloc::vec::Vec<T,A>::reserve
; Function Attrs: nounwind nonlazybind uwtable
declare void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h2f692e8ba7e2c189E"(ptr noalias noundef align 8 dereferenceable(24), i64 noundef) unnamed_addr #0

; alloc::raw_vec::RawVecInner<A>::grow_amortized
; Function Attrs: nounwind nonlazybind uwtable
declare { i64, i64 } @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$14grow_amortized17h7368fa7136fd38e3E"(ptr noalias noundef align 8 dereferenceable(16), i64 noundef, i64 noundef, i64 noundef range(i64 1, -9223372036854775807), i64 noundef) unnamed_addr #0

; alloc::raw_vec::handle_error
; Function Attrs: cold minsize noreturn nounwind nonlazybind optsize uwtable
declare void @_ZN5alloc7raw_vec12handle_error17hfa86a3a4628bd209E(i64 noundef range(i64 0, -9223372036854775807), i64) unnamed_addr #11

; __rustc::__rust_dealloc
; Function Attrs: nounwind nonlazybind allockind("free") uwtable
declare void @_RNvCs1Y7DaGC1cwg_7___rustc14___rust_dealloc(ptr allocptr noundef, i64 noundef, i64 noundef) unnamed_addr #12

; alloc::raw_vec::RawVecInner<A>::deallocate
; Function Attrs: nounwind nonlazybind uwtable
declare void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h1ab787cdb4e63fb3E"(ptr noalias noundef align 8 dereferenceable(16), i64 noundef range(i64 1, -9223372036854775807), i64 noundef) unnamed_addr #0

; core::fmt::Formatter::write_str
; Function Attrs: nounwind nonlazybind uwtable
declare noundef zeroext i1 @_ZN4core3fmt9Formatter9write_str17h72189eba35977850E(ptr noalias noundef align 8 dereferenceable(24), ptr noalias noundef nonnull readonly align 1 captures(address, read_provenance), i64 noundef) unnamed_addr #0

; Function Attrs: mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i64, i1 } @llvm.umul.with.overflow.i64(i64, i64) #13

; Function Attrs: mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i16 @llvm.cttz.i16(i16, i1 immarg) #13

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(inaccessiblemem: readwrite)
declare void @llvm.experimental.noalias.scope.decl(metadata) #14

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr writeonly captures(none), i8, i64, i1 immarg) #15

attributes #0 = { nounwind nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #1 = { mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: read, inaccessiblemem: write) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #2 = { mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: readwrite) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #3 = { mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(argmem: write) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #4 = { noinline nounwind nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #5 = { mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(none) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #6 = { nofree norecurse nosync nounwind nonlazybind memory(read, argmem: readwrite, inaccessiblemem: none) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #7 = { mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #8 = { cold noinline noreturn nounwind nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #9 = { mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #10 = { mustprogress nocallback nofree nosync nounwind willreturn memory(inaccessiblemem: write) }
attributes #11 = { cold minsize noreturn nounwind nonlazybind optsize uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #12 = { nounwind nonlazybind allockind("free") uwtable "alloc-family"="__rust_alloc" "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #13 = { mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #14 = { nocallback nofree nosync nounwind willreturn memory(inaccessiblemem: readwrite) }
attributes #15 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #16 = { nounwind }
attributes #17 = { noinline nounwind }
attributes #18 = { noreturn nounwind }
attributes #19 = { noinline noreturn nounwind }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}
!llvm.dbg.cu = !{!5}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{i32 7, !"Dwarf Version", i32 4}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{!"rustc version 1.94.1 (e408947bf 2026-03-25)"}
!5 = distinct !DICompileUnit(language: DW_LANG_Rust, file: !6, producer: "clang LLVM (rustc version 1.94.1 (e408947bf 2026-03-25))", isOptimized: true, runtimeVersion: 0, emissionKind: LineTablesOnly, splitDebugInlining: false, nameTableKind: None)
!6 = !DIFile(filename: "examples/nexagate-hpack-before.rs/@/nexagate_hpack_before.6f0f2380cbf4442f-cgu.0", directory: "/home/zero/Dev/Nexaaly/RuHealth")
!7 = distinct !DISubprogram(name: "spec_extend<u8, alloc::alloc::Global>", linkageName: "_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h9308aadac9002e8dE", scope: !9, file: !8, line: 54, type: !13, scopeLine: 54, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!8 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/spec_extend.rs", directory: "", checksumkind: CSK_MD5, checksum: "8aecc734f5ea30edf964e2d8611b8989")
!9 = !DINamespace(name: "{impl#4}", scope: !10)
!10 = !DINamespace(name: "spec_extend", scope: !11)
!11 = !DINamespace(name: "vec", scope: !12)
!12 = !DINamespace(name: "alloc", scope: null)
!13 = !DISubroutineType(types: !14)
!14 = !{}
!15 = !DILocation(line: 729, column: 18, scope: !16, inlinedAt: !23)
!16 = distinct !DILexicalBlock(scope: !18, file: !17, line: 726, column: 9)
!17 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs", directory: "", checksumkind: CSK_MD5, checksum: "d63b2a314425324f87d3f30449c90689")
!18 = distinct !DISubprogram(name: "offset_from_unsigned<u8>", linkageName: "_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$20offset_from_unsigned17h8c72f441ccc25c69E", scope: !19, file: !17, line: 701, type: !13, scopeLine: 701, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!19 = !DINamespace(name: "{impl#0}", scope: !20)
!20 = !DINamespace(name: "const_ptr", scope: !21)
!21 = !DINamespace(name: "ptr", scope: !22)
!22 = !DINamespace(name: "core", scope: null)
!23 = distinct !DILocation(line: 887, column: 37, scope: !24, inlinedAt: !28)
!24 = distinct !DISubprogram(name: "offset_from_unsigned<u8>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$20offset_from_unsigned17h25e5aa57e7745aefE", scope: !26, file: !25, line: 882, type: !13, scopeLine: 882, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!25 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs", directory: "", checksumkind: CSK_MD5, checksum: "741f53a1b46ef31ed27a8da0b10d1ba5")
!26 = !DINamespace(name: "{impl#0}", scope: !27)
!27 = !DINamespace(name: "mut_ptr", scope: !21)
!28 = distinct !DILocation(line: 954, column: 32, scope: !29, inlinedAt: !33)
!29 = distinct !DISubprogram(name: "offset_from_unsigned<u8>", linkageName: "_ZN4core3ptr8non_null16NonNull$LT$T$GT$20offset_from_unsigned17ha383deb0699d163aE", scope: !31, file: !30, line: 949, type: !13, scopeLine: 949, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!30 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs", directory: "", checksumkind: CSK_MD5, checksum: "b9319c401ce0dd8d5662df7fdc4fc942")
!31 = !DINamespace(name: "NonNull", scope: !32)
!32 = !DINamespace(name: "non_null", scope: !21)
!33 = distinct !DILocation(line: 57, column: 30, scope: !34, inlinedAt: !41)
!34 = distinct !DILexicalBlock(scope: !36, file: !35, line: 33, column: 13)
!35 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs", directory: "", checksumkind: CSK_MD5, checksum: "87d1f0c2746f51593d75ddf4c9271f14")
!36 = distinct !DILexicalBlock(scope: !37, file: !35, line: 25, column: 86)
!37 = distinct !DISubprogram(name: "make_slice<u8>", linkageName: "_ZN4core5slice4iter13Iter$LT$T$GT$10make_slice17h340982bb44e50481E", scope: !38, file: !35, line: 89, type: !13, scopeLine: 89, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!38 = !DINamespace(name: "Iter", scope: !39)
!39 = !DINamespace(name: "iter", scope: !40)
!40 = !DINamespace(name: "slice", scope: !22)
!41 = distinct !DILocation(line: 138, column: 14, scope: !42, inlinedAt: !44)
!42 = distinct !DISubprogram(name: "as_slice<u8>", linkageName: "_ZN4core5slice4iter13Iter$LT$T$GT$8as_slice17h63fdd79f79d4d331E", scope: !38, file: !43, line: 137, type: !13, scopeLine: 137, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!43 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs", directory: "", checksumkind: CSK_MD5, checksum: "f33ab2e22fe09095bf73c41c52bd166c")
!44 = !DILocation(line: 55, column: 30, scope: !7)
!45 = !DILocation(line: 2819, column: 14, scope: !46, inlinedAt: !50)
!46 = distinct !DILexicalBlock(scope: !48, file: !47, line: 2818, column: 9)
!47 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "d1270e596beca07970ab7bfe964f7ed5")
!48 = distinct !DISubprogram(name: "append_elements<u8, alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE", scope: !49, file: !47, line: 2817, type: !13, scopeLine: 2817, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!49 = !DINamespace(name: "Vec", scope: !11)
!50 = distinct !DILocation(line: 56, column: 23, scope: !51)
!51 = distinct !DILexicalBlock(scope: !7, file: !8, line: 55, column: 9)
!52 = !DILocation(line: 2933, column: 19, scope: !53, inlinedAt: !54)
!53 = distinct !DISubprogram(name: "len<u8, alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h8e8d467b23b8341cE", scope: !49, file: !47, line: 2932, type: !13, scopeLine: 2932, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!54 = distinct !DILocation(line: 2820, column: 24, scope: !46, inlinedAt: !50)
!55 = !{!56}
!56 = distinct !{!56, !57, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE: %self"}
!57 = distinct !{!57, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE"}
!58 = !DILocation(line: 2938, column: 37, scope: !59, inlinedAt: !54)
!59 = distinct !DILexicalBlock(scope: !53, file: !47, line: 2933, column: 9)
!60 = !DILocation(line: 2938, column: 18, scope: !59, inlinedAt: !54)
!61 = !DILocation(line: 597, column: 9, scope: !62, inlinedAt: !66)
!62 = distinct !DISubprogram(name: "non_null<alloc::alloc::Global, u8>", linkageName: "_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$8non_null17hed74395d28878007E", scope: !64, file: !63, line: 596, type: !13, scopeLine: 596, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!63 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "7b1e7af9755a3a9c6823f660d5f1b0ef")
!64 = !DINamespace(name: "RawVecInner", scope: !65)
!65 = !DINamespace(name: "raw_vec", scope: !12)
!66 = distinct !DILocation(line: 592, column: 14, scope: !67, inlinedAt: !68)
!67 = distinct !DISubprogram(name: "ptr<alloc::alloc::Global, u8>", linkageName: "_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$3ptr17h5aedaf1d87482799E", scope: !64, file: !63, line: 591, type: !13, scopeLine: 591, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!68 = distinct !DILocation(line: 295, column: 20, scope: !69, inlinedAt: !71)
!69 = distinct !DISubprogram(name: "ptr<u8, alloc::alloc::Global>", linkageName: "_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$3ptr17hf1f85788285d0755E", scope: !70, file: !63, line: 294, type: !13, scopeLine: 294, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!70 = !DINamespace(name: "RawVec", scope: !65)
!71 = distinct !DILocation(line: 1938, column: 18, scope: !72, inlinedAt: !73)
!72 = distinct !DISubprogram(name: "as_mut_ptr<u8, alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$10as_mut_ptr17h9d246dae164fcad9E", scope: !49, file: !47, line: 1935, type: !13, scopeLine: 1935, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!73 = distinct !DILocation(line: 2821, column: 67, scope: !74, inlinedAt: !50)
!74 = distinct !DILexicalBlock(scope: !46, file: !47, line: 2820, column: 9)
!75 = !DILocation(line: 961, column: 18, scope: !76, inlinedAt: !77)
!76 = distinct !DISubprogram(name: "add<u8>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$3add17hf84a61f6dba8a914E", scope: !26, file: !25, line: 927, type: !13, scopeLine: 927, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!77 = distinct !DILocation(line: 2821, column: 80, scope: !74, inlinedAt: !50)
!78 = !DILocation(line: 547, column: 14, scope: !79, inlinedAt: !81)
!79 = distinct !DISubprogram(name: "copy_nonoverlapping<u8>", linkageName: "_ZN4core3ptr19copy_nonoverlapping17h9dc9441742f3a0f3E", scope: !21, file: !80, line: 526, type: !13, scopeLine: 526, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!80 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "f351b8102469924b31844bc9d588a289")
!81 = distinct !DILocation(line: 2821, column: 18, scope: !74, inlinedAt: !50)
!82 = !DILocation(line: 2822, column: 9, scope: !74, inlinedAt: !50)
!83 = !DILocation(line: 57, column: 6, scope: !7)
!84 = distinct !DISubprogram(name: "len", linkageName: "_ZN21nexagate_hpack_before5hpack7decoder9FieldList3len17h41e11c4b526fe49bE", scope: !86, file: !85, line: 29, type: !13, scopeLine: 29, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!85 = !DIFile(filename: "examples/../fixtures/nexagate-before/hpack/decoder.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "92caffbbadce8da982c09b90369a6ed7")
!86 = !DINamespace(name: "FieldList", scope: !87)
!87 = !DINamespace(name: "decoder", scope: !88)
!88 = !DINamespace(name: "hpack", scope: !89)
!89 = !DINamespace(name: "nexagate_hpack_before", scope: null)
!90 = !DILocation(line: 2933, column: 19, scope: !91, inlinedAt: !92)
!91 = distinct !DISubprogram(name: "len<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h1c0ab8907c89c339E", scope: !49, file: !47, line: 2932, type: !13, scopeLine: 2932, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!92 = distinct !DILocation(line: 30, column: 21, scope: !84)
!93 = !{!94}
!94 = distinct !{!94, !95, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h1c0ab8907c89c339E: %self"}
!95 = distinct !{!95, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h1c0ab8907c89c339E"}
!96 = !DILocation(line: 2938, column: 37, scope: !97, inlinedAt: !92)
!97 = distinct !DILexicalBlock(scope: !91, file: !47, line: 2933, column: 9)
!98 = !DILocation(line: 2938, column: 18, scope: !97, inlinedAt: !92)
!99 = !DILocation(line: 31, column: 6, scope: !84)
!100 = distinct !DISubprogram(name: "iter", linkageName: "_ZN21nexagate_hpack_before5hpack7decoder9FieldList4iter17hf9f9e9ede7b62fd9E", scope: !86, file: !85, line: 37, type: !13, scopeLine: 37, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!101 = !DILocation(line: 597, column: 9, scope: !102, inlinedAt: !103)
!102 = distinct !DISubprogram(name: "non_null<alloc::alloc::Global, (core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$8non_null17haaec5a5f57ac22e6E", scope: !64, file: !63, line: 596, type: !13, scopeLine: 596, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!103 = distinct !DILocation(line: 592, column: 14, scope: !104, inlinedAt: !105)
!104 = distinct !DISubprogram(name: "ptr<alloc::alloc::Global, (core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$3ptr17h7f5b7528b9bd8317E", scope: !64, file: !63, line: 591, type: !13, scopeLine: 591, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!105 = distinct !DILocation(line: 295, column: 20, scope: !106, inlinedAt: !107)
!106 = distinct !DISubprogram(name: "ptr<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$3ptr17h1a58635acae81850E", scope: !70, file: !63, line: 294, type: !13, scopeLine: 294, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!107 = distinct !DILocation(line: 1854, column: 18, scope: !108, inlinedAt: !109)
!108 = distinct !DISubprogram(name: "as_ptr<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$6as_ptr17h078096cc7b69894cE", scope: !49, file: !47, line: 1851, type: !13, scopeLine: 1851, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!109 = distinct !DILocation(line: 1753, column: 76, scope: !110, inlinedAt: !111)
!110 = distinct !DISubprogram(name: "as_slice<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$8as_slice17hbb471f605d207cf5E", scope: !49, file: !47, line: 1736, type: !13, scopeLine: 1736, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!111 = distinct !DILocation(line: 3665, column: 14, scope: !112, inlinedAt: !114)
!112 = distinct !DISubprogram(name: "deref<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN72_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h40a3eba6e6a332a6E", scope: !113, file: !47, line: 3664, type: !13, scopeLine: 3664, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!113 = !DINamespace(name: "{impl#9}", scope: !11)
!114 = distinct !DILocation(line: 38, column: 9, scope: !100)
!115 = !{!116}
!116 = distinct !{!116, !117, !"_ZN72_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h40a3eba6e6a332a6E: %self"}
!117 = distinct !{!117, !"_ZN72_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h40a3eba6e6a332a6E"}
!118 = !DILocation(line: 1753, column: 86, scope: !110, inlinedAt: !111)
!119 = !DILocation(line: 961, column: 18, scope: !120, inlinedAt: !121)
!120 = distinct !DISubprogram(name: "add<(core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$3add17hd98f55085f593490E", scope: !26, file: !25, line: 927, type: !13, scopeLine: 927, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!121 = distinct !DILocation(line: 102, column: 78, scope: !122, inlinedAt: !125)
!122 = distinct !DILexicalBlock(scope: !123, file: !43, line: 98, column: 9)
!123 = distinct !DILexicalBlock(scope: !124, file: !43, line: 97, column: 9)
!124 = distinct !DISubprogram(name: "new<(core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN4core5slice4iter13Iter$LT$T$GT$3new17h37288bde62b6487cE", scope: !38, file: !43, line: 96, type: !13, scopeLine: 96, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!125 = distinct !DILocation(line: 1041, column: 9, scope: !126, inlinedAt: !129)
!126 = distinct !DISubprogram(name: "iter<(core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN4core5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$4iter17h12c5253ed6ed921cE", scope: !128, file: !127, line: 1040, type: !13, scopeLine: 1040, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!127 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "bd4112d6e61cf6a64838143565da7e2d")
!128 = !DINamespace(name: "{impl#0}", scope: !40)
!129 = distinct !DILocation(line: 39, column: 14, scope: !100)
!130 = !DILocation(line: 69, column: 9, scope: !131, inlinedAt: !137)
!131 = distinct !DISubprogram(name: "new<core::slice::iter::Iter<(core::ops::range::Range<usize>, core::ops::range::Range<usize>)>, nexagate_hpack_before::hpack::decoder::{impl#0}::iter::{closure_env#0}>", linkageName: "_ZN4core4iter8adapters3map16Map$LT$I$C$F$GT$3new17ha2d6b15d5dcb6eaeE", scope: !133, file: !132, line: 68, type: !13, scopeLine: 68, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!132 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/map.rs", directory: "", checksumkind: CSK_MD5, checksum: "47fd4c3c8424e034238ec6bb5a169812")
!133 = !DINamespace(name: "Map", scope: !134)
!134 = !DINamespace(name: "map", scope: !135)
!135 = !DINamespace(name: "adapters", scope: !136)
!136 = !DINamespace(name: "iter", scope: !22)
!137 = distinct !DILocation(line: 782, column: 9, scope: !138, inlinedAt: !143)
!138 = distinct !DISubprogram(name: "map<core::slice::iter::Iter<(core::ops::range::Range<usize>, core::ops::range::Range<usize>)>, (&[u8], &[u8]), nexagate_hpack_before::hpack::decoder::{impl#0}::iter::{closure_env#0}>", linkageName: "_ZN4core4iter6traits8iterator8Iterator3map17hb36a225a59c20449E", scope: !140, file: !139, line: 777, type: !13, scopeLine: 777, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!139 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs", directory: "", checksumkind: CSK_MD5, checksum: "c3bbdf0ad1335db2ea48f33d906fc9a8")
!140 = !DINamespace(name: "Iterator", scope: !141)
!141 = !DINamespace(name: "iterator", scope: !142)
!142 = !DINamespace(name: "traits", scope: !136)
!143 = distinct !DILocation(line: 40, column: 14, scope: !100)
!144 = !{!145}
!145 = distinct !{!145, !146, !"_ZN4core4iter6traits8iterator8Iterator3map17hb36a225a59c20449E: %_0"}
!146 = distinct !{!146, !"_ZN4core4iter6traits8iterator8Iterator3map17hb36a225a59c20449E"}
!147 = !{!148}
!148 = distinct !{!148, !146, !"_ZN4core4iter6traits8iterator8Iterator3map17hb36a225a59c20449E: %f"}
!149 = !DILocation(line: 41, column: 6, scope: !100)
!150 = distinct !DISubprogram(name: "push", linkageName: "_ZN21nexagate_hpack_before5hpack7decoder9FieldList4push17h46cdc8bdfe629df2E", scope: !86, file: !85, line: 43, type: !13, scopeLine: 43, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!151 = !DILocation(line: 2933, column: 19, scope: !152, inlinedAt: !153)
!152 = distinct !DISubprogram(name: "len<u8, alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h8e8d467b23b8341cE", scope: !49, file: !47, line: 2932, type: !13, scopeLine: 2932, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!153 = distinct !DILocation(line: 44, column: 27, scope: !150)
!154 = !{!155}
!155 = distinct !{!155, !156, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h8e8d467b23b8341cE: %self"}
!156 = distinct !{!156, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h8e8d467b23b8341cE"}
!157 = !DILocation(line: 2938, column: 37, scope: !158, inlinedAt: !153)
!158 = distinct !DILexicalBlock(scope: !152, file: !47, line: 2933, column: 9)
!159 = !DILocation(line: 2938, column: 18, scope: !158, inlinedAt: !153)
!160 = !DILocation(line: 2819, column: 14, scope: !46, inlinedAt: !161)
!161 = distinct !DILocation(line: 56, column: 23, scope: !51, inlinedAt: !162)
!162 = distinct !DILocation(line: 3436, column: 14, scope: !163, inlinedAt: !164)
!163 = distinct !DISubprogram(name: "extend_from_slice<u8, alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$17extend_from_slice17he23345362ef15a58E", scope: !49, file: !47, line: 3435, type: !13, scopeLine: 3435, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!164 = distinct !DILocation(line: 45, column: 19, scope: !165)
!165 = distinct !DILexicalBlock(scope: !150, file: !85, line: 44, column: 9)
!166 = !{!167}
!167 = distinct !{!167, !168, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$17extend_from_slice17he23345362ef15a58E: %other.0"}
!168 = distinct !{!168, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$17extend_from_slice17he23345362ef15a58E"}
!169 = !DILocation(line: 2933, column: 19, scope: !53, inlinedAt: !170)
!170 = distinct !DILocation(line: 2820, column: 24, scope: !46, inlinedAt: !161)
!171 = !{!172, !174, !176}
!172 = distinct !{!172, !173, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE: %self"}
!173 = distinct !{!173, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE"}
!174 = distinct !{!174, !175, !"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h9308aadac9002e8dE: %self"}
!175 = distinct !{!175, !"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h9308aadac9002e8dE"}
!176 = distinct !{!176, !168, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$17extend_from_slice17he23345362ef15a58E: %self"}
!177 = !DILocation(line: 2938, column: 37, scope: !59, inlinedAt: !170)
!178 = !DILocation(line: 2938, column: 18, scope: !59, inlinedAt: !170)
!179 = !DILocation(line: 597, column: 9, scope: !62, inlinedAt: !180)
!180 = distinct !DILocation(line: 592, column: 14, scope: !67, inlinedAt: !181)
!181 = distinct !DILocation(line: 295, column: 20, scope: !69, inlinedAt: !182)
!182 = distinct !DILocation(line: 1938, column: 18, scope: !72, inlinedAt: !183)
!183 = distinct !DILocation(line: 2821, column: 67, scope: !74, inlinedAt: !161)
!184 = !DILocation(line: 961, column: 18, scope: !76, inlinedAt: !185)
!185 = distinct !DILocation(line: 2821, column: 80, scope: !74, inlinedAt: !161)
!186 = !DILocation(line: 547, column: 14, scope: !79, inlinedAt: !187)
!187 = distinct !DILocation(line: 2821, column: 18, scope: !74, inlinedAt: !161)
!188 = !DILocation(line: 2822, column: 9, scope: !74, inlinedAt: !161)
!189 = !DILocation(line: 2938, column: 37, scope: !158, inlinedAt: !190)
!190 = distinct !DILocation(line: 46, column: 27, scope: !165)
!191 = !DILocation(line: 2938, column: 18, scope: !158, inlinedAt: !190)
!192 = !DILocation(line: 2819, column: 14, scope: !46, inlinedAt: !193)
!193 = distinct !DILocation(line: 56, column: 23, scope: !51, inlinedAt: !194)
!194 = distinct !DILocation(line: 3436, column: 14, scope: !163, inlinedAt: !195)
!195 = distinct !DILocation(line: 47, column: 19, scope: !196)
!196 = distinct !DILexicalBlock(scope: !165, file: !85, line: 46, column: 9)
!197 = !{!198}
!198 = distinct !{!198, !199, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$17extend_from_slice17he23345362ef15a58E: %other.0"}
!199 = distinct !{!199, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$17extend_from_slice17he23345362ef15a58E"}
!200 = !DILocation(line: 2933, column: 19, scope: !53, inlinedAt: !201)
!201 = distinct !DILocation(line: 2820, column: 24, scope: !46, inlinedAt: !193)
!202 = !{!203, !205, !207}
!203 = distinct !{!203, !204, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE: %self"}
!204 = distinct !{!204, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE"}
!205 = distinct !{!205, !206, !"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h9308aadac9002e8dE: %self"}
!206 = distinct !{!206, !"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h9308aadac9002e8dE"}
!207 = distinct !{!207, !199, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$17extend_from_slice17he23345362ef15a58E: %self"}
!208 = !DILocation(line: 2938, column: 37, scope: !59, inlinedAt: !201)
!209 = !DILocation(line: 2938, column: 18, scope: !59, inlinedAt: !201)
!210 = !DILocation(line: 597, column: 9, scope: !62, inlinedAt: !211)
!211 = distinct !DILocation(line: 592, column: 14, scope: !67, inlinedAt: !212)
!212 = distinct !DILocation(line: 295, column: 20, scope: !69, inlinedAt: !213)
!213 = distinct !DILocation(line: 1938, column: 18, scope: !72, inlinedAt: !214)
!214 = distinct !DILocation(line: 2821, column: 67, scope: !74, inlinedAt: !193)
!215 = !DILocation(line: 961, column: 18, scope: !76, inlinedAt: !216)
!216 = distinct !DILocation(line: 2821, column: 80, scope: !74, inlinedAt: !193)
!217 = !DILocation(line: 547, column: 14, scope: !79, inlinedAt: !218)
!218 = distinct !DILocation(line: 2821, column: 18, scope: !74, inlinedAt: !193)
!219 = !DILocation(line: 2822, column: 9, scope: !74, inlinedAt: !193)
!220 = !DILocation(line: 48, column: 9, scope: !196)
!221 = !DILocation(line: 1030, column: 19, scope: !222, inlinedAt: !223)
!222 = distinct !DISubprogram(name: "push_mut<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$8push_mut17hd772ca1d8cafcdffE", scope: !49, file: !47, line: 1028, type: !13, scopeLine: 1028, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!223 = distinct !DILocation(line: 994, column: 22, scope: !224, inlinedAt: !225)
!224 = distinct !DISubprogram(name: "push<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$4push17hb288ea25d35b9a2cE", scope: !49, file: !47, line: 993, type: !13, scopeLine: 993, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!225 = distinct !DILocation(line: 48, column: 21, scope: !196)
!226 = !{!227, !229}
!227 = distinct !{!227, !228, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$8push_mut17hd772ca1d8cafcdffE: %self"}
!228 = distinct !{!228, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$8push_mut17hd772ca1d8cafcdffE"}
!229 = distinct !{!229, !230, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$4push17hb288ea25d35b9a2cE: %self"}
!230 = distinct !{!230, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$4push17hb288ea25d35b9a2cE"}
!231 = !{!232, !233}
!232 = distinct !{!232, !228, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$8push_mut17hd772ca1d8cafcdffE: %value"}
!233 = distinct !{!233, !230, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$4push17hb288ea25d35b9a2cE: %value"}
!234 = !DILocation(line: 602, column: 49, scope: !235, inlinedAt: !236)
!235 = distinct !DISubprogram(name: "capacity<alloc::alloc::Global>", linkageName: "_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$8capacity17hca97d44c0f019f09E", scope: !64, file: !63, line: 601, type: !13, scopeLine: 601, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!236 = distinct !DILocation(line: 308, column: 20, scope: !237, inlinedAt: !238)
!237 = distinct !DISubprogram(name: "capacity<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8capacity17h0b72ffdff2053c9aE", scope: !70, file: !63, line: 307, type: !13, scopeLine: 307, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!238 = distinct !DILocation(line: 1033, column: 28, scope: !239, inlinedAt: !223)
!239 = distinct !DILexicalBlock(scope: !222, file: !47, line: 1030, column: 9)
!240 = !{i64 0, i64 -9223372036854775808}
!241 = !DILocation(line: 1033, column: 12, scope: !239, inlinedAt: !223)
!242 = !DILocation(line: 1034, column: 22, scope: !239, inlinedAt: !223)
!243 = !DILocation(line: 1033, column: 9, scope: !239, inlinedAt: !223)
!244 = !DILocation(line: 44, column: 34, scope: !150)
!245 = !DILocation(line: 46, column: 34, scope: !165)
!246 = !DILocation(line: 597, column: 9, scope: !247, inlinedAt: !248)
!247 = distinct !DISubprogram(name: "non_null<alloc::alloc::Global, (core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$8non_null17haaec5a5f57ac22e6E", scope: !64, file: !63, line: 596, type: !13, scopeLine: 596, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!248 = distinct !DILocation(line: 592, column: 14, scope: !249, inlinedAt: !250)
!249 = distinct !DISubprogram(name: "ptr<alloc::alloc::Global, (core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$3ptr17h7f5b7528b9bd8317E", scope: !64, file: !63, line: 591, type: !13, scopeLine: 591, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!250 = distinct !DILocation(line: 295, column: 20, scope: !251, inlinedAt: !252)
!251 = distinct !DISubprogram(name: "ptr<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$3ptr17h1a58635acae81850E", scope: !70, file: !63, line: 294, type: !13, scopeLine: 294, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!252 = distinct !DILocation(line: 1938, column: 18, scope: !253, inlinedAt: !254)
!253 = distinct !DISubprogram(name: "as_mut_ptr<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$10as_mut_ptr17h3c8b791f090532f8E", scope: !49, file: !47, line: 1935, type: !13, scopeLine: 1935, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!254 = distinct !DILocation(line: 1037, column: 28, scope: !239, inlinedAt: !223)
!255 = !DILocation(line: 961, column: 18, scope: !256, inlinedAt: !257)
!256 = distinct !DISubprogram(name: "add<(core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$3add17hd98f55085f593490E", scope: !26, file: !25, line: 927, type: !13, scopeLine: 927, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!257 = distinct !DILocation(line: 1037, column: 41, scope: !239, inlinedAt: !223)
!258 = !DILocation(line: 1910, column: 9, scope: !259, inlinedAt: !260)
!259 = distinct !DISubprogram(name: "write<(core::ops::range::Range<usize>, core::ops::range::Range<usize>)>", linkageName: "_ZN4core3ptr5write17h6b65a3c3e3d1a71fE", scope: !21, file: !80, line: 1887, type: !13, scopeLine: 1887, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!260 = distinct !DILocation(line: 1038, column: 13, scope: !261, inlinedAt: !223)
!261 = distinct !DILexicalBlock(scope: !239, file: !47, line: 1037, column: 13)
!262 = !DILocation(line: 1039, column: 13, scope: !261, inlinedAt: !223)
!263 = !DILocation(line: 49, column: 6, scope: !150)
!264 = distinct !DISubprogram(name: "clear", linkageName: "_ZN21nexagate_hpack_before5hpack7decoder9FieldList5clear17hef74b301bd19135cE", scope: !86, file: !85, line: 24, type: !13, scopeLine: 24, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!265 = !DILocation(line: 1789, column: 92, scope: !266, inlinedAt: !267)
!266 = distinct !DISubprogram(name: "as_mut_slice<u8, alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$12as_mut_slice17h091d941cd8ec60fdE", scope: !49, file: !47, line: 1772, type: !13, scopeLine: 1772, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!267 = distinct !DILocation(line: 2905, column: 36, scope: !268, inlinedAt: !269)
!268 = distinct !DISubprogram(name: "clear<u8, alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$5clear17h7fe0afd16375f126E", scope: !49, file: !47, line: 2904, type: !13, scopeLine: 2904, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!269 = distinct !DILocation(line: 25, column: 19, scope: !264)
!270 = !DILocation(line: 2914, column: 13, scope: !271, inlinedAt: !269)
!271 = distinct !DILexicalBlock(scope: !268, file: !47, line: 2905, column: 9)
!272 = !{!273}
!273 = distinct !{!273, !274, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$5clear17h7fe0afd16375f126E: %self"}
!274 = distinct !{!274, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$5clear17h7fe0afd16375f126E"}
!275 = !DILocation(line: 1789, column: 92, scope: !276, inlinedAt: !277)
!276 = distinct !DISubprogram(name: "as_mut_slice<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$12as_mut_slice17h6823e64508b81f82E", scope: !49, file: !47, line: 1772, type: !13, scopeLine: 1772, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!277 = distinct !DILocation(line: 2905, column: 36, scope: !278, inlinedAt: !279)
!278 = distinct !DISubprogram(name: "clear<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$5clear17h29f40780ac48155fE", scope: !49, file: !47, line: 2904, type: !13, scopeLine: 2904, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!279 = distinct !DILocation(line: 26, column: 21, scope: !264)
!280 = !DILocation(line: 2914, column: 13, scope: !281, inlinedAt: !279)
!281 = distinct !DILexicalBlock(scope: !278, file: !47, line: 2905, column: 9)
!282 = !{!283}
!283 = distinct !{!283, !284, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$5clear17h29f40780ac48155fE: %self"}
!284 = distinct !{!284, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$5clear17h29f40780ac48155fE"}
!285 = !DILocation(line: 27, column: 6, scope: !264)
!286 = distinct !DISubprogram(name: "is_empty", linkageName: "_ZN21nexagate_hpack_before5hpack7decoder9FieldList8is_empty17h9e8ba878cb51e8a7E", scope: !86, file: !85, line: 33, type: !13, scopeLine: 33, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!287 = !DILocation(line: 2933, column: 19, scope: !288, inlinedAt: !289)
!288 = distinct !DISubprogram(name: "len<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h1c0ab8907c89c339E", scope: !49, file: !47, line: 2932, type: !13, scopeLine: 2932, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!289 = distinct !DILocation(line: 2958, column: 14, scope: !290, inlinedAt: !291)
!290 = distinct !DISubprogram(name: "is_empty<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc3vec16Vec$LT$T$C$A$GT$8is_empty17h72d3840a4e17f11bE", scope: !49, file: !47, line: 2957, type: !13, scopeLine: 2957, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!291 = distinct !DILocation(line: 34, column: 21, scope: !286)
!292 = !{!293}
!293 = distinct !{!293, !294, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$8is_empty17h72d3840a4e17f11bE: %self"}
!294 = distinct !{!294, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$8is_empty17h72d3840a4e17f11bE"}
!295 = !DILocation(line: 2938, column: 37, scope: !296, inlinedAt: !289)
!296 = distinct !DILexicalBlock(scope: !288, file: !47, line: 2933, column: 9)
!297 = !DILocation(line: 2938, column: 18, scope: !296, inlinedAt: !289)
!298 = !DILocation(line: 2958, column: 9, scope: !290, inlinedAt: !291)
!299 = !DILocation(line: 35, column: 6, scope: !286)
!300 = distinct !DISubprogram(name: "fmt<&str>", linkageName: "_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hf912134bce2d0087E", scope: !302, file: !301, line: 2865, type: !13, scopeLine: 2865, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!301 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/fmt/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "66c54229528687a22c80807f245fa4df")
!302 = !DINamespace(name: "{impl#82}", scope: !303)
!303 = !DINamespace(name: "fmt", scope: !22)
!304 = !DILocation(line: 2865, column: 71, scope: !300)
!305 = !{i64 8}
!306 = !DILocation(line: 2865, column: 62, scope: !300)
!307 = !DILocation(line: 2865, column: 84, scope: !300)
!308 = !DILocation(line: 2819, column: 14, scope: !46, inlinedAt: !309)
!309 = distinct !DILocation(line: 56, column: 23, scope: !51, inlinedAt: !310)
!310 = distinct !DILocation(line: 3436, column: 14, scope: !163)
!311 = !DILocation(line: 2933, column: 19, scope: !53, inlinedAt: !312)
!312 = distinct !DILocation(line: 2820, column: 24, scope: !46, inlinedAt: !309)
!313 = !{!314, !316}
!314 = distinct !{!314, !315, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE: %self"}
!315 = distinct !{!315, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$15append_elements17h575f27cfbda8ca9eE"}
!316 = distinct !{!316, !317, !"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h9308aadac9002e8dE: %self"}
!317 = distinct !{!317, !"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h9308aadac9002e8dE"}
!318 = !DILocation(line: 2938, column: 37, scope: !59, inlinedAt: !312)
!319 = !DILocation(line: 2938, column: 18, scope: !59, inlinedAt: !312)
!320 = !DILocation(line: 597, column: 9, scope: !62, inlinedAt: !321)
!321 = distinct !DILocation(line: 592, column: 14, scope: !67, inlinedAt: !322)
!322 = distinct !DILocation(line: 295, column: 20, scope: !69, inlinedAt: !323)
!323 = distinct !DILocation(line: 1938, column: 18, scope: !72, inlinedAt: !324)
!324 = distinct !DILocation(line: 2821, column: 67, scope: !74, inlinedAt: !309)
!325 = !DILocation(line: 961, column: 18, scope: !76, inlinedAt: !326)
!326 = distinct !DILocation(line: 2821, column: 80, scope: !74, inlinedAt: !309)
!327 = !DILocation(line: 547, column: 14, scope: !79, inlinedAt: !328)
!328 = distinct !DILocation(line: 2821, column: 18, scope: !74, inlinedAt: !309)
!329 = !DILocation(line: 2822, column: 9, scope: !74, inlinedAt: !309)
!330 = !DILocation(line: 3437, column: 6, scope: !163)
!331 = distinct !DISubprogram(name: "grow_one<(core::ops::range::Range<usize>, core::ops::range::Range<usize>), alloc::alloc::Global>", linkageName: "_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h76175b00c5618055E", scope: !70, file: !63, line: 186, type: !13, scopeLine: 186, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!332 = !DILocation(line: 476, column: 56, scope: !333, inlinedAt: !335)
!333 = distinct !DILexicalBlock(scope: !334, file: !63, line: 476, column: 95)
!334 = distinct !DISubprogram(name: "grow_one<alloc::alloc::Global>", linkageName: "_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$8grow_one17h71781a1d7f9991b1E", scope: !64, file: !63, line: 474, type: !13, scopeLine: 474, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!335 = !DILocation(line: 188, column: 29, scope: !331)
!336 = !DILocation(line: 476, column: 41, scope: !333, inlinedAt: !335)
!337 = !DILocation(line: 476, column: 27, scope: !333, inlinedAt: !335)
!338 = !DILocation(line: 476, column: 16, scope: !333, inlinedAt: !335)
!339 = !{!"branch_weights", !"expected", i32 2000, i32 1}
!340 = !DILocation(line: 477, column: 13, scope: !333, inlinedAt: !335)
!341 = !DILocation(line: 189, column: 6, scope: !331)
!342 = distinct !DISubprogram(name: "drop<(&[u8], usize), alloc::alloc::Global>", linkageName: "_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h580f52b1ec55ace2E", scope: !343, file: !47, line: 4155, type: !13, scopeLine: 4155, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!343 = !DINamespace(name: "{impl#26}", scope: !11)
!344 = !DILocation(line: 4163, column: 6, scope: !342)
!345 = distinct !DISubprogram(name: "drop<(&[u8], usize), alloc::alloc::Global>", linkageName: "_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h1babb42875e99263E", scope: !346, file: !63, line: 406, type: !13, scopeLine: 406, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!346 = !DINamespace(name: "{impl#3}", scope: !65)
!347 = !DILocation(line: 408, column: 29, scope: !345)
!348 = !DILocation(line: 409, column: 6, scope: !345)
!349 = distinct !DISubprogram(name: "drop<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>)), alloc::alloc::Global>", linkageName: "_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hb71e566929c0c2b0E", scope: !351, file: !350, line: 3493, type: !13, scopeLine: 3493, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!350 = !DIFile(filename: "src/raw/mod.rs", directory: "/rust/deps/hashbrown-0.16.1", checksumkind: CSK_MD5, checksum: "d4743e4e526b3ca0804e467dc6a5bf88")
!351 = !DINamespace(name: "{impl#18}", scope: !352)
!352 = !DINamespace(name: "raw", scope: !353)
!353 = !DINamespace(name: "hashbrown", scope: null)
!354 = !DILocation(line: 3503, column: 18, scope: !349)
!355 = !DILocation(line: 3505, column: 6, scope: !349)
!356 = distinct !DISubprogram(name: "fmt", linkageName: "_ZN79_$LT$nexagate_hpack_before..hpack..HpackError$u20$as$u20$core..fmt..Display$GT$3fmt17h3d558c1ca0f89cedE", scope: !358, file: !357, line: 38, type: !13, scopeLine: 38, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!357 = !DIFile(filename: "examples/../fixtures/nexagate-before/hpack/mod.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "dc4adc4475bc438edb15f6cf1cadbcb6")
!358 = !DINamespace(name: "{impl#0}", scope: !88)
!359 = !DILocation(line: 39, column: 15, scope: !356)
!360 = !{i64 0, i64 3}
!361 = !DILocation(line: 39, column: 9, scope: !356)
!362 = !DILocation(line: 40, column: 35, scope: !356)
!363 = !DILocation(line: 40, column: 41, scope: !364)
!364 = !DILexicalBlockFile(scope: !365, file: !357, discriminator: 0)
!365 = distinct !DILexicalBlock(scope: !367, file: !366, line: 612, column: 24)
!366 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/macros/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "556043ed3ddb5d4fa16c25cfee6b5d07")
!367 = distinct !DILexicalBlock(scope: !356, file: !357, line: 40, column: 13)
!368 = !{!369}
!369 = distinct !{!369, !370, !"_ZN4core3fmt9Formatter9write_fmt17h13a7173128c99489E: %self"}
!370 = distinct !{!370, !"_ZN4core3fmt9Formatter9write_fmt17h13a7173128c99489E"}
!371 = !DILocation(line: 40, column: 41, scope: !367)
!372 = !DILocation(line: 2125, column: 19, scope: !373, inlinedAt: !375)
!373 = distinct !DISubprogram(name: "write_fmt", linkageName: "_ZN4core3fmt9Formatter9write_fmt17h13a7173128c99489E", scope: !374, file: !301, line: 2121, type: !13, scopeLine: 2121, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!374 = !DINamespace(name: "Formatter", scope: !303)
!375 = distinct !DILocation(line: 40, column: 41, scope: !367)
!376 = !{i64 1}
!377 = !DILocation(line: 2125, column: 13, scope: !373, inlinedAt: !375)
!378 = !DILocation(line: 40, column: 80, scope: !356)
!379 = !DILocation(line: 41, column: 33, scope: !356)
!380 = !DILocation(line: 41, column: 39, scope: !381)
!381 = !DILexicalBlockFile(scope: !382, file: !357, discriminator: 0)
!382 = distinct !DILexicalBlock(scope: !383, file: !366, line: 612, column: 24)
!383 = distinct !DILexicalBlock(scope: !356, file: !357, line: 41, column: 13)
!384 = !{!385}
!385 = distinct !{!385, !386, !"_ZN4core3fmt9Formatter9write_fmt17h13a7173128c99489E: %self"}
!386 = distinct !{!386, !"_ZN4core3fmt9Formatter9write_fmt17h13a7173128c99489E"}
!387 = !DILocation(line: 41, column: 39, scope: !383)
!388 = !DILocation(line: 2125, column: 19, scope: !373, inlinedAt: !389)
!389 = distinct !DILocation(line: 41, column: 39, scope: !383)
!390 = !DILocation(line: 2125, column: 13, scope: !373, inlinedAt: !389)
!391 = !DILocation(line: 41, column: 74, scope: !356)
!392 = !DILocation(line: 42, column: 39, scope: !356)
!393 = !DILocation(line: 44, column: 6, scope: !356)
!394 = distinct !DISubprogram(name: "default", linkageName: "_ZN89_$LT$nexagate_hpack_before..hpack..encoder..Encoder$u20$as$u20$core..default..Default$GT$7default17h80dd29de72c76951E", scope: !396, file: !395, line: 51, type: !13, scopeLine: 51, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!395 = !DIFile(filename: "examples/../fixtures/nexagate-before/hpack/encoder.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "3b8800600b91fc5d99e22b9922fb1759")
!396 = !DINamespace(name: "{impl#0}", scope: !397)
!397 = !DINamespace(name: "encoder", scope: !88)
!398 = !DILocation(line: 59, column: 9, scope: !399, inlinedAt: !401)
!399 = distinct !DISubprogram(name: "new", linkageName: "_ZN21nexagate_hpack_before5hpack7encoder7Encoder3new17h9d1758727fafc168E", scope: !400, file: !395, line: 58, type: !13, scopeLine: 58, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!400 = !DINamespace(name: "Encoder", scope: !397)
!401 = distinct !DILocation(line: 52, column: 9, scope: !394)
!402 = !DILocation(line: 783, column: 9, scope: !403, inlinedAt: !408)
!403 = distinct !DISubprogram(name: "new<(alloc::vec::Vec<u8, alloc::alloc::Global>, alloc::vec::Vec<u8, alloc::alloc::Global>)>", linkageName: "_ZN5alloc11collections9vec_deque17VecDeque$LT$T$GT$3new17h1219026f52604d3aE", scope: !405, file: !404, line: 781, type: !13, scopeLine: 781, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!404 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "1bd8432c344a69791a7df76ba3482643")
!405 = !DINamespace(name: "VecDeque", scope: !406)
!406 = !DINamespace(name: "vec_deque", scope: !407)
!407 = !DINamespace(name: "collections", scope: !12)
!408 = distinct !DILocation(line: 166, column: 9, scope: !409, inlinedAt: !411)
!409 = distinct !DISubprogram(name: "default<(alloc::vec::Vec<u8, alloc::alloc::Global>, alloc::vec::Vec<u8, alloc::alloc::Global>)>", linkageName: "_ZN91_$LT$alloc..collections..vec_deque..VecDeque$LT$T$GT$$u20$as$u20$core..default..Default$GT$7default17h77f718fe2ad05716E", scope: !410, file: !404, line: 165, type: !13, scopeLine: 165, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!410 = !DINamespace(name: "{impl#2}", scope: !406)
!411 = distinct !DILocation(line: 18, column: 5, scope: !412, inlinedAt: !416)
!412 = distinct !DISubprogram(name: "default", linkageName: "_ZN92_$LT$nexagate_hpack_before..hpack..table..DynamicTable$u20$as$u20$core..default..Default$GT$7default17h69c1f95f8586a105E", scope: !414, file: !413, line: 15, type: !13, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!413 = !DIFile(filename: "examples/../fixtures/nexagate-before/hpack/table.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "5d78dd90936d01a5a2d7c6583cd3c009")
!414 = !DINamespace(name: "{impl#2}", scope: !415)
!415 = !DINamespace(name: "table", scope: !88)
!416 = distinct !DILocation(line: 27, column: 15, scope: !417, inlinedAt: !419)
!417 = distinct !DISubprogram(name: "new", linkageName: "_ZN21nexagate_hpack_before5hpack5table12DynamicTable3new17hca5283b0dd1e466fE", scope: !418, file: !413, line: 24, type: !13, scopeLine: 24, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!418 = !DINamespace(name: "DynamicTable", scope: !415)
!419 = distinct !DILocation(line: 60, column: 20, scope: !399, inlinedAt: !401)
!420 = !{!421}
!421 = distinct !{!421, !422, !"_ZN21nexagate_hpack_before5hpack7encoder7Encoder3new17h9d1758727fafc168E: %_0"}
!422 = distinct !{!422, !"_ZN21nexagate_hpack_before5hpack7encoder7Encoder3new17h9d1758727fafc168E"}
!423 = !DILocation(line: 53, column: 6, scope: !394)
!424 = distinct !DISubprogram(name: "drop_elements<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E", scope: !425, file: !350, line: 2250, type: !13, scopeLine: 2250, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!425 = !DINamespace(name: "RawTableInner", scope: !352)
!426 = !DILocation(line: 2253, column: 29, scope: !424)
!427 = !DILocation(line: 2479, column: 9, scope: !428, inlinedAt: !429)
!428 = distinct !DISubprogram(name: "data_end<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN9hashbrown3raw13RawTableInner8data_end17h9afebd59970cf5dbE", scope: !425, file: !350, line: 2478, type: !13, scopeLine: 2478, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!429 = !DILocation(line: 2206, column: 49, scope: !430, inlinedAt: !431)
!430 = distinct !DISubprogram(name: "iter<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN9hashbrown3raw13RawTableInner4iter17h7edcf9d28c56f8b1E", scope: !425, file: !350, line: 2178, type: !13, scopeLine: 2178, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!431 = !DILocation(line: 2257, column: 30, scope: !424)
!432 = !DILocation(line: 1320, column: 5, scope: !433, inlinedAt: !438)
!433 = distinct !DISubprogram(name: "_mm_load_si128", linkageName: "_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E", scope: !435, file: !434, line: 1319, type: !13, scopeLine: 1319, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!434 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs", directory: "", checksumkind: CSK_MD5, checksum: "29c370186713c63399256ec742839eeb")
!435 = !DINamespace(name: "sse2", scope: !436)
!436 = !DINamespace(name: "x86", scope: !437)
!437 = !DINamespace(name: "core_arch", scope: !22)
!438 = distinct !DILocation(line: 61, column: 15, scope: !439, inlinedAt: !445)
!439 = distinct !DISubprogram(name: "load_aligned", linkageName: "_ZN9hashbrown7control5group4sse25Group12load_aligned17ha07ba8dbe9b4d6c4E", scope: !441, file: !440, line: 59, type: !13, scopeLine: 59, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!440 = !DIFile(filename: "src/control/group/sse2.rs", directory: "/rust/deps/hashbrown-0.16.1", checksumkind: CSK_MD5, checksum: "6d810d400930bb3b974ad2d7eb5dceef")
!441 = !DINamespace(name: "Group", scope: !442)
!442 = !DINamespace(name: "sse2", scope: !443)
!443 = !DINamespace(name: "group", scope: !444)
!444 = !DINamespace(name: "control", scope: !353)
!445 = distinct !DILocation(line: 3592, column: 29, scope: !446, inlinedAt: !449)
!446 = distinct !DILexicalBlock(scope: !447, file: !350, line: 3588, column: 9)
!447 = distinct !DISubprogram(name: "new<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17hfd5cca0e9bf5885fE", scope: !448, file: !350, line: 3584, type: !13, scopeLine: 3584, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!448 = !DINamespace(name: "RawIterRange", scope: !352)
!449 = distinct !DILocation(line: 2209, column: 19, scope: !450, inlinedAt: !431)
!450 = distinct !DILexicalBlock(scope: !430, file: !350, line: 2206, column: 9)
!451 = !{!452, !454}
!452 = distinct !{!452, !453, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E: %_0"}
!453 = distinct !{!453, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E"}
!454 = distinct !{!454, !455, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17hfd5cca0e9bf5885fE: %_0"}
!455 = distinct !{!455, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17hfd5cca0e9bf5885fE"}
!456 = !DILocation(line: 1563, column: 9, scope: !457, inlinedAt: !460)
!457 = distinct !DILexicalBlock(scope: !458, file: !434, line: 1562, column: 9)
!458 = distinct !DILexicalBlock(scope: !459, file: !434, line: 1561, column: 9)
!459 = distinct !DISubprogram(name: "_mm_movemask_epi8", linkageName: "_ZN4core9core_arch3x864sse217_mm_movemask_epi817he2e5199c67e3b8e9E", scope: !435, file: !434, line: 1559, type: !13, scopeLine: 1559, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!460 = distinct !DILocation(line: 111, column: 21, scope: !461, inlinedAt: !463)
!461 = distinct !DILexicalBlock(scope: !462, file: !440, line: 109, column: 9)
!462 = distinct !DISubprogram(name: "match_empty_or_deleted", linkageName: "_ZN9hashbrown7control5group4sse25Group22match_empty_or_deleted17hc9e99f2577a91021E", scope: !441, file: !440, line: 101, type: !13, scopeLine: 101, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!463 = distinct !DILocation(line: 118, column: 14, scope: !464, inlinedAt: !465)
!464 = distinct !DISubprogram(name: "match_full", linkageName: "_ZN9hashbrown7control5group4sse25Group10match_full17h6dd36ef263b02c12E", scope: !441, file: !440, line: 117, type: !13, scopeLine: 117, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!465 = distinct !DILocation(line: 3592, column: 62, scope: !446, inlinedAt: !449)
!466 = !DILocation(line: 31, column: 17, scope: !467, inlinedAt: !471)
!467 = distinct !DISubprogram(name: "invert", linkageName: "_ZN9hashbrown7control7bitmask7BitMask6invert17hc82d9047e9705115E", scope: !469, file: !468, line: 30, type: !13, scopeLine: 30, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!468 = !DIFile(filename: "src/control/bitmask.rs", directory: "/rust/deps/hashbrown-0.16.1", checksumkind: CSK_MD5, checksum: "ff7f3e6a3e2daa5cd0d49465a9feef11")
!469 = !DINamespace(name: "BitMask", scope: !470)
!470 = !DINamespace(name: "bitmask", scope: !444)
!471 = distinct !DILocation(line: 118, column: 39, scope: !464, inlinedAt: !465)
!472 = !DILocation(line: 863, column: 18, scope: !473, inlinedAt: !474)
!473 = distinct !DISubprogram(name: "add<u8>", linkageName: "_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$3add17h25a2da1fa2e8fea7E", scope: !19, file: !17, line: 829, type: !13, scopeLine: 829, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!474 = distinct !DILocation(line: 3593, column: 30, scope: !475, inlinedAt: !449)
!475 = distinct !DILexicalBlock(scope: !446, file: !350, line: 3592, column: 9)
!476 = !DILocation(line: 3851, column: 12, scope: !477, inlinedAt: !479)
!477 = distinct !DISubprogram(name: "next<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h4ed32a1fe5f651a2E", scope: !478, file: !350, line: 3848, type: !13, scopeLine: 3848, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!478 = !DINamespace(name: "{impl#29}", scope: !352)
!479 = !DILocation(line: 2257, column: 25, scope: !480)
!480 = !DILexicalBlockFile(scope: !481, file: !350, discriminator: 2)
!481 = distinct !DILexicalBlock(scope: !424, file: !350, line: 2257, column: 13)
!482 = !DILocation(line: 2263, column: 6, scope: !424)
!483 = !DILocation(line: 50, column: 32, scope: !484, inlinedAt: !486)
!484 = distinct !DILexicalBlock(scope: !485, file: !468, line: 50, column: 64)
!485 = distinct !DISubprogram(name: "lowest_set_bit", linkageName: "_ZN9hashbrown7control7bitmask7BitMask14lowest_set_bit17h05ab5489aeb844d7E", scope: !469, file: !468, line: 49, type: !13, scopeLine: 49, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!486 = distinct !DILocation(line: 113, column: 26, scope: !487, inlinedAt: !489)
!487 = distinct !DISubprogram(name: "next", linkageName: "_ZN99_$LT$hashbrown..control..bitmask..BitMaskIter$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17heef676c83c1f2c7fE", scope: !488, file: !468, line: 112, type: !13, scopeLine: 112, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!488 = !DINamespace(name: "{impl#2}", scope: !470)
!489 = distinct !DILocation(line: 3653, column: 53, scope: !490, inlinedAt: !492)
!490 = distinct !DILexicalBlock(scope: !491, file: !350, line: 3653, column: 60)
!491 = distinct !DISubprogram(name: "next_impl<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>)), false>", linkageName: "_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E", scope: !448, file: !350, line: 3651, type: !13, scopeLine: 3651, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!492 = distinct !DILocation(line: 3857, column: 23, scope: !477, inlinedAt: !479)
!493 = !DILocation(line: 50, column: 16, scope: !484, inlinedAt: !486)
!494 = !DILocation(line: 31, column: 17, scope: !495, inlinedAt: !496)
!495 = distinct !DISubprogram(name: "invert", linkageName: "_ZN9hashbrown7control7bitmask7BitMask6invert17hc82d9047e9705115E", scope: !469, file: !468, line: 30, type: !13, scopeLine: 30, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!496 = distinct !DILocation(line: 118, column: 39, scope: !497, inlinedAt: !498)
!497 = distinct !DISubprogram(name: "match_full", linkageName: "_ZN9hashbrown7control5group4sse25Group10match_full17h6dd36ef263b02c12E", scope: !441, file: !440, line: 117, type: !13, scopeLine: 117, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!498 = distinct !DILocation(line: 3667, column: 18, scope: !491, inlinedAt: !492)
!499 = !DILocation(line: 3666, column: 54, scope: !491, inlinedAt: !492)
!500 = !DILocation(line: 1320, column: 5, scope: !433, inlinedAt: !501)
!501 = distinct !DILocation(line: 61, column: 15, scope: !502, inlinedAt: !503)
!502 = distinct !DISubprogram(name: "load_aligned", linkageName: "_ZN9hashbrown7control5group4sse25Group12load_aligned17ha07ba8dbe9b4d6c4E", scope: !441, file: !440, line: 59, type: !13, scopeLine: 59, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!503 = distinct !DILocation(line: 3666, column: 34, scope: !491, inlinedAt: !492)
!504 = !{!505, !507}
!505 = distinct !{!505, !506, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E: %_0"}
!506 = distinct !{!506, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E"}
!507 = distinct !{!507, !508, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E: %self"}
!508 = distinct !{!508, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E"}
!509 = !DILocation(line: 1563, column: 9, scope: !457, inlinedAt: !510)
!510 = distinct !DILocation(line: 111, column: 21, scope: !511, inlinedAt: !513)
!511 = distinct !DILexicalBlock(scope: !512, file: !440, line: 109, column: 9)
!512 = distinct !DISubprogram(name: "match_empty_or_deleted", linkageName: "_ZN9hashbrown7control5group4sse25Group22match_empty_or_deleted17hc9e99f2577a91021E", scope: !441, file: !440, line: 101, type: !13, scopeLine: 101, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!513 = distinct !DILocation(line: 118, column: 14, scope: !497, inlinedAt: !498)
!514 = !DILocation(line: 1072, column: 22, scope: !515, inlinedAt: !516)
!515 = distinct !DISubprogram(name: "sub<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$3sub17ha2cef81c67de1474E", scope: !26, file: !25, line: 1033, type: !13, scopeLine: 1033, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!516 = distinct !DILocation(line: 495, column: 31, scope: !517, inlinedAt: !520)
!517 = !DILexicalBlockFile(scope: !518, file: !350, discriminator: 2)
!518 = distinct !DISubprogram(name: "next_n<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN9hashbrown3raw15Bucket$LT$T$GT$6next_n17h9e0b4bc9ba5b3c07E", scope: !519, file: !350, line: 490, type: !13, scopeLine: 490, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!519 = !DINamespace(name: "Bucket", scope: !352)
!520 = distinct !DILocation(line: 3669, column: 35, scope: !491, inlinedAt: !492)
!521 = !DILocation(line: 863, column: 18, scope: !522, inlinedAt: !523)
!522 = distinct !DISubprogram(name: "add<u8>", linkageName: "_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$3add17h25a2da1fa2e8fea7E", scope: !19, file: !17, line: 829, type: !13, scopeLine: 829, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!523 = distinct !DILocation(line: 3670, column: 45, scope: !491, inlinedAt: !492)
!524 = !DILocation(line: 2257, column: 25, scope: !424)
!525 = !DILocation(line: 113, column: 19, scope: !487, inlinedAt: !489)
!526 = !DILocation(line: 499, column: 18, scope: !527, inlinedAt: !532)
!527 = distinct !DISubprogram(name: "get<u16>", linkageName: "_ZN4core3num7nonzero16NonZero$LT$T$GT$3get17h464eb7fad7985e0cE", scope: !529, file: !528, line: 480, type: !13, scopeLine: 480, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!528 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs", directory: "", checksumkind: CSK_MD5, checksum: "39bffe678f409cd2f9a313432f00c918")
!529 = !DINamespace(name: "NonZero", scope: !530)
!530 = !DINamespace(name: "nonzero", scope: !531)
!531 = !DINamespace(name: "num", scope: !22)
!532 = distinct !DILocation(line: 641, column: 51, scope: !533, inlinedAt: !534)
!533 = distinct !DISubprogram(name: "trailing_zeros", linkageName: "_ZN4core3num7nonzero18NonZero$LT$u16$GT$14trailing_zeros17h9863e36eefd6fa3aE", scope: !529, file: !528, line: 638, type: !13, scopeLine: 638, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!534 = distinct !DILocation(line: 80, column: 21, scope: !535, inlinedAt: !536)
!535 = distinct !DISubprogram(name: "nonzero_trailing_zeros", linkageName: "_ZN9hashbrown7control7bitmask7BitMask22nonzero_trailing_zeros17he3b1206b6b1f4022E", scope: !469, file: !468, line: 74, type: !13, scopeLine: 74, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!536 = distinct !DILocation(line: 51, column: 18, scope: !484, inlinedAt: !486)
!537 = !DILocation(line: 641, column: 21, scope: !533, inlinedAt: !534)
!538 = !DILocation(line: 80, column: 13, scope: !535, inlinedAt: !536)
!539 = !DILocation(line: 38, column: 17, scope: !540, inlinedAt: !541)
!540 = distinct !DISubprogram(name: "remove_lowest_bit", linkageName: "_ZN9hashbrown7control7bitmask7BitMask17remove_lowest_bit17h24d9ea58abdac4c1E", scope: !469, file: !468, line: 37, type: !13, scopeLine: 37, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!541 = distinct !DILocation(line: 114, column: 25, scope: !542, inlinedAt: !489)
!542 = distinct !DILexicalBlock(scope: !487, file: !468, line: 113, column: 9)
!543 = !DILocation(line: 1072, column: 47, scope: !515, inlinedAt: !544)
!544 = distinct !DILocation(line: 495, column: 31, scope: !518, inlinedAt: !545)
!545 = distinct !DILocation(line: 3654, column: 39, scope: !490, inlinedAt: !492)
!546 = !DILocation(line: 1072, column: 22, scope: !515, inlinedAt: !544)
!547 = !DILocation(line: 3861, column: 9, scope: !548, inlinedAt: !479)
!548 = distinct !DILexicalBlock(scope: !477, file: !350, line: 3855, column: 9)
!549 = !DILocation(line: 805, column: 1, scope: !550, inlinedAt: !551)
!550 = distinct !DISubprogram(name: "drop_in_place<(usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>)>", linkageName: "_ZN4core3ptr92drop_in_place$LT$$LP$usize$C$alloc..vec..Vec$LT$$LP$$RF$$u5b$u8$u5d$$C$usize$RP$$GT$$RP$$GT$17h5ee5e1b122e7f447E", scope: !21, file: !80, line: 805, type: !13, scopeLine: 805, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!551 = distinct !DILocation(line: 805, column: 1, scope: !552, inlinedAt: !553)
!552 = distinct !DISubprogram(name: "drop_in_place<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN4core3ptr119drop_in_place$LT$$LP$$RF$$u5b$u8$u5d$$C$$LP$usize$C$alloc..vec..Vec$LT$$LP$$RF$$u5b$u8$u5d$$C$usize$RP$$GT$$RP$$RP$$GT$17h5d573612b63b9641E", scope: !21, file: !80, line: 805, type: !13, scopeLine: 805, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!553 = distinct !DILocation(line: 1400, column: 18, scope: !554, inlinedAt: !555)
!554 = distinct !DISubprogram(name: "drop_in_place<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$13drop_in_place17h011b3a25b5b40a60E", scope: !26, file: !25, line: 1395, type: !13, scopeLine: 1395, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!555 = !DILocation(line: 519, column: 23, scope: !556, inlinedAt: !557)
!556 = distinct !DISubprogram(name: "drop<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>))>", linkageName: "_ZN9hashbrown3raw15Bucket$LT$T$GT$4drop17h53bf8104285f9bd0E", scope: !519, file: !350, line: 518, type: !13, scopeLine: 518, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!557 = !DILocation(line: 2260, column: 22, scope: !558)
!558 = distinct !DILexicalBlock(scope: !481, file: !350, line: 2257, column: 42)
!559 = !DILocation(line: 408, column: 29, scope: !345, inlinedAt: !560)
!560 = distinct !DILocation(line: 805, column: 1, scope: !561, inlinedAt: !562)
!561 = distinct !DISubprogram(name: "drop_in_place<alloc::raw_vec::RawVec<(&[u8], usize), alloc::alloc::Global>>", linkageName: "_ZN4core3ptr83drop_in_place$LT$alloc..raw_vec..RawVec$LT$$LP$$RF$$u5b$u8$u5d$$C$usize$RP$$GT$$GT$17he4898e445c7391d8E", scope: !21, file: !80, line: 805, type: !13, scopeLine: 805, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!562 = distinct !DILocation(line: 805, column: 1, scope: !563, inlinedAt: !564)
!563 = distinct !DISubprogram(name: "drop_in_place<alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>>", linkageName: "_ZN4core3ptr76drop_in_place$LT$alloc..vec..Vec$LT$$LP$$RF$$u5b$u8$u5d$$C$usize$RP$$GT$$GT$17h0bf4fc5963638fbfE", scope: !21, file: !80, line: 805, type: !13, scopeLine: 805, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!564 = distinct !DILocation(line: 805, column: 1, scope: !550, inlinedAt: !551)
!565 = distinct !DISubprogram(name: "drop_inner_table<(&[u8], (usize, alloc::vec::Vec<(&[u8], usize), alloc::alloc::Global>)), alloc::alloc::Global>", linkageName: "_ZN9hashbrown3raw13RawTableInner16drop_inner_table17hf55df0bb53f2638dE", scope: !425, file: !350, line: 2311, type: !13, scopeLine: 2311, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!566 = !DILocation(line: 2697, column: 9, scope: !567, inlinedAt: !568)
!567 = distinct !DISubprogram(name: "is_empty_singleton", linkageName: "_ZN9hashbrown3raw13RawTableInner18is_empty_singleton17h7b5e9b146aa92bf0E", scope: !425, file: !350, line: 2696, type: !13, scopeLine: 2696, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!568 = !DILocation(line: 2312, column: 18, scope: !565)
!569 = !DILocation(line: 2312, column: 13, scope: !565)
!570 = !{!571}
!571 = distinct !{!571, !572, !"_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E: %self"}
!572 = distinct !{!572, !"_ZN9hashbrown3raw13RawTableInner13drop_elements17h6647967e016c2191E"}
!573 = !DILocation(line: 2315, column: 22, scope: !565)
!574 = !DILocation(line: 2253, column: 29, scope: !424, inlinedAt: !575)
!575 = distinct !DILocation(line: 2315, column: 22, scope: !565)
!576 = !DILocation(line: 2479, column: 9, scope: !428, inlinedAt: !577)
!577 = distinct !DILocation(line: 2206, column: 49, scope: !430, inlinedAt: !578)
!578 = distinct !DILocation(line: 2257, column: 30, scope: !424, inlinedAt: !575)
!579 = !DILocation(line: 1320, column: 5, scope: !433, inlinedAt: !580)
!580 = distinct !DILocation(line: 61, column: 15, scope: !439, inlinedAt: !581)
!581 = distinct !DILocation(line: 3592, column: 29, scope: !446, inlinedAt: !582)
!582 = distinct !DILocation(line: 2209, column: 19, scope: !450, inlinedAt: !578)
!583 = !{!584, !586, !571}
!584 = distinct !{!584, !585, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E: %_0"}
!585 = distinct !{!585, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E"}
!586 = distinct !{!586, !587, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17hfd5cca0e9bf5885fE: %_0"}
!587 = distinct !{!587, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17hfd5cca0e9bf5885fE"}
!588 = !DILocation(line: 1563, column: 9, scope: !457, inlinedAt: !589)
!589 = distinct !DILocation(line: 111, column: 21, scope: !461, inlinedAt: !590)
!590 = distinct !DILocation(line: 118, column: 14, scope: !464, inlinedAt: !591)
!591 = distinct !DILocation(line: 3592, column: 62, scope: !446, inlinedAt: !582)
!592 = !DILocation(line: 31, column: 17, scope: !467, inlinedAt: !593)
!593 = distinct !DILocation(line: 118, column: 39, scope: !464, inlinedAt: !591)
!594 = !DILocation(line: 863, column: 18, scope: !473, inlinedAt: !595)
!595 = distinct !DILocation(line: 3593, column: 30, scope: !475, inlinedAt: !582)
!596 = !DILocation(line: 3851, column: 12, scope: !477, inlinedAt: !597)
!597 = distinct !DILocation(line: 2257, column: 25, scope: !480, inlinedAt: !575)
!598 = !DILocation(line: 50, column: 32, scope: !484, inlinedAt: !599)
!599 = distinct !DILocation(line: 113, column: 26, scope: !487, inlinedAt: !600)
!600 = distinct !DILocation(line: 3653, column: 53, scope: !490, inlinedAt: !601)
!601 = distinct !DILocation(line: 3857, column: 23, scope: !477, inlinedAt: !597)
!602 = !DILocation(line: 50, column: 16, scope: !484, inlinedAt: !599)
!603 = !DILocation(line: 31, column: 17, scope: !495, inlinedAt: !604)
!604 = distinct !DILocation(line: 118, column: 39, scope: !497, inlinedAt: !605)
!605 = distinct !DILocation(line: 3667, column: 18, scope: !491, inlinedAt: !601)
!606 = !DILocation(line: 3666, column: 54, scope: !491, inlinedAt: !601)
!607 = !DILocation(line: 1320, column: 5, scope: !433, inlinedAt: !608)
!608 = distinct !DILocation(line: 61, column: 15, scope: !502, inlinedAt: !609)
!609 = distinct !DILocation(line: 3666, column: 34, scope: !491, inlinedAt: !601)
!610 = !{!611, !613, !571}
!611 = distinct !{!611, !612, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E: %_0"}
!612 = distinct !{!612, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E"}
!613 = distinct !{!613, !614, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E: %self"}
!614 = distinct !{!614, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$9next_impl17hc1f0c10073349e39E"}
!615 = !DILocation(line: 1563, column: 9, scope: !457, inlinedAt: !616)
!616 = distinct !DILocation(line: 111, column: 21, scope: !511, inlinedAt: !617)
!617 = distinct !DILocation(line: 118, column: 14, scope: !497, inlinedAt: !605)
!618 = !DILocation(line: 1072, column: 22, scope: !515, inlinedAt: !619)
!619 = distinct !DILocation(line: 495, column: 31, scope: !517, inlinedAt: !620)
!620 = distinct !DILocation(line: 3669, column: 35, scope: !491, inlinedAt: !601)
!621 = !DILocation(line: 863, column: 18, scope: !522, inlinedAt: !622)
!622 = distinct !DILocation(line: 3670, column: 45, scope: !491, inlinedAt: !601)
!623 = !DILocation(line: 2257, column: 25, scope: !424, inlinedAt: !575)
!624 = !DILocation(line: 113, column: 19, scope: !487, inlinedAt: !600)
!625 = !DILocation(line: 499, column: 18, scope: !527, inlinedAt: !626)
!626 = distinct !DILocation(line: 641, column: 51, scope: !533, inlinedAt: !627)
!627 = distinct !DILocation(line: 80, column: 21, scope: !535, inlinedAt: !628)
!628 = distinct !DILocation(line: 51, column: 18, scope: !484, inlinedAt: !599)
!629 = !DILocation(line: 641, column: 21, scope: !533, inlinedAt: !627)
!630 = !DILocation(line: 80, column: 13, scope: !535, inlinedAt: !628)
!631 = !DILocation(line: 38, column: 17, scope: !540, inlinedAt: !632)
!632 = distinct !DILocation(line: 114, column: 25, scope: !542, inlinedAt: !600)
!633 = !DILocation(line: 1072, column: 47, scope: !515, inlinedAt: !634)
!634 = distinct !DILocation(line: 495, column: 31, scope: !518, inlinedAt: !635)
!635 = distinct !DILocation(line: 3654, column: 39, scope: !490, inlinedAt: !601)
!636 = !DILocation(line: 1072, column: 22, scope: !515, inlinedAt: !634)
!637 = !DILocation(line: 3861, column: 9, scope: !548, inlinedAt: !597)
!638 = !DILocation(line: 805, column: 1, scope: !550, inlinedAt: !639)
!639 = distinct !DILocation(line: 805, column: 1, scope: !552, inlinedAt: !640)
!640 = distinct !DILocation(line: 1400, column: 18, scope: !554, inlinedAt: !641)
!641 = distinct !DILocation(line: 519, column: 23, scope: !556, inlinedAt: !642)
!642 = distinct !DILocation(line: 2260, column: 22, scope: !558, inlinedAt: !575)
!643 = !DILocation(line: 408, column: 29, scope: !345, inlinedAt: !644)
!644 = distinct !DILocation(line: 805, column: 1, scope: !561, inlinedAt: !645)
!645 = distinct !DILocation(line: 805, column: 1, scope: !563, inlinedAt: !646)
!646 = distinct !DILocation(line: 805, column: 1, scope: !550, inlinedAt: !639)
!647 = !DILocation(line: 2676, column: 9, scope: !648, inlinedAt: !649)
!648 = distinct !DISubprogram(name: "buckets", linkageName: "_ZN9hashbrown3raw13RawTableInner7buckets17h77230f7269c2183cE", scope: !425, file: !350, line: 2675, type: !13, scopeLine: 2675, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!649 = !DILocation(line: 3167, column: 82, scope: !650, inlinedAt: !651)
!650 = distinct !DISubprogram(name: "allocation_info", linkageName: "_ZN9hashbrown3raw13RawTableInner15allocation_info17he770ec60b358a6a2E", scope: !425, file: !350, line: 3160, type: !13, scopeLine: 3160, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!651 = !DILocation(line: 3136, column: 34, scope: !652, inlinedAt: !653)
!652 = distinct !DISubprogram(name: "free_buckets<alloc::alloc::Global>", linkageName: "_ZN9hashbrown3raw13RawTableInner12free_buckets17h314fcc3c0dd0f275E", scope: !425, file: !350, line: 3130, type: !13, scopeLine: 3130, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!653 = !DILocation(line: 2319, column: 22, scope: !565)
!654 = !DILocation(line: 2970, column: 26, scope: !655, inlinedAt: !658)
!655 = distinct !DISubprogram(name: "overflowing_mul", linkageName: "_ZN4core3num23_$LT$impl$u20$usize$GT$15overflowing_mul17h7acf0e86b6470d2bE", scope: !657, file: !656, line: 2969, type: !13, scopeLine: 2969, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!656 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs", directory: "", checksumkind: CSK_MD5, checksum: "13811679c51984d23ae828c0feb09156")
!657 = !DINamespace(name: "{impl#11}", scope: !531)
!658 = distinct !DILocation(line: 1080, column: 31, scope: !659, inlinedAt: !660)
!659 = distinct !DISubprogram(name: "checked_mul", linkageName: "_ZN4core3num23_$LT$impl$u20$usize$GT$11checked_mul17h68ad29d8cd96f176E", scope: !657, file: !656, line: 1079, type: !13, scopeLine: 1079, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!660 = distinct !DILocation(line: 224, column: 18, scope: !661, inlinedAt: !664)
!661 = distinct !DILexicalBlock(scope: !662, file: !350, line: 221, column: 9)
!662 = distinct !DISubprogram(name: "calculate_layout_for", linkageName: "_ZN9hashbrown3raw11TableLayout20calculate_layout_for17he05461673a20bd68E", scope: !663, file: !350, line: 218, type: !13, scopeLine: 218, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!663 = !DINamespace(name: "TableLayout", scope: !352)
!664 = distinct !DILocation(line: 3167, column: 56, scope: !650, inlinedAt: !651)
!665 = !DILocation(line: 458, column: 8, scope: !666, inlinedAt: !669)
!666 = distinct !DISubprogram(name: "unlikely", linkageName: "_ZN4core10intrinsics8unlikely17hadfd422a37d3f242E", scope: !668, file: !667, line: 457, type: !13, scopeLine: 457, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!667 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs", directory: "", checksumkind: CSK_MD5, checksum: "dd3192b00b8dfb3774128c2c0abf87d6")
!668 = !DINamespace(name: "intrinsics", scope: !22)
!669 = distinct !DILocation(line: 1081, column: 16, scope: !670, inlinedAt: !660)
!670 = distinct !DILexicalBlock(scope: !659, file: !656, line: 1080, column: 13)
!671 = !{!"branch_weights", !"expected", i32 1, i32 2000}
!672 = !DILocation(line: 224, column: 52, scope: !661, inlinedAt: !664)
!673 = !DILocation(line: 688, column: 37, scope: !674, inlinedAt: !675)
!674 = distinct !DISubprogram(name: "checked_add", linkageName: "_ZN4core3num23_$LT$impl$u20$usize$GT$11checked_add17h8221e944eae783b7E", scope: !657, file: !656, line: 680, type: !13, scopeLine: 680, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!675 = distinct !DILocation(line: 224, column: 40, scope: !661, inlinedAt: !664)
!676 = !DILocation(line: 458, column: 8, scope: !666, inlinedAt: !677)
!677 = distinct !DILocation(line: 688, column: 16, scope: !674, inlinedAt: !675)
!678 = !DILocation(line: 224, column: 71, scope: !661, inlinedAt: !664)
!679 = !DILocation(line: 224, column: 13, scope: !661, inlinedAt: !664)
!680 = !DILocation(line: 225, column: 43, scope: !681, inlinedAt: !664)
!681 = distinct !DILexicalBlock(scope: !661, file: !350, line: 223, column: 9)
!682 = !DILocation(line: 688, column: 37, scope: !674, inlinedAt: !683)
!683 = distinct !DILocation(line: 225, column: 31, scope: !681, inlinedAt: !664)
!684 = !DILocation(line: 458, column: 8, scope: !666, inlinedAt: !685)
!685 = distinct !DILocation(line: 688, column: 16, scope: !686, inlinedAt: !683)
!686 = !DILexicalBlockFile(scope: !674, file: !656, discriminator: 2)
!687 = !{!"branch_weights", i32 2002, i32 2000}
!688 = !DILocation(line: 132, column: 40, scope: !689, inlinedAt: !694)
!689 = distinct !DISubprogram(name: "from_size_align_unchecked", linkageName: "_ZN4core5alloc6layout6Layout25from_size_align_unchecked17h4054fff2886ce95bE", scope: !691, file: !690, line: 121, type: !13, scopeLine: 121, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!690 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/alloc/layout.rs", directory: "", checksumkind: CSK_MD5, checksum: "e0bac6553ae7e16e85f6bad44506df59")
!691 = !DINamespace(name: "Layout", scope: !692)
!692 = !DINamespace(name: "layout", scope: !693)
!693 = !DINamespace(name: "alloc", scope: !22)
!694 = distinct !DILocation(line: 234, column: 22, scope: !695, inlinedAt: !664)
!695 = distinct !DILexicalBlock(scope: !681, file: !350, line: 225, column: 9)
!696 = !DILocation(line: 237, column: 6, scope: !662, inlinedAt: !664)
!697 = !DILocation(line: 0, scope: !661, inlinedAt: !664)
!698 = !DILocation(line: 3167, column: 43, scope: !650, inlinedAt: !651)
!699 = !DILocation(line: 3167, column: 37, scope: !650, inlinedAt: !651)
!700 = !DILocation(line: 200, column: 12, scope: !701, inlinedAt: !705)
!701 = distinct !DISubprogram(name: "deallocate_impl_runtime", linkageName: "_ZN5alloc5alloc6Global23deallocate_impl_runtime17hb50d146c419ec31aE", scope: !703, file: !702, line: 199, type: !13, scopeLine: 199, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!702 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs", directory: "", checksumkind: CSK_MD5, checksum: "a10b93f2cf928b7124472720843a6712")
!703 = !DINamespace(name: "Global", scope: !704)
!704 = !DINamespace(name: "alloc", scope: !12)
!705 = distinct !DILocation(line: 324, column: 9, scope: !706, inlinedAt: !707)
!706 = distinct !DISubprogram(name: "deallocate_impl", linkageName: "_ZN5alloc5alloc6Global15deallocate_impl17h22049c3391b7dddfE", scope: !703, file: !702, line: 323, type: !13, scopeLine: 323, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!707 = distinct !DILocation(line: 442, column: 23, scope: !708, inlinedAt: !710)
!708 = distinct !DISubprogram(name: "deallocate", linkageName: "_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h05227860ec94ccf7E", scope: !709, file: !702, line: 440, type: !13, scopeLine: 440, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!709 = !DINamespace(name: "{impl#1}", scope: !704)
!710 = distinct !DILocation(line: 3137, column: 15, scope: !711, inlinedAt: !653)
!711 = distinct !DILexicalBlock(scope: !652, file: !350, line: 3136, column: 9)
!712 = !DILocation(line: 3173, column: 45, scope: !713, inlinedAt: !651)
!713 = distinct !DILexicalBlock(scope: !650, file: !350, line: 3167, column: 9)
!714 = !DILocation(line: 1072, column: 47, scope: !715, inlinedAt: !716)
!715 = distinct !DISubprogram(name: "sub<u8>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$3sub17h155b1930dacff52bE", scope: !26, file: !25, line: 1033, type: !13, scopeLine: 1033, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!716 = !DILocation(line: 3173, column: 64, scope: !713, inlinedAt: !651)
!717 = !DILocation(line: 1072, column: 22, scope: !715, inlinedAt: !716)
!718 = !DILocation(line: 115, column: 14, scope: !719, inlinedAt: !720)
!719 = distinct !DISubprogram(name: "dealloc", linkageName: "_ZN5alloc5alloc7dealloc17h248c9ee58f11063eE", scope: !704, file: !702, line: 114, type: !13, scopeLine: 114, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!720 = distinct !DILocation(line: 209, column: 22, scope: !701, inlinedAt: !705)
!721 = !DILocation(line: 200, column: 9, scope: !701, inlinedAt: !705)
!722 = !DILocation(line: 2322, column: 6, scope: !565)
!723 = !DILocation(line: 863, column: 18, scope: !473, inlinedAt: !724)
!724 = !DILocation(line: 3588, column: 24, scope: !447)
!725 = !DILocation(line: 1320, column: 5, scope: !433, inlinedAt: !726)
!726 = distinct !DILocation(line: 61, column: 15, scope: !439, inlinedAt: !727)
!727 = !DILocation(line: 3592, column: 29, scope: !446)
!728 = !{!729}
!729 = distinct !{!729, !730, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E: %_0"}
!730 = distinct !{!730, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E"}
!731 = !DILocation(line: 1563, column: 9, scope: !457, inlinedAt: !732)
!732 = distinct !DILocation(line: 111, column: 21, scope: !461, inlinedAt: !733)
!733 = !DILocation(line: 118, column: 14, scope: !464, inlinedAt: !734)
!734 = !DILocation(line: 3592, column: 62, scope: !446)
!735 = !DILocation(line: 31, column: 17, scope: !467, inlinedAt: !736)
!736 = !DILocation(line: 118, column: 39, scope: !464, inlinedAt: !734)
!737 = !DILocation(line: 863, column: 18, scope: !473, inlinedAt: !738)
!738 = !DILocation(line: 3593, column: 30, scope: !475)
!739 = !DILocation(line: 3595, column: 9, scope: !740)
!740 = distinct !DILexicalBlock(scope: !475, file: !350, line: 3593, column: 9)
!741 = !DILocation(line: 3601, column: 6, scope: !447)
!742 = !DILocation(line: 50, column: 32, scope: !484, inlinedAt: !743)
!743 = !DILocation(line: 113, column: 26, scope: !487, inlinedAt: !744)
!744 = !DILocation(line: 3653, column: 53, scope: !490)
!745 = !DILocation(line: 50, column: 16, scope: !484, inlinedAt: !743)
!746 = !DILocation(line: 31, column: 17, scope: !495, inlinedAt: !747)
!747 = !DILocation(line: 118, column: 39, scope: !497, inlinedAt: !748)
!748 = !DILocation(line: 3667, column: 18, scope: !491)
!749 = !DILocation(line: 0, scope: !491)
!750 = !DILocation(line: 3666, column: 13, scope: !491)
!751 = !DILocation(line: 3669, column: 13, scope: !491)
!752 = !DILocation(line: 113, column: 19, scope: !487, inlinedAt: !744)
!753 = !DILocation(line: 499, column: 18, scope: !527, inlinedAt: !754)
!754 = !DILocation(line: 641, column: 51, scope: !533, inlinedAt: !755)
!755 = !DILocation(line: 80, column: 21, scope: !535, inlinedAt: !756)
!756 = !DILocation(line: 51, column: 18, scope: !484, inlinedAt: !743)
!757 = !DILocation(line: 641, column: 21, scope: !533, inlinedAt: !755)
!758 = !DILocation(line: 80, column: 13, scope: !535, inlinedAt: !756)
!759 = !DILocation(line: 38, column: 17, scope: !540, inlinedAt: !760)
!760 = !DILocation(line: 114, column: 25, scope: !542, inlinedAt: !744)
!761 = !DILocation(line: 114, column: 9, scope: !542, inlinedAt: !744)
!762 = !DILocation(line: 495, column: 13, scope: !518, inlinedAt: !763)
!763 = !DILocation(line: 3654, column: 39, scope: !490)
!764 = !DILocation(line: 1072, column: 47, scope: !515, inlinedAt: !765)
!765 = !DILocation(line: 495, column: 31, scope: !518, inlinedAt: !763)
!766 = !DILocation(line: 1072, column: 22, scope: !515, inlinedAt: !765)
!767 = !DILocation(line: 3672, column: 6, scope: !491)
!768 = !DILocation(line: 3666, column: 54, scope: !491)
!769 = !DILocation(line: 1320, column: 5, scope: !433, inlinedAt: !770)
!770 = distinct !DILocation(line: 61, column: 15, scope: !502, inlinedAt: !771)
!771 = !DILocation(line: 3666, column: 34, scope: !491)
!772 = !{!773}
!773 = distinct !{!773, !774, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E: %_0"}
!774 = distinct !{!774, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h454e7a42ebc284a5E"}
!775 = !DILocation(line: 1563, column: 9, scope: !457, inlinedAt: !776)
!776 = distinct !DILocation(line: 111, column: 21, scope: !511, inlinedAt: !777)
!777 = !DILocation(line: 118, column: 14, scope: !497, inlinedAt: !748)
!778 = !DILocation(line: 1072, column: 22, scope: !515, inlinedAt: !779)
!779 = !DILocation(line: 495, column: 31, scope: !517, inlinedAt: !780)
!780 = !DILocation(line: 3669, column: 35, scope: !491)
!781 = !DILocation(line: 863, column: 18, scope: !522, inlinedAt: !782)
!782 = !DILocation(line: 3670, column: 45, scope: !491)
!783 = distinct !DISubprogram(name: "phage_target", scope: !89, file: !784, line: 6, type: !13, scopeLine: 6, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!784 = !DIFile(filename: "examples/nexagate-hpack-before.rs", directory: "/home/zero/Dev/Nexaaly/RuHealth", checksumkind: CSK_MD5, checksum: "c37f83b80e2c182a1e08fabbed3c9df9")
!785 = !DILocation(line: 22, column: 8, scope: !783)
!786 = !DILocation(line: 25, column: 9, scope: !783)
!787 = !DILocation(line: 25, column: 16, scope: !783)
!788 = !DILocation(line: 79, column: 17, scope: !789, inlinedAt: !794)
!789 = distinct !DISubprogram(name: "from", linkageName: "_ZN4core7convert3num65_$LT$impl$u20$core..convert..From$LT$u8$GT$$u20$for$u20$usize$GT$4from17h38581d55ae52cc82E", scope: !791, file: !790, line: 78, type: !13, scopeLine: 78, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!790 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/convert/num.rs", directory: "", checksumkind: CSK_MD5, checksum: "87ae55fcbe39b68ccbdcd352f6dbfc81")
!791 = !DINamespace(name: "{impl#68}", scope: !792)
!792 = !DINamespace(name: "num", scope: !793)
!793 = !DINamespace(name: "convert", scope: !22)
!794 = distinct !DILocation(line: 26, column: 37, scope: !795)
!795 = distinct !DILexicalBlock(scope: !783, file: !784, line: 25, column: 5)
!796 = !{!797}
!797 = distinct !{!797, !798, !"_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE: %input.0"}
!798 = distinct !{!798, !"_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE"}
!799 = !DILocation(line: 26, column: 11, scope: !795)
!800 = !DILocation(line: 156, column: 16, scope: !801, inlinedAt: !803)
!801 = distinct !DILexicalBlock(scope: !802, file: !127, line: 156, column: 35)
!802 = distinct !DISubprogram(name: "first<u8>", linkageName: "_ZN4core5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$5first17hb34d18d4784c2ae7E", scope: !128, file: !127, line: 155, type: !13, scopeLine: 155, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!803 = distinct !DILocation(line: 69, column: 10, scope: !804, inlinedAt: !805)
!804 = distinct !DISubprogram(name: "decode_int", linkageName: "_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE", scope: !88, file: !357, line: 67, type: !13, scopeLine: 67, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!805 = distinct !DILocation(line: 26, column: 11, scope: !795)
!806 = !DILocation(line: 0, scope: !807, inlinedAt: !811)
!807 = distinct !DISubprogram(name: "ok_or<&u8, nexagate_hpack_before::hpack::HpackError>", linkageName: "_ZN4core6option15Option$LT$T$GT$5ok_or17h564cd14c6b8e27ccE", scope: !809, file: !808, line: 1337, type: !13, scopeLine: 1337, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!808 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs", directory: "", checksumkind: CSK_MD5, checksum: "5a7d1d921bc51a881934fbca4e95b016")
!809 = !DINamespace(name: "Option", scope: !810)
!810 = !DINamespace(name: "option", scope: !22)
!811 = distinct !DILocation(line: 70, column: 10, scope: !804, inlinedAt: !805)
!812 = !DILocation(line: 68, column: 18, scope: !804, inlinedAt: !805)
!813 = !DILocation(line: 2189, column: 23, scope: !814, inlinedAt: !819)
!814 = distinct !DILexicalBlock(scope: !816, file: !815, line: 2189, column: 13)
!815 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs", directory: "", checksumkind: CSK_MD5, checksum: "1fd87bf9971f21fc110ff160efbaf9a6")
!816 = distinct !DISubprogram(name: "from_residual<(u64, usize), nexagate_hpack_before::hpack::HpackError, nexagate_hpack_before::hpack::HpackError>", linkageName: "_ZN153_$LT$core..result..Result$LT$T$C$F$GT$$u20$as$u20$core..ops..try_trait..FromResidual$LT$core..result..Result$LT$core..convert..Infallible$C$E$GT$$GT$$GT$13from_residual17h970bc59f43c58b19E", scope: !817, file: !815, line: 2187, type: !13, scopeLine: 2187, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!817 = !DINamespace(name: "{impl#28}", scope: !818)
!818 = !DINamespace(name: "result", scope: !22)
!819 = distinct !DILocation(line: 68, column: 18, scope: !820, inlinedAt: !805)
!820 = distinct !DILexicalBlock(scope: !821, file: !357, line: 70, column: 59)
!821 = distinct !DILexicalBlock(scope: !804, file: !357, line: 70, column: 59)
!822 = !DILocation(line: 0, scope: !823, inlinedAt: !805)
!823 = !DILexicalBlockFile(scope: !804, file: !784, discriminator: 0)
!824 = !DILocation(line: 68, column: 17, scope: !804, inlinedAt: !805)
!825 = !{!826}
!826 = distinct !{!826, !798, !"_ZN21nexagate_hpack_before5hpack10decode_int17h5a55be566d7011aaE: %_0"}
!827 = !DILocation(line: 71, column: 15, scope: !828, inlinedAt: !805)
!828 = distinct !DILexicalBlock(scope: !804, file: !357, line: 68, column: 5)
!829 = !DILocation(line: 79, column: 17, scope: !830, inlinedAt: !832)
!830 = distinct !DISubprogram(name: "from", linkageName: "_ZN4core7convert3num63_$LT$impl$u20$core..convert..From$LT$u8$GT$$u20$for$u20$u64$GT$4from17h13fb8097f6320c2eE", scope: !831, file: !790, line: 78, type: !13, scopeLine: 78, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!831 = !DINamespace(name: "{impl#66}", scope: !792)
!832 = distinct !DILocation(line: 72, column: 21, scope: !833, inlinedAt: !805)
!833 = distinct !DILexicalBlock(scope: !828, file: !357, line: 71, column: 5)
!834 = !DILocation(line: 72, column: 21, scope: !833, inlinedAt: !805)
!835 = !DILocation(line: 73, column: 8, scope: !836, inlinedAt: !805)
!836 = distinct !DILexicalBlock(scope: !833, file: !357, line: 72, column: 5)
!837 = !DILocation(line: 961, column: 18, scope: !838, inlinedAt: !839)
!838 = distinct !DISubprogram(name: "add<u8>", linkageName: "_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$3add17hf84a61f6dba8a914E", scope: !26, file: !25, line: 927, type: !13, scopeLine: 927, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!839 = distinct !DILocation(line: 102, column: 78, scope: !840, inlinedAt: !843)
!840 = distinct !DILexicalBlock(scope: !841, file: !43, line: 98, column: 9)
!841 = distinct !DILexicalBlock(scope: !842, file: !43, line: 97, column: 9)
!842 = distinct !DISubprogram(name: "new<u8>", linkageName: "_ZN4core5slice4iter13Iter$LT$T$GT$3new17h01d823508871c8cdE", scope: !38, file: !43, line: 96, type: !13, scopeLine: 96, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!843 = distinct !DILocation(line: 1041, column: 9, scope: !844, inlinedAt: !845)
!844 = distinct !DISubprogram(name: "iter<u8>", linkageName: "_ZN4core5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$4iter17h9554425a8dc85dc6E", scope: !128, file: !127, line: 1040, type: !13, scopeLine: 1040, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!845 = distinct !DILocation(line: 77, column: 34, scope: !846, inlinedAt: !805)
!846 = distinct !DILexicalBlock(scope: !836, file: !357, line: 76, column: 5)
!847 = !DILocation(line: 77, column: 5, scope: !848, inlinedAt: !805)
!848 = distinct !DILexicalBlock(scope: !846, file: !357, line: 77, column: 5)
!849 = !DILocation(line: 77, column: 23, scope: !846, inlinedAt: !805)
!850 = !DILocation(line: 0, scope: !833, inlinedAt: !805)
!851 = !DILocation(line: 1692, column: 9, scope: !852, inlinedAt: !854)
!852 = distinct !DISubprogram(name: "eq<u8>", linkageName: "_ZN78_$LT$core..ptr..non_null..NonNull$LT$T$GT$$u20$as$u20$core..cmp..PartialEq$GT$2eq17h9a2841cab9c505f9E", scope: !853, file: !30, line: 1691, type: !13, scopeLine: 1691, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!853 = !DINamespace(name: "{impl#16}", scope: !32)
!854 = distinct !DILocation(line: 180, column: 28, scope: !855, inlinedAt: !859)
!855 = distinct !DILexicalBlock(scope: !856, file: !35, line: 162, column: 17)
!856 = distinct !DILexicalBlock(scope: !857, file: !35, line: 161, column: 17)
!857 = distinct !DISubprogram(name: "next<u8>", linkageName: "_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h077ed6dddeb8f9cbE", scope: !858, file: !35, line: 157, type: !13, scopeLine: 157, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!858 = !DINamespace(name: "{impl#171}", scope: !39)
!859 = distinct !DILocation(line: 80, column: 27, scope: !860, inlinedAt: !864)
!860 = distinct !DISubprogram(name: "next<core::slice::iter::Iter<u8>>", linkageName: "_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE", scope: !862, file: !861, line: 79, type: !13, scopeLine: 79, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!861 = !DIFile(filename: "/home/zero/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/enumerate.rs", directory: "", checksumkind: CSK_MD5, checksum: "be8a5e8de3bc6e50de6cd50459716008")
!862 = !DINamespace(name: "{impl#1}", scope: !863)
!863 = !DINamespace(name: "enumerate", scope: !135)
!864 = distinct !DILocation(line: 77, column: 23, scope: !848, inlinedAt: !805)
!865 = !DILocation(line: 180, column: 28, scope: !855, inlinedAt: !859)
!866 = !DILocation(line: 2726, column: 15, scope: !867, inlinedAt: !869)
!867 = distinct !DISubprogram(name: "branch<&u8>", linkageName: "_ZN75_$LT$core..option..Option$LT$T$GT$$u20$as$u20$core..ops..try_trait..Try$GT$6branch17h1950b89f8b9f4812E", scope: !868, file: !808, line: 2725, type: !13, scopeLine: 2725, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition | DISPFlagOptimized, unit: !5, templateParams: !14)
!868 = !DINamespace(name: "{impl#47}", scope: !810)
!869 = distinct !DILocation(line: 80, column: 17, scope: !860, inlinedAt: !864)
!870 = !DILocation(line: 2726, column: 9, scope: !867, inlinedAt: !869)
!871 = !DILocation(line: 82, column: 9, scope: !872, inlinedAt: !864)
!872 = distinct !DILexicalBlock(scope: !873, file: !861, line: 81, column: 9)
!873 = distinct !DILexicalBlock(scope: !860, file: !861, line: 80, column: 9)
!874 = !DILocation(line: 84, column: 6, scope: !860, inlinedAt: !864)
!875 = !{!876, !826, !797}
!876 = distinct !{!876, !877, !"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE: %self"}
!877 = distinct !{!877, !"_ZN110_$LT$core..iter..adapters..enumerate..Enumerate$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h3293550d9724f57dE"}
!878 = !DILocation(line: 0, scope: !860, inlinedAt: !864)
!879 = !DILocation(line: 77, column: 23, scope: !848, inlinedAt: !805)
!880 = !DILocation(line: 77, column: 14, scope: !848, inlinedAt: !805)
!881 = !DILocation(line: 78, column: 18, scope: !882, inlinedAt: !805)
!882 = distinct !DILexicalBlock(scope: !848, file: !357, line: 77, column: 53)
!883 = !DILocation(line: 78, column: 28, scope: !882, inlinedAt: !805)
!884 = !DILocation(line: 79, column: 17, scope: !830, inlinedAt: !885)
!885 = distinct !DILocation(line: 78, column: 18, scope: !882, inlinedAt: !805)
!886 = !DILocation(line: 78, column: 9, scope: !882, inlinedAt: !805)
!887 = !{!826, !797}
!888 = !DILocation(line: 79, column: 12, scope: !882, inlinedAt: !805)
!889 = !DILocation(line: 82, column: 12, scope: !882, inlinedAt: !805)
!890 = !DILocation(line: 85, column: 9, scope: !882, inlinedAt: !805)
!891 = !DILocation(line: 83, column: 31, scope: !882, inlinedAt: !805)
!892 = !DILocation(line: 27, column: 30, scope: !893)
!893 = distinct !DILexicalBlock(scope: !795, file: !784, line: 27, column: 9)
!894 = !DILocation(line: 0, scope: !895, inlinedAt: !805)
!895 = !DILexicalBlockFile(scope: !882, file: !784, discriminator: 0)
!896 = !DILocation(line: 0, scope: !804, inlinedAt: !805)
!897 = !DILocation(line: 26, column: 5, scope: !795)
!898 = !DILocation(line: 0, scope: !795)
!899 = !DILocation(line: 30, column: 1, scope: !783)
!900 = !DILocation(line: 30, column: 2, scope: !783)
!901 = !DILocation(line: 0, scope: !783)
