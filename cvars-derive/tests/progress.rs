#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/test_parse.rs");
    t.pass("tests/test_getters.rs");
    t.pass("tests/test_setters.rs");
    t.pass("tests/test_string_setters.rs");
}
