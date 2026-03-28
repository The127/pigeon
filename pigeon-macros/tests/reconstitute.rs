#[test]
fn reconstitute_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/valid_struct.rs");
    t.compile_fail("tests/ui/tuple_struct.rs");
    t.compile_fail("tests/ui/enum_input.rs");
}
