//! Each feature compiles one exact Phage property for Kani. The bytes,
//! domain exclusions and function bodies are shared rather than translated.
#![allow(dead_code, unused_imports)]

#[cfg(not(any(
    feature = "quic",
    feature = "hpack",
    feature = "old-hpack",
    feature = "assert-forms",
    feature = "false-assert",
    feature = "const-generics",
    feature = "string-map",
    feature = "string-map-bad"
)))]
compile_error!("select one comparison feature to avoid an empty proof");

#[cfg(feature = "quic")]
#[path = "../../examples/nexagate-quic.rs"]
mod property;

#[cfg(any(feature = "string-map", feature = "string-map-bad"))]
#[path = "../../examples/string-map.rs"]
mod property;
#[cfg(feature = "hpack")]
#[path = "../../examples/nexagate-hpack-after.rs"]
mod property;
#[cfg(feature = "old-hpack")]
#[path = "../../examples/nexagate-hpack-before.rs"]
mod property;
#[cfg(feature = "assert-forms")]
#[path = "../../examples/issue-4874-assert-forms.rs"]
mod property;
#[cfg(feature = "false-assert")]
#[path = "../../examples/issue-4874-false-assert.rs"]
mod property;
#[cfg(feature = "const-generics")]
#[path = "../../examples/issue-4876-const-generics.rs"]
mod property;

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(32)]
fn shared_property() {
    #[cfg(feature = "quic")]
    {
        let b: [u8; 8] = kani::any();
        assert!(property::phage_target(
            b[0],
            b[1],
            b[2],
            b[3],
            b[4],
            b[5],
            b[6],
            b[7],
            kani::any()
        ));
    }
    #[cfg(any(feature = "hpack", feature = "old-hpack"))]
    {
        let b: [u8; 12] = kani::any();
        assert!(property::phage_target(
            b[0],
            b[1],
            b[2],
            b[3],
            b[4],
            b[5],
            b[6],
            b[7],
            b[8],
            b[9],
            b[10],
            b[11],
            kani::any(),
            kani::any()
        ));
    }
    #[cfg(any(
        feature = "assert-forms",
        feature = "false-assert",
        feature = "const-generics"
    ))]
    assert!(property::phage_target(kani::any()));
    #[cfg(feature = "string-map")]
    assert!(property::phage_target(kani::any(), kani::any()));
    #[cfg(feature = "string-map-bad")]
    assert!(property::string_map_bad(kani::any(), kani::any()));
}
