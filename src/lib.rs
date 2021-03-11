// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

#![doc(html_root_url = "https://docs.rs/ocaml-interop/0.5.3")]

//! _Zinc-iron alloy coating is used in parts that need very good corrosion protection._
//!
//! **API IS CONSIDERED UNSTABLE AT THE MOMENT AND IS LIKELY TO CHANGE IN THE FUTURE**
//!
//! [ocaml-interop](https://github.com/simplestaking/ocaml-interop) is an OCaml<->Rust FFI with an emphasis on safety inspired by [caml-oxide](https://github.com/stedolan/caml-oxide) and [ocaml-rs](https://github.com/zshipko/ocaml-rs).
//!
//! ## Table of Contents
//!
//! - [How does it work](#how-does-it-work)
//! - [Usage](#usage)
//!   * [Rules](#rules)
//!     + [Rule 1: The OCaml runtime handle](#rule-1-the-ocaml-runtime-handle)
//!     + [Rule 2: Liveness of OCaml values and rooting](#rule-2-liveness-of-ocaml-values-and-rooting)
//!     + [Rule 3: Liveness and scope of OCaml roots](#rule-3-liveness-and-scope-ocaml-roots)
//!   * [Converting between OCaml and Rust data](#converting-between-ocaml-and-rust-data)
//!     + [`FromOCaml` trait](#fromocaml-trait)
//!     + [`ToOCaml` trait](#toocaml-trait)
//!   * [Calling convention](#calling-convention)
//!   * [OCaml exceptions](#ocaml-exceptions)
//!   * [Calling into OCaml from Rust](#calling-into-ocaml-from-rust)
//!   * [Calling into Rust from OCaml](#calling-into-rust-from-ocaml)
//! - [References and links](#references-and-links)
//!
//! ## How does it work
//!
//! ocaml-interop, just like [caml-oxide](https://github.com/stedolan/caml-oxide), encodes the invariants of OCaml's garbage collector into the rules of Rust's borrow checker. Any violation of these invariants results in a compilation error produced by Rust's borrow checker.
//!
//! ## Usage
//!
//! ### Rules
//!
//! There are a few rules that have to be followed when calling into the OCaml runtime:
//!
//! #### Rule 1: The OCaml runtime handle
//!
//! To interact with the OCaml runtime (be it reading, mutating or allocating values, or to call functions) a reference to the OCaml runtime handle must be available.
//! Any function that interacts with the OCaml runtime takes as a first argument a reference to the OCaml runtime handle.
//!
//! #### Rule 2: Liveness of OCaml values and rooting
//!
//! Rust references to OCaml values become stale after calls into the OCaml runtime and cannot be used again. This is enforced by Rust's borrow checker.
//!
//! To have OCaml values survive across calls into the OCaml runtime, they have to be rooted, and then recovered from a root.
//!
//! Rooting is only possible inside `ocaml_frame!` blocks, which initialize a list of root variables that can be used to root OCaml values.
//!
//! #### Rule 3: Liveness and scope of OCaml roots
//!
//! OCaml roots are only valid inside the [`ocaml_frame!`] that instantiated them, and cannot escape this scope.
//!
//! Example (fails to compile):
//!
//! ```rust,compile_fail
//! # use ocaml_interop::*;
//! # ocaml! {
//! #     fn ocaml_function(arg1: String) -> String;
//! #     fn another_ocaml_function(arg: String);
//! # }
//! # let a_string = "string";
//! # let arg1 = "arg1";
//! # let cr = unsafe { &mut OCamlRuntime::recover_handle() };
//! let escape = ocaml_frame!(cr, (arg1_root), {
//!     let arg1 = arg1.to_boxroot(cr);
//!     let arg1_root = arg1_root.keep(arg1);
//!     let result = ocaml_function(cr, arg1_root, /* ..., argN */);
//!     let s: String = result.to_rust();
//!     // ...
//!     arg1_root
//! });
//! ```
//!
//! In the above example `arg1_root` cannot escape the [`ocaml_frame!`] scope, Rust's borrow checker will complain:
//!
//! ```text,no_run
//! error[E0716]: temporary value dropped while borrowed
//!   --> src/lib.rs:64:14
//!    |
//!    |   let escape = ocaml_frame!(cr, (arg1_root), {
//!    |  _____------___^
//!    | |     |
//!    | |     borrow later stored here
//!    | |     let arg1 = arg1.to_boxroot(cr);
//!    | |     let arg1_root = arg1_root.keep(arg1);
//!    | |     let result = ocaml_function(cr, arg1_root, /* ..., argN */);
//! ...  |
//!    | |     arg1_root
//!    | | });
//!    | |  ^
//!    | |  |
//!    | |__creates a temporary which is freed while still in use
//!    |    temporary value is freed at the end of this statement
//! ```
//!
//! A similar error would happen if a root-variable escaped the frame scope.
//!
//! ### Converting between OCaml and Rust data
//!
//! #### [`FromOCaml`] trait
//!
//! The [`FromOCaml`] trait implements conversion from OCaml values into Rust values, using the `from_ocaml` function.
//!
//! [`OCaml`]`<T>` values have a `to_rust()` method that is usually more convenient than `Type::from_ocaml(ocaml_value)`, and works for any combination that implements the `FromOCaml` trait.
//!
//! [`OCamlRef`]`<T>` values have a `to_rust(cr)` that needs an [`OCamlRuntime`] reference to be passed to it.
//!
//! #### [`ToOCaml`] trait
//!
//! The [`ToOCaml`] trait implements conversion from Rust values into OCaml values, using the `to_ocaml` method. It takes a single parameter that must be a `&mut OCamlRuntime`.
//!
//! A more convenient way to convert Rust values into OCaml values is provided by the [`to_ocaml!`] macro that accepts a root variable as an optional third argument to return a root containing the value.
//!
//! ### Calling convention
//!
//! There are two possible calling conventions in regards to rooting, one with *callee rooted arguments*, and another with *caller rooted arguments*.
//!
//! #### Callee rooted arguments calling convention
//!
//! With this calling convention, values that are arguments to a function call are passed directly. Functions that receive arguments are responsible for rooting them. This is how OCaml's C API and `ocaml-interop` versions before `0.5.0` work.
//!
//! #### Caller rooted arguments calling convention
//!
//! With this calling convention, values that are arguments to a function call must be rooted by the caller. Then instead of the value, it is the root pointing to the value that is passed as an argument. This is how `ocaml-interop` works starting with version `0.5.0`.
//!
//! When a Rust function is called from OCaml, it will receive arguments as `OCamlRef<T>` values, and when a OCaml function is called from Rust, arguments will be passed as `OCamlRef<T>` values.
//!
//! ### OCaml exceptions
//!
//! If an OCaml function called from Rust raises an exception, this will result in a panic.
//!
//! OCaml functions meant to be called from Rust should not raise exceptions to signal errors, but instead return `result` or `option` values, which can then be mapped into `Result` and `Option` values in Rust.
//!
//! ### Calling into OCaml from Rust
//!
//! The following code defines two OCaml functions and registers them using the `Callback.register` mechanism:
//!
//! ```ocaml
//! let increment_bytes bytes first_n =
//!   let limit = (min (Bytes.length bytes) first_n) - 1 in
//!   for i = 0 to limit do
//!     let value = (Bytes.get_uint8 bytes i) + 1 in
//!     Bytes.set_uint8 bytes i value
//!   done;
//!   bytes
//!
//! let twice x = 2 * x
//!
//! let () =
//!   Callback.register "increment_bytes" increment_bytes;
//!   Callback.register "twice" twice
//! ```
//!
//! To be able to call these from Rust, there are a few things that need to be done:
//!
//! - The OCaml runtime has to be initialized. If the driving program is a Rust application, it has to be done explicitly by doing `let runtime = OCamlRuntime::init()`, but if the driving program is an OCaml application, this is not required.
//! - Functions that were exported from the OCaml side with `Callback.register` have to be declared using the [`ocaml!`] macro.
//! - Before the program exist, or once the OCaml runtime is not required anymore, it has to be de-initialized by calling the `shutdown()` method on the OCaml runtime handle.
//!
//! ### Example
//!
//! ```rust,no_run
//! use ocaml_interop::{
//!     BoxRoot, FromOCaml, OCaml, OCamlRef, ToOCaml, OCamlRuntime
//! };
//!
//! // To call an OCaml function, it first has to be declared inside an `ocaml!` macro block:
//! mod ocaml_funcs {
//!     use ocaml_interop::{ocaml, OCamlInt};
//!
//!     ocaml! {
//!         // OCaml: `val increment_bytes: bytes -> int -> bytes`
//!         // registered with `Callback.register "increment_bytes" increment_bytes`
//!         pub fn increment_bytes(bytes: String, first_n: OCamlInt) -> String;
//!         // OCaml: `val twice: int -> int`
//!         // registered with `Callback.register "twice" twice`
//!         pub fn twice(num: OCamlInt) -> OCamlInt;
//!     }
//! }
//!
//! fn increment_bytes(
//!     cr: &mut OCamlRuntime,
//!     bytes1: String,
//!     bytes2: String,
//!     first_n: usize,
//! ) -> (String, String) {
//!     // Any calls into the OCaml runtime takes as input a `&mut` reference to an `OCamlRuntime`
//!     // value that is obtained as the result of initializing the OCaml runtime.
//!     // If rooting of OCaml values is needed, a new frame has to be opened by using the
//!     // `ocaml_frame!` macro.
//!     // The first argument to the macro is a reference to an `OCamlRuntime`, followed by a
//!     // list of "root variables" (more on this later). The last argument
//!     // is the block of code that will run inside that frame.
//!     // The `ToOCaml` trait provides the `to_ocaml` method to convert Rust
//!     // values into OCaml values.
//!     let ocaml_bytes1: BoxRoot<String> = bytes1.to_boxroot(cr);
//!
//!     // Same as above. Here the convenience macro [`to_ocaml!`] is used.
//!     // It works like `value.to_boxroot(cr)`, but has an optional third argument that
//!     // can be a root variable to perform the rooting.
//!     // This variation returns an `OCamlRef` value instead of an `OCaml` one.
//!     let bytes2_root = bytes2.to_boxroot(cr);
//!
//!     // Rust `i64` integers can be converted into OCaml fixnums with `OCaml::of_i64`
//!     // and `OCaml::of_i64_unchecked`.
//!     // Such conversion doesn't require any allocation on the OCaml side, and doesn't
//!     // invalidate other `OCaml<T>` values.
//!     let ocaml_first_n = unsafe { OCaml::of_i64_unchecked(first_n as i64) };
//!
//!     // Any OCaml function (declared above in a `ocaml!` block) can be called as a regular
//!     // Rust function, by passing a `&mut OCamlRuntime` as the first argument, followed by
//!     // the rest of the arguments declared for that function.
//!     // Arguments to these functions must be references to roots: `OCamlRef<T>`
//!     let result1 = ocaml_funcs::increment_bytes(
//!         cr,             // &mut OCamlRuntime
//!         &ocaml_bytes1,    // OCamlRef<String>
//!         // Immediate OCaml values, such as ints and books have an as_value_ref() method
//!         // that can be used to simulate rooting.
//!         &ocaml_first_n, // OCamlRef<OCamlInt>
//!     );
//!
//!     // Perform the conversion of the OCaml result value into a
//!     // Rust value while the reference is still valid because the
//!     // call that follows will invalidate it.
//!     // Alternatively, the result of `rootvar.keep(result1)` could be used
//!     // to be able to reference the value later through an `OCamlRef` value.
//!     let new_bytes1: String = result1.to_rust(cr);
//!     let result2 = ocaml_funcs::increment_bytes(
//!         cr,
//!         &bytes2_root,
//!         &ocaml_first_n,
//!     );
//!
//!     (new_bytes1, result2.to_rust(cr))
//! }
//!
//! fn twice(cr: &mut OCamlRuntime, num: usize) -> usize {
//!     let ocaml_num = unsafe { OCaml::of_i64_unchecked(num as i64) };
//!     let result = ocaml_funcs::twice(cr, &ocaml_num);
//!     result.to_rust::<i64>(cr) as usize
//! }
//!
//! fn entry_point() {
//!     // IMPORTANT: the OCaml runtime has to be initialized first.
//!     let mut cr = OCamlRuntime::init();
//!     // `cr` is the OCaml runtime handle, must be passed to any function
//!     // that interacts with the OCaml runtime.
//!     let first_n = twice(&mut cr, 5);
//!     let bytes1 = "000000000000000".to_owned();
//!     let bytes2 = "aaaaaaaaaaaaaaa".to_owned();
//!     println!("Bytes1 before: {}", bytes1);
//!     println!("Bytes2 before: {}", bytes2);
//!     let (result1, result2) = increment_bytes(&mut cr, bytes1, bytes2, first_n);
//!     println!("Bytes1 after: {}", result1);
//!     println!("Bytes2 after: {}", result2);
//!     // `OCamlRuntime`'s `Drop` implementation will pefrorm the necessary cleanup
//!     // to shutdown the OCaml runtime.
//! }
//! ```
//!
//! ### Calling into Rust from OCaml
//!
//! To be able to call a Rust function from OCaml, it has to be defined in a way that exposes it to OCaml. This can be done with the [`ocaml_export!`] macro.
//!
//! #### Example
//!
//! ```rust,no_run
//! use ocaml_interop::{
//!     ocaml_export, FromOCaml, OCamlInt, OCaml, OCamlBytes,
//!     OCamlRef, ToOCaml,
//! };
//!
//! // `ocaml_export` expands the function definitions by adding `pub` visibility and
//! // the required `#[no_mangle]` and `extern` declarations. It also takes care of
//! // acquiring the OCaml runtime handle and binding it to the name provided as
//! // the first parameter of the function.
//! ocaml_export! {
//!     // The first parameter is a name to which the GC frame handle will be bound to.
//!     // The remaining parameters must have type `OCamlRef<T>`, and the return
//!     // value `OCaml<T>`.
//!     fn rust_twice(cr, num: OCamlRef<OCamlInt>) -> OCaml<OCamlInt> {
//!         let num: i64 = num.to_rust(cr);
//!         unsafe { OCaml::of_i64_unchecked(num * 2) }
//!     }
//!
//!     fn rust_increment_bytes(
//!         cr,
//!         bytes: OCamlRef<OCamlBytes>,
//!         first_n: OCamlRef<OCamlInt>,
//!     ) -> OCaml<OCamlBytes> {
//!         let first_n: i64 = first_n.to_rust(cr);
//!         let first_n = first_n as usize;
//!         let mut vec: Vec<u8> = bytes.to_rust(cr);
//!
//!         for i in 0..first_n {
//!             vec[i] += 1;
//!         }
//!
//!         vec.to_ocaml(cr)
//!     }
//! }
//! ```
//!
//! Then in OCaml, these functions can be referred to in the same way as C functions:
//!
//! ```ocaml
//! external rust_twice: int -> int = "rust_twice"
//! external rust_increment_bytes: bytes -> int -> bytes = "rust_increment_bytes"
//! ```
//!
//! ## References and links
//!
//! - OCaml Manual: [Chapter 20  Interfacing C with OCaml](https://caml.inria.fr/pub/docs/manual-ocaml/intfc.html).
//! - [Safely Mixing OCaml and Rust](https://docs.google.com/viewer?a=v&pid=sites&srcid=ZGVmYXVsdGRvbWFpbnxtbHdvcmtzaG9wcGV8Z3g6NDNmNDlmNTcxMDk1YTRmNg) paper by Stephen Dolan.
//! - [Safely Mixing OCaml and Rust](https://www.youtube.com/watch?v=UXfcENNM_ts) talk by Stephen Dolan.
//! - [CAMLroot: revisiting the OCaml FFI](https://arxiv.org/abs/1812.04905).
//! - [caml-oxide](https://github.com/stedolan/caml-oxide), the code from that paper.
//! - [ocaml-rs](https://github.com/zshipko/ocaml-rs), another OCaml<->Rust FFI library.

