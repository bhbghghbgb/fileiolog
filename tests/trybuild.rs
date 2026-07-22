#[test]
fn trybuild_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/01-happy-path.rs");
    t.compile_fail("tests/trybuild/02-missing-etw-attr.rs");
    t.compile_fail("tests/trybuild/03-duplicate-id-version.rs");
    t.compile_fail("tests/trybuild/04-zero-event-attr.rs");
    t.pass("tests/trybuild/05-multiple-event-attrs.rs");
    t.compile_fail("tests/trybuild/06-tuple-struct.rs");
    t.compile_fail("tests/trybuild/07-skip-attr.rs");
    t.pass("tests/trybuild/08-optional-version.rs");
    t.pass("tests/trybuild/09-provider-attr.rs");
    t.pass("tests/trybuild/10-mask-all.rs");
    t.pass("tests/trybuild/11-mask-partial.rs");
    t.compile_fail("tests/trybuild/12-duplicate-versionless.rs");
    t.pass("tests/trybuild/13-etw-prop-convert.rs");
    t.compile_fail("tests/trybuild/14-convert-without-parse-as.rs");
    t.compile_fail("tests/trybuild/15-skip-prop.rs");
    t.pass("tests/trybuild/16-kernel-basic.rs");
    t.pass("tests/trybuild/17-kernel-provider-flag.rs");
    t.pass("tests/trybuild/18-user-provider-keyword.rs");
    t.compile_fail("tests/trybuild/19-kernel-keyword-mask-error.rs");
    t.compile_fail("tests/trybuild/20-user-enable-flag-error.rs");
    t.compile_fail("tests/trybuild/21-kernel-no-flag-error.rs");
    t.compile_fail("tests/trybuild/22-kernel-provider-keyword-mask-error.rs");
    t.compile_fail("tests/trybuild/23-user-provider-enable-flag-error.rs");
}
