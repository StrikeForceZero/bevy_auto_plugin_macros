#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    #[cfg(feature = "nightly")]
    t.compile_fail("tests/ui/nightly/*.rs");
    #[cfg(feature = "stable")]
    t.compile_fail("tests/ui/stable/*.rs");
}
