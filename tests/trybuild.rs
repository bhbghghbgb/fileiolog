#[test]
fn trybuild_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/01-happy-path.rs");
    t.compile_fail("tests/trybuild/02-missing-etw-attr.rs");
    t.compile_fail("tests/trybuild/03-duplicate-id-version.rs");
    t.compile_fail("tests/trybuild/04-zero-event-attr.rs");
    t.compile_fail("tests/trybuild/05-multiple-event-attrs.rs");
    t.compile_fail("tests/trybuild/06-tuple-struct.rs");
}
