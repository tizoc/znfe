// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use core::str;
use ocaml_sys::{caml_alloc, store_field};

use crate::{
    memory::{
        alloc_bytes, alloc_cons, alloc_double, alloc_int32, alloc_int64, alloc_some, alloc_string,
        alloc_tuple, alloc_tuple_3, alloc_tuple_4, OCamlRef,
    },
    mlvalues::{
        tag, OCamlBytes, OCamlFloat, OCamlInt, OCamlInt32, OCamlInt64, OCamlList, RawOCaml, FALSE,
        NONE, TRUE,
    },
    runtime::OCamlRuntime,
    value::OCaml,
    BoxRoot,
};

/// Implements conversion from Rust values into OCaml values.
pub unsafe trait ToOCaml<T> {
    /// Convert to OCaml value. Return an already rooted value as [`BoxRoot`]`<T>`.
    fn to_boxroot(&self, cr: &mut OCamlRuntime) -> BoxRoot<T> {
        BoxRoot::new(self.to_ocaml(cr))
    }

    /// Convert to OCaml value.
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, T>;
}

unsafe impl<'root, T> ToOCaml<T> for OCamlRef<'root, T> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, T> {
        unsafe { OCaml::new(cr, self.get_raw()) }
    }
}

unsafe impl ToOCaml<OCamlInt> for i64 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt> {
        unsafe { OCaml::new(cr, ((self << 1) | 1) as RawOCaml) }
    }
}

unsafe impl ToOCaml<OCamlInt> for i32 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt> {
        (*self as i64).to_ocaml(cr)
    }
}

unsafe impl ToOCaml<OCamlInt32> for i32 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt32> {
        alloc_int32(cr, *self)
    }
}

unsafe impl ToOCaml<OCamlInt64> for i64 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt64> {
        alloc_int64(cr, *self)
    }
}

unsafe impl ToOCaml<OCamlFloat> for f64 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlFloat> {
        alloc_double(cr, *self)
    }
}

unsafe impl ToOCaml<bool> for bool {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, bool> {
        unsafe { OCaml::new(cr, if *self { TRUE } else { FALSE }) }
    }
}

// TODO: impl using Borrow trait instead?
unsafe impl ToOCaml<String> for &str {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        alloc_string(cr, self)
    }
}

unsafe impl ToOCaml<OCamlBytes> for &str {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        alloc_bytes(cr, self.as_bytes())
    }
}

unsafe impl ToOCaml<OCamlBytes> for &[u8] {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        alloc_bytes(cr, self)
    }
}

unsafe impl ToOCaml<String> for &[u8] {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        alloc_string(cr, unsafe { str::from_utf8_unchecked(self) })
    }
}

unsafe impl ToOCaml<String> for String {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        self.as_str().to_ocaml(cr)
    }
}

unsafe impl ToOCaml<OCamlBytes> for String {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        self.as_str().to_ocaml(cr)
    }
}

unsafe impl ToOCaml<String> for Vec<u8> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        self.as_slice().to_ocaml(cr)
    }
}

unsafe impl ToOCaml<OCamlBytes> for Vec<u8> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        self.as_slice().to_ocaml(cr)
    }
}

unsafe impl<A, OCamlA> ToOCaml<OCamlA> for Box<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlA> {
        self.as_ref().to_ocaml(cr)
    }
}

unsafe impl<A, OCamlA: 'static> ToOCaml<Option<OCamlA>> for Option<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, Option<OCamlA>> {
        if let Some(value) = self {
            let ocaml_value = value.to_boxroot(cr);
            alloc_some(cr, &ocaml_value)
        } else {
            unsafe { OCaml::new(cr, NONE) }
        }
    }
}

unsafe impl<A, OCamlA, Err, OCamlErr> ToOCaml<Result<OCamlA, OCamlErr>> for Result<A, Err>
where
    A: ToOCaml<OCamlA>,
    Err: ToOCaml<OCamlErr>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, Result<OCamlA, OCamlErr>> {
        match self {
            Ok(value) => {
                let ocaml_ok = unsafe { caml_alloc(1, tag::TAG_OK) };
                let ocaml_value = value.to_ocaml(cr);
                unsafe { store_field(ocaml_ok, 0, ocaml_value.get_raw()) };
                unsafe { OCaml::new(cr, ocaml_ok) }
            }
            Err(error) => {
                let ocaml_err = unsafe { caml_alloc(1, tag::TAG_ERROR) };
                let ocaml_error = error.to_ocaml(cr);
                unsafe { store_field(ocaml_err, 0, ocaml_error.get_raw()) };
                unsafe { OCaml::new(cr, ocaml_err) }
            }
        }
    }
}

unsafe impl<A, B, OCamlA: 'static, OCamlB: 'static> ToOCaml<(OCamlA, OCamlB)> for (A, B)
where
    A: ToOCaml<OCamlA>,
    B: ToOCaml<OCamlB>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, (OCamlA, OCamlB)> {
        let fst = self.0.to_boxroot(cr);
        let snd = self.1.to_boxroot(cr);
        alloc_tuple(cr, &fst, &snd)
    }
}

unsafe impl<A, B, C, OCamlA: 'static, OCamlB: 'static, OCamlC: 'static>
    ToOCaml<(OCamlA, OCamlB, OCamlC)> for (A, B, C)
where
    A: ToOCaml<OCamlA>,
    B: ToOCaml<OCamlB>,
    C: ToOCaml<OCamlC>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, (OCamlA, OCamlB, OCamlC)> {
        let fst = self.0.to_boxroot(cr);
        let snd = self.1.to_boxroot(cr);
        let elt3 = self.2.to_boxroot(cr);
        alloc_tuple_3(cr, &fst, &snd, &elt3)
    }
}

unsafe impl<A, B, C, D, OCamlA: 'static, OCamlB: 'static, OCamlC: 'static, OCamlD: 'static>
    ToOCaml<(OCamlA, OCamlB, OCamlC, OCamlD)> for (A, B, C, D)
where
    A: ToOCaml<OCamlA>,
    B: ToOCaml<OCamlB>,
    C: ToOCaml<OCamlC>,
    D: ToOCaml<OCamlD>,
{
    fn to_ocaml<'a>(
        &self,
        cr: &'a mut OCamlRuntime,
    ) -> OCaml<'a, (OCamlA, OCamlB, OCamlC, OCamlD)> {
        let fst = self.0.to_boxroot(cr);
        let snd = self.1.to_boxroot(cr);
        let elt3 = self.2.to_boxroot(cr);
        let elt4 = self.3.to_boxroot(cr);
        alloc_tuple_4(cr, &fst, &snd, &elt3, &elt4)
    }
}

unsafe impl<A, OCamlA: 'static> ToOCaml<OCamlList<OCamlA>> for Vec<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlList<OCamlA>> {
        (&self).to_ocaml(cr)
    }
}

unsafe impl<A, OCamlA: 'static> ToOCaml<OCamlList<OCamlA>> for &Vec<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlList<OCamlA>> {
        let mut result = BoxRoot::new(OCaml::nil());
        for elt in self.iter().rev() {
            let ov = elt.to_boxroot(cr);
            let cons = alloc_cons(cr, &ov, &result);
            result.keep(cons);
        }
        cr.get(&result)
    }
}
