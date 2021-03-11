// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::{
    conv::FromOCaml,
    mlvalues::{tag, OCamlBytes, OCamlFloat, OCamlInt32, OCamlInt64, OCamlList, RawOCaml},
    runtime::OCamlRuntime,
    value::OCaml,
};
use core::{cell::UnsafeCell, marker::PhantomData};
pub use ocaml_sys::{
    caml_alloc, local_roots as ocaml_sys_local_roots, set_local_roots as ocaml_sys_set_local_roots,
    store_field,
};
use ocaml_sys::{
    caml_alloc_string, caml_alloc_tuple, caml_copy_double, caml_copy_int32, caml_copy_int64,
    string_val,
};

pub struct OCamlCell<T> {
    cell: UnsafeCell<RawOCaml>,
    _marker: PhantomData<T>,
}

static_assertions::assert_eq_size!(OCamlCell<bool>, OCaml<'static, bool>, RawOCaml);

/// An `OCamlRef<T>` is a reference to a location containing a [`OCaml`]`<T>` value.
///
/// Usually obtained as the result of rooting an OCaml value.
pub type OCamlRef<'a, T> = &'a OCamlCell<T>;

impl<T> OCamlCell<T> {
    #[doc(hidden)]
    pub unsafe fn create_ref<'a>(val: *const RawOCaml) -> OCamlRef<'a, T> {
        &*(val as *const OCamlCell<T>)
    }

    /// Converts this value into a Rust value.
    pub fn to_rust<RustT>(&self, cr: &OCamlRuntime) -> RustT
    where
        RustT: FromOCaml<T>,
    {
        RustT::from_ocaml(cr.get(self))
    }

    /// Borrows the raw value contained in this root.
    ///
    /// # Safety
    ///
    /// The [`RawOCaml`] value obtained may become invalid after the OCaml GC runs.
    pub unsafe fn get_raw(&self) -> RawOCaml {
        *self.cell.get()
    }
}

pub fn alloc_bytes<'a>(cr: &'a mut OCamlRuntime, s: &[u8]) -> OCaml<'a, OCamlBytes> {
    unsafe {
        let len = s.len();
        let value = caml_alloc_string(len);
        let ptr = string_val(value);
        core::ptr::copy_nonoverlapping(s.as_ptr(), ptr, len);
        OCaml::new(cr, value)
    }
}

pub fn alloc_string<'a>(cr: &'a mut OCamlRuntime, s: &str) -> OCaml<'a, String> {
    unsafe {
        let len = s.len();
        let value = caml_alloc_string(len);
        let ptr = string_val(value);
        core::ptr::copy_nonoverlapping(s.as_ptr(), ptr, len);
        OCaml::new(cr, value)
    }
}

pub fn alloc_int32(cr: &mut OCamlRuntime, i: i32) -> OCaml<OCamlInt32> {
    unsafe { OCaml::new(cr, caml_copy_int32(i)) }
}

pub fn alloc_int64(cr: &mut OCamlRuntime, i: i64) -> OCaml<OCamlInt64> {
    unsafe { OCaml::new(cr, caml_copy_int64(i)) }
}

pub fn alloc_double(cr: &mut OCamlRuntime, d: f64) -> OCaml<OCamlFloat> {
    unsafe { OCaml::new(cr, caml_copy_double(d)) }
}

// TODO: it is possible to directly alter the fields memory upon first allocation of
// small values (like tuples and conses are) without going through `caml_modify` to get
// a little bit of extra performance.

pub fn alloc_some<'a, 'b, A>(
    cr: &'a mut OCamlRuntime,
    value: OCamlRef<'b, A>,
) -> OCaml<'a, Option<A>> {
    unsafe {
        let ocaml_some = caml_alloc(1, tag::SOME);
        store_field(ocaml_some, 0, value.get_raw());
        OCaml::new(cr, ocaml_some)
    }
}

pub unsafe fn alloc_tuple<T>(cr: &mut OCamlRuntime, size: usize) -> OCaml<T> {
    let ocaml_tuple = caml_alloc_tuple(size);
    OCaml::new(cr, ocaml_tuple)
}

pub fn alloc_cons<'a, 'b, A>(
    cr: &'a mut OCamlRuntime,
    head: OCamlRef<'b, A>,
    tail: OCamlRef<'b, OCamlList<A>>,
) -> OCaml<'a, OCamlList<A>> {
    unsafe {
        let ocaml_cons = caml_alloc(2, tag::CONS);
        store_field(ocaml_cons, 0, head.get_raw());
        store_field(ocaml_cons, 1, tail.get_raw());
        OCaml::new(cr, ocaml_cons)
    }
}
