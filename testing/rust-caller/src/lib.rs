extern crate znfe;

use lazy_static::lazy_static;
use znfe::{
    alloc_ocaml, call_ocaml, with_frame, FromOCaml, Intnat, OCaml, OCamlClosure, ToOCaml,
    ToOCamlInteger,
};

lazy_static! {
    static ref OCAML_TWICE: OCamlClosure =
        OCamlClosure::named("twice").expect("Missing 'twice' function");
    static ref OCAML_INCREMENT_BYTES: OCamlClosure =
        OCamlClosure::named("increment_bytes").expect("Missing 'increment_bytes' function");
}

pub fn increment_bytes(bytes: &str, first_n: usize) -> String {
    let res = with_frame(|gc| {
        let bytes = alloc_ocaml! {bytes.to_ocaml(gc)};
        let bytes_ref = bytes.reference(gc);
        let first_n = alloc_ocaml! {(first_n as i64).to_ocaml(gc)};
        let result = call_ocaml! {OCAML_INCREMENT_BYTES(gc, bytes_ref.get(gc), first_n)};
        let result: OCaml<String> = result.expect("Error in 'increment_bytes' call result");
        result.into()
    });

    unsafe { String::from_raw_ocaml(res) }
}

pub fn twice(num: i64) -> i64 {
    let res = with_frame(|gc| {
        let num = num.to_ocaml_fixnum();
        let result = call_ocaml! {OCAML_TWICE(gc, num)};
        let result: OCaml<Intnat> = result.expect("Error in 'twice' call result");
        result.into()
    });

    unsafe { i64::from_raw_ocaml(res) }
}

// Tests

// NOTE: required because at the moment, no synchronization is done on OCaml calls
#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_twice() {
    znfe::init_runtime();
    assert_eq!(twice(10), 20);
}

#[test]
#[serial]
fn test_increment_bytes() {
    znfe::init_runtime();
    assert_eq!(increment_bytes("0000000000000000", 10), "1111111111000000");
}