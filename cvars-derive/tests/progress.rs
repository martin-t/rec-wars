#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/parse.rs");
    t.pass("tests/getters.rs");
    // TODO commented out lines in all tests
}
