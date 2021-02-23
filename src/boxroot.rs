// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::{marker::PhantomData, ops::Deref};

use ocaml_boxroot_sys::{
    boxroot_create, boxroot_delete, boxroot_get, boxroot_get_ref, boxroot_modify,
    BoxRoot as PrimitiveBoxRoot,
};

use crate::{memory::OCamlCell, OCaml, OCamlRef, OCamlRuntime, RawOCaml};

pub struct BoxRoot<T: 'static> {
    boxroot: PrimitiveBoxRoot,
    _marker: PhantomData<T>,
}

impl<T> BoxRoot<T> {
    /// Creates a new root from an [`OCaml`]`<T>` value.
    pub fn new<'a>(val: OCaml<'a, T>) -> BoxRoot<T> {
        BoxRoot {
            boxroot: unsafe { boxroot_create(val.raw) },
            _marker: PhantomData,
        }
    }

    /// Creates a new root from a [`RawOCaml`] value.
    ///
    /// # Safety
    ///
    /// The type of the value is not validated in any way.
    pub unsafe fn from_raw(raw: RawOCaml) -> BoxRoot<T> {
        BoxRoot {
            boxroot: boxroot_create(raw),
            _marker: PhantomData,
        }
    }

    /// Gets the value stored in this root as an [`OCaml`]`<T>`.
    pub fn get<'a>(&self, cr: &'a OCamlRuntime) -> OCaml<'a, T> {
        unsafe { OCaml::new(cr, boxroot_get(self.boxroot)) }
    }

    /// Gets the value stored in this root as a [`RawOCaml`].
    ///
    /// # Safety
    ///
    /// The [`RawOCaml`] value obtained may become invalid after the OCaml GC runs,
    /// and correct usage will not be enforced by the borrow checker.
    pub unsafe fn get_raw(&self) -> RawOCaml {
        boxroot_get(self.boxroot)
    }

    /// Roots the OCaml value `val`, returning an [`OCamlRef`]`<T>`.
    pub fn keep<'tmp>(&'tmp mut self, val: OCaml<T>) -> OCamlRef<'tmp, T> {
        unsafe {
            boxroot_modify(&mut self.boxroot, val.raw);
            &*(boxroot_get_ref(self.boxroot) as *const OCamlCell<T>)
        }
    }
}

impl<T> Drop for BoxRoot<T> {
    fn drop(&mut self) {
        unsafe { boxroot_delete(self.boxroot) }
    }
}

impl<T> Deref for BoxRoot<T> {
    type Target = OCamlCell<T>;

    fn deref(&self) -> OCamlRef<T> {
        unsafe { &*(boxroot_get_ref(self.boxroot) as *const OCamlCell<T>) }
    }
}
