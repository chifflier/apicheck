#[macro_use] extern crate pretty_assertions;

extern crate json;

mod utils;

#[test]
fn run_all() {
    utils::run_test("01").expect("test 01");
    utils::run_test("02").expect("test 02");
    utils::run_test("03").expect("test 03");
    utils::run_test("04").expect("test 04");
    utils::run_test("05").expect("test 05");
    utils::run_test("enums_01").expect("test enums_01");
    utils::run_test("enums_02").expect("test enums_02");
    utils::run_test("functions").expect("test functions");
    utils::run_test("mods").expect("test mods");
    utils::run_test("mods_02").expect("test mods_02");
    utils::run_test("structs_01").expect("test structs_01");
    utils::run_test("structs_02").expect("test structs_02");
    utils::run_test("traits").expect("test traits");
}
