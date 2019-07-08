#[macro_use] extern crate pretty_assertions;

extern crate json;

mod utils;

#[test]
fn apicheck_run_all() {
    utils::run_check_test("01").expect("test 01");
    utils::run_check_test("02").expect("test 02");
    utils::run_check_test("03").expect("test 03");
    utils::run_check_test("04").expect("test 04");
    utils::run_check_test("05").expect("test 05");
    utils::run_check_test("const_fn").expect("test const_fn");
    utils::run_check_test("enums_01").expect("test enums_01");
    utils::run_check_test("enums_02").expect("test enums_02");
    utils::run_check_test("functions").expect("test functions");
    utils::run_check_test("mods").expect("test mods");
    utils::run_check_test("mods_02").expect("test mods_02");
    utils::run_check_test("structs_01").expect("test structs_01");
    utils::run_check_test("structs_02").expect("test structs_02");
    utils::run_check_test("traits").expect("test traits");
}

#[test]
fn apidiff_run_all() {
    utils::run_diff_test("01","01",0).expect("diff 01/01");
    utils::run_diff_test("01","02",1).expect("diff 01/02");
    utils::run_diff_test("02","03",0).expect("diff 02/03");
    utils::run_diff_test("03","04",1).expect("diff 03/04");
}