mod boxroot;
mod closure;
mod conv;
mod error;
mod macros;
mod memory;
mod mlvalues;
mod runtime;
mod value;

pub use crate::boxroot::BoxRoot;

pub use crate::closure::{OCamlFn1, OCamlFn2, OCamlFn3, OCamlFn4, OCamlFn5};
pub use crate::conv::{FromOCaml, ToOCaml};
pub use crate::error::OCamlException;
pub use crate::memory::OCamlRef;
pub use crate::mlvalues::{
    OCamlBytes, OCamlFloat, OCamlInt, OCamlInt32, OCamlInt64, OCamlList, RawOCaml,
};
pub use crate::runtime::OCamlRuntime;
pub use crate::value::OCaml;

#[doc(hidden)]
pub mod internal {
    pub use crate::closure::OCamlClosure;
    pub use crate::memory::{caml_alloc, store_field};
    pub use crate::mlvalues::tag;
    pub use crate::mlvalues::UNIT;
    pub use ocaml_boxroot_sys::{boxroot_setup, boxroot_teardown};
    pub use ocaml_sys::caml_hash_variant;

    // To bypass ocaml_sys::int_val unsafe declaration
    pub fn int_val(val: super::RawOCaml) -> isize {
        unsafe { ocaml_sys::int_val(val) }
    }
}

#[doc(hidden)]
#[cfg(doctest)]
pub mod compile_fail_tests;

#[doc(hidden)]
#[cfg(test)]
mod compile_ok_tests;
