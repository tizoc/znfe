// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use ocaml_sys::{caml_register_global_root, caml_remove_global_root};
use std::{cell::UnsafeCell, marker::PhantomData, ops::Deref};

use crate::{memory::OCamlCell, OCaml, OCamlRef, OCamlRuntime};

pub struct BoxRoot<T: 'static>(Box<UnsafeCell<OCaml<'static, T>>>);

impl<T> BoxRoot<T> {
    pub fn new<'a>(v: OCaml<'a, T>) -> BoxRoot<T> {
        BoxRoot(unsafe {
            let r = Box::new(UnsafeCell::new(OCaml {
                raw: v.raw,
                _marker: PhantomData,
            }));

            // Immediate values don't need to be registered
            if v.is_block() {
                caml_register_global_root(r.get() as *mut isize);
            };

            r
        })
    }

    pub fn get<'a>(&self, _cr: &'a OCamlRuntime) -> OCaml<'a, T> {
        unsafe { *(*self.0).get() }
    }

    /// Roots an [`OCaml`] value.
    pub fn keep<'tmp>(&'tmp mut self, val: OCaml<T>) -> OCamlRef<'tmp, T> {
        unsafe {
            let cell = self.0.get();
            *cell = OCaml {
                raw: val.raw,
                _marker: PhantomData,
            };
            &*(cell as *const OCamlCell<T>)
        }
    }
}

impl<T> Drop for BoxRoot<T> {
    fn drop(&mut self) {
        unsafe {
            caml_remove_global_root(self.0.get() as *mut isize);
        };
    }
}

impl<T> Deref for BoxRoot<T> {
    type Target = OCamlCell<T>;

    fn deref(&self) -> OCamlRef<T> {
        let cell: &UnsafeCell<OCaml<'static, T>> = &self.0;
        unsafe { std::mem::transmute(cell) }
    }
}
