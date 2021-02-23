// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use ocaml_interop::{
    ocaml_export, ocaml_unpack_polymorphic_variant, ocaml_unpack_variant, OCaml, OCamlBytes,
    OCamlFloat, OCamlInt, OCamlInt32, OCamlInt64, OCamlList, OCamlRef, ToOCaml,
};
use std::{thread, time};

enum Movement {
    Step { count: i32 },
    RotateLeft,
    RotateRight,
}

enum PolymorphicMovement {
    Step { count: i32 },
    RotateLeft,
    RotateRight,
}

ocaml_export! {
    fn rust_twice(cr, num: OCamlRef<OCamlInt>) -> OCaml<OCamlInt> {
        let num: i64 = num.to_rust(cr);
        unsafe { OCaml::of_i64_unchecked(num * 2) }
    }

    fn rust_twice_boxed_i64(cr, num: OCamlRef<OCamlInt64>) -> OCaml<OCamlInt64> {
        let num: i64 = num.to_rust(cr);
        let result = num * 2;
        result.to_ocaml(cr)
    }

    fn rust_twice_boxed_i32(cr, num: OCamlRef<OCamlInt32>) -> OCaml<OCamlInt32> {
        let num: i32 = num.to_rust(cr);
        let result = num * 2;
        result.to_ocaml(cr)
    }

    fn rust_add_unboxed_floats_noalloc(_cr, num: f64, num2: f64) -> f64 {
        num * num2
    }

    fn rust_twice_boxed_float(cr, num: OCamlRef<OCamlFloat>) -> OCaml<OCamlFloat> {
        let num: f64 = num.to_rust(cr);
        let result = num * 2.0;
        result.to_ocaml(cr)
    }

    fn rust_twice_unboxed_float(_cr, num: f64) -> f64 {
        num * 2.0
    }

    fn rust_increment_bytes(cr, bytes: OCamlRef<OCamlBytes>, first_n: OCamlRef<OCamlInt>) -> OCaml<OCamlBytes> {
        let first_n: i64 = first_n.to_rust(cr);
        let first_n = first_n as usize;
        let mut vec: Vec<u8> = bytes.to_rust(cr);

        for i in 0..first_n {
            vec[i] += 1;
        }

        vec.to_ocaml(cr)
    }

    fn rust_increment_ints_list(cr, ints: OCamlRef<OCamlList<OCamlInt>>) -> OCaml<OCamlList<OCamlInt>> {
        let mut vec: Vec<i64> = ints.to_rust(cr);

        for i in 0..vec.len() {
            vec[i] += 1;
        }

        vec.to_ocaml(cr)
    }

    fn rust_make_tuple(cr, fst: OCamlRef<String>, snd: OCamlRef<OCamlInt>) -> OCaml<(String, OCamlInt)> {
        let fst: String = fst.to_rust(cr);
        let snd: i64 = snd.to_rust(cr);
        let tuple = (fst, snd);
        tuple.to_ocaml(cr)
    }

    fn rust_make_some(cr, value: OCamlRef<String>) -> OCaml<Option<String>> {
        let value: String = value.to_rust(cr);
        let some_value = Some(value);
        some_value.to_ocaml(cr)
    }

    fn rust_make_ok(cr, value: OCamlRef<OCamlInt>) -> OCaml<Result<OCamlInt, String>> {
        let value: i64 = value.to_rust(cr);
        let ok_value: Result<i64, String> = Ok(value);
        ok_value.to_ocaml(cr)
    }

    fn rust_make_error(cr, value: OCamlRef<String>) -> OCaml<Result<OCamlInt, String>> {
        let value: String = value.to_rust(cr);
        let error_value: Result<i64, String> = Err(value);
        error_value.to_ocaml(cr)
    }

    fn rust_sleep_releasing(cr, millis: OCamlRef<OCamlInt>) {
        let millis: i64 = millis.to_rust(cr);
        cr.releasing_runtime(|| thread::sleep(time::Duration::from_millis(millis as u64)));
        OCaml::unit()
    }

    fn rust_sleep(cr, millis: OCamlRef<OCamlInt>) {
        let millis: i64 = millis.to_rust(cr);
        thread::sleep(time::Duration::from_millis(millis as u64));
        OCaml::unit()
    }

    fn rust_string_of_movement(cr, movement: OCamlRef<PolymorphicMovement>) -> OCaml<String> {
        let movement = cr.get(movement);
        let pm = ocaml_unpack_variant! {
            movement => {
                Step(count: OCamlInt) => { Movement::Step {count} },
                RotateLeft => Movement::RotateLeft,
                RotateRight => Movement::RotateRight,
            }
        };
        let s = match pm {
            Err(_) => "Error unpacking".to_owned(),
            Ok(Movement::Step {count}) => format!("Step({})", count),
            Ok(Movement::RotateLeft) => "RotateLeft".to_owned(),
            Ok(Movement::RotateRight) => "RotateRight".to_owned(),
        };
        s.to_ocaml(cr)
    }

    fn rust_string_of_polymorphic_movement(cr, polymorphic_movement: OCamlRef<PolymorphicMovement>) -> OCaml<String> {
        let polymorphic_movement = cr.get(polymorphic_movement);
        let pm = ocaml_unpack_polymorphic_variant! {
            polymorphic_movement => {
                Step(count: OCamlInt) => { PolymorphicMovement::Step {count} },
                RotateLeft => PolymorphicMovement::RotateLeft,
                RotateRight => PolymorphicMovement::RotateRight,
            }
        };
        let s = match pm {
            Err(_) => "Error unpacking".to_owned(),
            Ok(PolymorphicMovement::Step {count}) => format!("`Step({})", count),
            Ok(PolymorphicMovement::RotateLeft) => "`RotateLeft".to_owned(),
            Ok(PolymorphicMovement::RotateRight) => "`RotateRight".to_owned(),
        };
        s.to_ocaml(cr)
    }
}
