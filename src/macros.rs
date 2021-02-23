// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

#[cfg(doc)]
use crate::*;

/// Declares OCaml functions.
///
/// `ocaml! { pub fn ocaml_name(arg1: Typ1, ...) -> Ret_typ; ... }` declares a function that has been
/// defined in OCaml code and registered with `Callback.register "ocaml_name" the_function`.
///
/// Visibility and return value type can be omitted. The return type defaults to unit when omitted.
///
/// When invoking one of these functions, the first argument must be a `&mut OCamlRuntime`,
/// and the remaining arguments `OCamlRef<ArgT>`.
///
/// The return value is a `BoxRoot<RetType>`.
///
/// Calls that raise an OCaml exception will `panic!`.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # struct MyRecord {};
/// ocaml! {
///     // Declares `print_endline`, with a single `String` (`OCaml<String>` when invoked)
///     // argument and unit return type (default when omitted).
///     pub fn print_endline(s: String);
///
///     // Declares `bytes_concat`, with two arguments, an OCaml `bytes` separator,
///     // and an OCaml list of segments to concatenate. Return value is an OCaml `bytes`
///     // value.
///     fn bytes_concat(sep: OCamlBytes, segments: OCamlList<OCamlBytes>) -> OCamlBytes;
/// }
/// ```
#[macro_export]
macro_rules! ocaml {
    () => ();

    ($vis:vis fn $name:ident(
        $arg:ident: $typ:ty $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis fn $name<'a>(
            cr: &'a mut $crate::OCamlRuntime,
            $arg: $crate::OCamlRef<$typ>,
        ) -> $crate::BoxRoot<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(closure, $name);
            $crate::BoxRoot::new(closure.call(cr, $arg))
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $arg1:ident: $typ1:ty,
        $arg2:ident: $typ2:ty $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis fn $name<'a>(
            cr: &'a mut $crate::OCamlRuntime,
            $arg1: $crate::OCamlRef<$typ1>,
            $arg2: $crate::OCamlRef<$typ2>,
        ) -> $crate::BoxRoot<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(closure, $name);
            $crate::BoxRoot::new(closure.call2(cr, $arg1, $arg2))
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $arg1:ident: $typ1:ty,
        $arg2:ident: $typ2:ty,
        $arg3:ident: $typ3:ty $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis fn $name<'a>(
            cr: &'a mut $crate::OCamlRuntime,
            $arg1: $crate::OCamlRef<$typ1>,
            $arg2: $crate::OCamlRef<$typ2>,
            $arg3: $crate::OCamlRef<$typ3>,
        ) -> $crate::BoxRoot<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(closure, $name);
            $crate::BoxRoot::new(closure.call3(cr, $arg1, $arg2, $arg3))
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $($arg:ident: $typ:ty),+ $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis fn $name<'a>(
            cr: &'a mut $crate::OCamlRuntime,
            $($arg: $crate::OCamlRef<$typ>),+
    ) -> $crate::BoxRoot<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(closure, $name);
            $crate::BoxRoot::new(closure.call_n(cr, &mut [$(unsafe { $arg.get_raw() }),+]))
        }

        $crate::ocaml!($($t)*);
    }
}

/// Defines Rust functions callable from OCaml.
///
/// The first argument in these functions declarations is a name to bind a `&mut OCamlRuntme`.
///
/// Arguments and return values must be of type [`OCaml`]`<T>`, or `f64` in the case of unboxed floats.
///
/// The return type defaults to unit when omitted.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// ocaml_export! {
///     fn rust_twice(cr, num: OCamlRef<OCamlInt>) -> OCaml<OCamlInt> {
///         let num: i64 = num.to_rust(cr);
///         unsafe { OCaml::of_i64_unchecked(num * 2) }
///     }
///
///     fn rust_twice_boxed_i32(cr, num: OCamlRef<OCamlInt32>) -> OCaml<OCamlInt32> {
///         let num: i32 = num.to_rust(cr);
///         let result = num * 2;
///         result.to_ocaml(cr)
///     }
///
///     fn rust_add_unboxed_floats_noalloc(_cr, num: f64, num2: f64) -> f64 {
///         num * num2
///     }
///
///     fn rust_twice_boxed_float(cr, num: OCamlRef<OCamlFloat>) -> OCaml<OCamlFloat> {
///         let num: f64 = num.to_rust(cr);
///         let result = num * 2.0;
///         result.to_ocaml(cr)
///     }
///
///     fn rust_increment_ints_list(cr, ints: OCamlRef<OCamlList<OCamlInt>>) -> OCaml<OCamlList<OCamlInt>> {
///         let mut vec: Vec<i64> = ints.to_rust(cr);
///
///         for i in 0..vec.len() {
///             vec[i] += 1;
///         }
///
///         vec.to_ocaml(cr)
///     }
///
///     fn rust_make_tuple(cr, fst: OCamlRef<String>, snd: OCamlRef<OCamlInt>) -> OCaml<(String, OCamlInt)> {
///         let fst: String = fst.to_rust(cr);
///         let snd: i64 = snd.to_rust(cr);
///         let tuple = (fst, snd);
///         tuple.to_ocaml(cr)
///     }
/// }
/// ```
#[macro_export]
macro_rules! ocaml_export {
    {} => ();

    // Unboxed float return
    {
        fn $name:ident( $cr:ident, $($args:tt)*) -> f64
           $body:block

        $($t:tt)*
    } => {
        $crate::expand_exported_function!(
            @name $name
            @cr $cr
            @final_args { }
            @proc_args { $($args)*, }
            @return { f64 }
            @body $body
            @original_args $($args)*
        );

        $crate::ocaml_export!{$($t)*}
    };

    // Other (or empty) return value type
    {
        fn $name:ident( $cr:ident, $($args:tt)*) $(-> $rtyp:ty)?
           $body:block

        $($t:tt)*
    } => {
        $crate::expand_exported_function!(
            @name $name
            @cr $cr
            @final_args { }
            @proc_args { $($args)*, }
            @return { $($rtyp)? }
            @body $body
            @original_args $($args)*
        );

        $crate::ocaml_export!{$($t)*}
    };

    // Invalid arguments

    {
        fn $name:ident( $($invalid_args:tt)* ) $(-> $rtyp:ty)?
           $body:block

        $($t:tt)*
    } => {
        compile_error!("Rust->OCaml exported functions must include an identifier for the OCaml runtime handle followed by at least one argument");
    }
}

/// Implements conversion between a Rust struct and an OCaml record.
///
/// See the [`impl_to_ocaml_record!`] and [`impl_from_ocaml_record!`] macros
/// for more details.
#[macro_export]
macro_rules! impl_conv_ocaml_record {
    ($rust_typ:ident => $ocaml_typ:ident {
        $($field:ident : $ocaml_field_typ:ty $(=> $conv_expr:expr)?),+ $(,)?
    }) => {
        $crate::impl_to_ocaml_record! {
            $rust_typ => $ocaml_typ {
                $($field : $ocaml_field_typ $(=> $conv_expr)?),+
            }
        }

        $crate::impl_from_ocaml_record! {
            $ocaml_typ => $rust_typ {
                $($field : $ocaml_field_typ),+
            }
        }
    };

    ($both_typ:ident {
        $($t:tt)*
    }) => {
        $crate::impl_conv_ocaml_record! {
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Implements conversion between a Rust enum and an OCaml variant.
///
/// See the [`impl_to_ocaml_variant!`] and [`impl_from_ocaml_variant!`] macros
/// for more details.
#[macro_export]
macro_rules! impl_conv_ocaml_variant {
    ($rust_typ:ty => $ocaml_typ:ty {
        $($($tag:ident)::+ $(($($slot_name:ident: $slot_typ:ty),+ $(,)?))? $(=> $conv:expr)?),+ $(,)?
    }) => {
        $crate::impl_to_ocaml_variant! {
            $rust_typ => $ocaml_typ {
                $($($tag)::+ $(($($slot_name: $slot_typ),+))? $(=> $conv)?),+
            }
        }

        $crate::impl_from_ocaml_variant! {
            $ocaml_typ => $rust_typ {
                $($($tag)::+ $(($($slot_name: $slot_typ),+))?),+
            }
        }
    };

    ($both_typ:ty {
        $($t:tt)*
    }) => {
        $crate::impl_conv_ocaml_variant!{
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Unpacks an OCaml record into a Rust record
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_mystruct(unit: ()) -> MyStruct; }
/// struct MyStruct {
///     int_field: i64,
///     string_field: String,
/// }
///
/// // Assuming an OCaml record declaration like:
/// //
/// //      type my_struct = {
/// //          int_field: int;
/// //          string_field: string;
/// //      }
/// //
/// // NOTE: What is important is the order of the fields, not their names.
///
/// # fn unpack_record_example(cr: &mut OCamlRuntime) {
/// let ocaml_struct_root = make_mystruct(cr, &OCaml::unit());
/// let ocaml_struct = cr.get(&ocaml_struct_root);
/// let my_struct = ocaml_unpack_record! {
///     //  value    => RustConstructor { field: OCamlType, ... }
///     ocaml_struct => MyStruct {
///         int_field: OCamlInt,
///         string_field: String,
///     }
/// };
/// // ...
/// # ()
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_unpack_record {
    ($var:ident => $cons:ident {
        $($field:ident : $ocaml_typ:ty),+ $(,)?
    }) => {{
        let record = $var;
        unsafe {
            let mut current = 0;

            $(
                let $field = record.field::<$ocaml_typ>(current).to_rust();
                current += 1;
            )+

            $cons {
                $($field),+
            }
        }
    }};

    ($var:ident => $cons:ident (
        $($field:ident : $ocaml_typ:ty),+ $(,)?
    )) => {{
        let record = $var;
        unsafe {
            let mut current = 0;

            $(
                let $field = record.field::<$ocaml_typ>(current).to_rust();
                current += 1;
            )+

            $cons (
                $($field),+
            )
        }
    }};
}

/// Allocates an OCaml memory block tagged with the specified value.
///
/// It is used internally to allocate OCaml variants, its direct use is
/// not recommended.
#[macro_export]
macro_rules! ocaml_alloc_tagged_block {
    ($cr:ident, $tag:expr, $($field:ident : $ocaml_typ:ty),+ $(,)?) => {
        unsafe {
            let mut current = 0;
            let field_count = $crate::count_fields!($($field)*);
            let block: $crate::BoxRoot<()> = $crate::BoxRoot::new($crate::OCaml::new($cr, $crate::internal::caml_alloc(field_count, $tag)));
            $(
                let $field: $crate::OCaml<$ocaml_typ> = $field.to_ocaml($cr);
                $crate::internal::store_field(block.get_raw(), current, $field.raw());
                current += 1;
            )+
            $crate::OCaml::new($cr, block.get_raw())
        }
    };
}

/// Allocates an OCaml record built from a Rust record
///
/// Most of the time the [`impl_to_ocaml_record!`] macro will be used to define how records
/// should be converted. This macro is useful when implementing OCaml allocation
/// functions directly.
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// struct MyStruct {
///     int_field: u8,
///     string_field: String,
/// }
///
/// // Assuming an OCaml record declaration like:
/// //
/// //      type my_struct = {
/// //          int_field: int;
/// //          string_field: string;
/// //      }
/// //
/// // NOTE: What is important is the order of the fields, not their names.
///
/// # fn alloc_record_example(cr: &mut OCamlRuntime) {
/// let ms = MyStruct { int_field: 132, string_field: "blah".to_owned() };
/// let ocaml_ms: OCaml<MyStruct> = ocaml_alloc_record! {
///     //  value { field: OCamlType, ... }
///     cr, ms {  // cr: &mut OCamlRuntime
///         // optionally `=> expr` can be used to pre-process the field value
///         // before the conversion into OCaml takes place.
///         // Inside the expression, a variable with the same name as the field
///         // is bound to a reference to the field value.
///         int_field: OCamlInt => { *int_field as i64 },
///         string_field: String,
///     }
/// };
/// // ...
/// # ()
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_alloc_record {
    ($cr:ident, $self:ident {
        $($field:ident : $ocaml_typ:ty $(=> $conv_expr:expr)?),+ $(,)?
    }) => {
        unsafe {
            let mut current = 0;
            let field_count = $crate::count_fields!($($field)*);
            let record: $crate::BoxRoot<()> = $crate::BoxRoot::new($crate::OCaml::new($cr, $crate::internal::caml_alloc(field_count, 0)));
            $(
                let $field = &$crate::prepare_field_for_mapping!($self.$field $(=> $conv_expr)?);
                let $field: $crate::OCaml<$ocaml_typ> = $field.to_ocaml($cr);
                $crate::internal::store_field(record.get_raw(), current, $field.raw());
                current += 1;
            )+
            $crate::OCaml::new($cr, record.get_raw())
        }
    };
}

/// Implements [`FromOCaml`] for mapping an OCaml record into a Rust record.
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_mystruct(unit: ()) -> MyStruct; }
/// struct MyStruct {
///     int_field: i64,
///     string_field: String,
/// }
///
/// // Assuming an OCaml record declaration like:
/// //
/// //      type my_struct = {
/// //          int_field: int;
/// //          string_field: string;
/// //      }
/// //
/// // NOTE: What is important is the order of the fields, not their names.
///
/// impl_from_ocaml_record! {
///     // Optionally, if Rust and OCaml types don't match:
///     // OCamlType => RustType { ... }
///     MyStruct {
///         int_field: OCamlInt,
///         string_field: String,
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_from_ocaml_record {
    ($ocaml_typ:ident => $rust_typ:ident {
        $($field:ident : $ocaml_field_typ:ty),+ $(,)?
    }) => {
        unsafe impl $crate::FromOCaml<$ocaml_typ> for $rust_typ {
            fn from_ocaml(v: $crate::OCaml<$ocaml_typ>) -> Self {
                $crate::ocaml_unpack_record! { v =>
                    $rust_typ {
                        $($field : $ocaml_field_typ),+
                    }
                }
            }
        }
    };

    ($both_typ:ident {
        $($t:tt)*
    }) => {
        $crate::impl_from_ocaml_record! {
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };

    ($ocaml_typ:ident => $rust_typ:ident (
        $($field:ident : $ocaml_field_typ:ty),+ $(,)?
    )) => {
        unsafe impl $crate::FromOCaml<$ocaml_typ> for $rust_typ {
            fn from_ocaml(v: $crate::OCaml<$ocaml_typ>) -> Self {
                $crate::ocaml_unpack_record! { v =>
                    $rust_typ (
                        $($field : $ocaml_field_typ),+
                    )
                }
            }
        }
    };

    ($both_typ:ident (
        $($t:tt)*
    )) => {
        $crate::impl_from_ocaml_record! {
            $both_typ => $both_typ (
                $($t)*
            )
        }
    };
}

/// Implements [`ToOCaml`] for mapping a Rust record into an OCaml record.
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// struct MyStruct {
///     int_field: u8,
///     string_field: String,
/// }
///
/// // Assuming an OCaml record declaration like:
/// //
/// //      type my_struct = {
/// //          int_field: int;
/// //          string_field: string;
/// //      }
/// //
/// // NOTE: What is important is the order of the fields, not their names.
///
/// impl_to_ocaml_record! {
///     // Optionally, if Rust and OCaml types don't match:
///     // RustType => OCamlType { ... }
///     MyStruct {
///         // optionally `=> expr` can be used to preprocess the field value
///         // before the conversion into OCaml takes place.
///         // Inside the expression, a variable with the same name as the field
///         // is bound to a reference to the field value.
///         int_field: OCamlInt => { *int_field as i64 },
///         string_field: String,
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_to_ocaml_record {
    ($rust_typ:ty => $ocaml_typ:ident {
        $($field:ident : $ocaml_field_typ:ty $(=> $conv_expr:expr)?),+ $(,)?
    }) => {
        unsafe impl $crate::ToOCaml<$ocaml_typ> for $rust_typ {
            fn to_ocaml<'a>(&self, cr: &'a mut $crate::OCamlRuntime) -> $crate::OCaml<'a, $ocaml_typ> {
                $crate::ocaml_alloc_record! {
                    cr, self {
                        $($field : $ocaml_field_typ $(=> $conv_expr)?),+
                    }
                }
            }
        }
    };

    ($both_typ:ident {
        $($t:tt)*
    }) => {
        $crate::impl_to_ocaml_record! {
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Implements [`FromOCaml`] for mapping an OCaml variant into a Rust enum.
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement =
/// //        | StepLeft
/// //        | StepRight
/// //        | Rotate of float
/// //
/// // NOTE: What is important is the order of the tags, not their names.
///
/// impl_from_ocaml_variant! {
///     // Optionally, if Rust and OCaml types don't match:
///     // OCamlType => RustType { ... }
///     Movement {
///         // Alternative: StepLeft  => Movement::StepLeft
///         //              <anyname> => <build-expr>
///         Movement::StepLeft,
///         Movement::StepRight,
///         // Tag field names are mandatory
///         Movement::Rotate(rotation: OCamlFloat),
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_from_ocaml_variant {
    ($ocaml_typ:ty => $rust_typ:ty {
        $($t:tt)*
    }) => {
        unsafe impl $crate::FromOCaml<$ocaml_typ> for $rust_typ {
            fn from_ocaml(v: $crate::OCaml<$ocaml_typ>) -> Self {
                let result = $crate::ocaml_unpack_variant! {
                    v => {
                        $($t)*
                    }
                };

                let msg = concat!(
                    "Failure when unpacking an OCaml<", stringify!($ocaml_typ), "> variant into ",
                    stringify!($rust_typ), " (unexpected tag value)");

                result.expect(msg)
            }
        }
    };

    ($both_typ:ty {
        $($t:tt)*
    }) => {
        $crate::impl_from_ocaml_variant!{
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Unpacks an OCaml variant and maps it into a Rust enum.
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Note
///
/// Unlike with [`ocaml_unpack_record!`], the result of [`ocaml_unpack_variant!`] is a `Result` value.
/// An error will be returned in the case of an unexpected tag value. This may change in the future.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_ocaml_movement(unit: ()) -> Movement; }
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement =
/// //        | StepLeft
/// //        | StepRight
/// //        | Rotate of float
/// //
/// // NOTE: What is important is the order of the tags, not their names.
///
/// # fn unpack_variant_example(cr: &mut OCamlRuntime) {
/// let ocaml_variant_root = make_ocaml_movement(cr, &OCaml::unit());
/// let ocaml_variant = cr.get(&ocaml_variant_root);
/// let result = ocaml_unpack_variant! {
///     ocaml_variant => {
///         // Alternative: StepLeft  => Movement::StepLeft
///         //              <anyname> => <build-expr>
///         Movement::StepLeft,
///         Movement::StepRight,
///         // Tag field names are mandatory
///         Movement::Rotate(rotation: OCamlFloat),
///     }
/// }.unwrap();
/// // ...
/// # }
#[macro_export]
macro_rules! ocaml_unpack_variant {
    ($self:ident => {
        $($($tag:ident)::+ $(($($slot_name:ident: $slot_typ:ty),+ $(,)?))? $(=> $conv:expr)?),+ $(,)?
    }) => {
        (|| {
            let mut current_block_tag = 0;
            let mut current_long_tag = 0;

            $(
                $crate::unpack_variant_tag!(
                    $self, current_block_tag, current_long_tag,
                    $($tag)::+ $(($($slot_name: $slot_typ),+))? $(=> $conv)?);
            )+

            Err("Invalid tag value found when converting from an OCaml variant")
        })()
    };

    ($self:ident => {
        $($($tag:ident)::+ $({$($slot_name:ident: $slot_typ:ty),+ $(,)?})? $(=> $conv:expr)?),+ $(,)?
    }) => {
        (|| {
            let mut current_block_tag = 0;
            let mut current_long_tag = 0;

            $(
                $crate::unpack_variant_tag!(
                    $self, current_block_tag, current_long_tag,
                    $($tag)::+ $({$($slot_name: $slot_typ),+})? $(=> $conv)?);
            )+

            Err("Invalid tag value found when converting from an OCaml variant")
        })()
    };
}

/// Allocates an OCaml variant, mapped from a Rust enum.
///
/// The match in this conversion is exhaustive, and requires that every enum case is covered.
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_ocaml_movement(unit: ()) -> Movement; }
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement =
/// //        | StepLeft
/// //        | StepRight
/// //        | Rotate of float
/// //
/// // NOTE: What is important is the order of the tags, not their names.
///
/// # fn alloc_variant_example(cr: &mut OCamlRuntime) {
/// let movement = Movement::Rotate(180.0);
/// let ocaml_movement: OCaml<Movement> = ocaml_alloc_variant! {
///     cr, movement => {
///         Movement::StepLeft,
///         Movement::StepRight,
///         // Tag field names are mandatory
///         Movement::Rotate(rotation: OCamlFloat),
///     }
/// };
/// // ...
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_alloc_variant {
    ($cr:ident, $self:ident => {
        $($($tag:ident)::+ $(($($slot_name:ident: $slot_typ:ty),+ $(,)?))? $(,)?),+
    }) => {
        $crate::ocaml_alloc_variant_match!{
            $cr, $self, 0u8, 0u8,

            @units {}
            @blocks {}

            @pending $({ $($tag)::+ $(($($slot_name: $slot_typ),+))? })+
        }
    };
}

/// Implements [`ToOCaml`] for mapping a Rust enum into an OCaml variant.
///
/// The match in this conversion is exhaustive, and requires that every enum case is covered.
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement =
/// //        | StepLeft
/// //        | StepRight
/// //        | Rotate of float
/// //
/// // NOTE: What is important is the order of the tags, not their names.
///
/// impl_to_ocaml_variant! {
///     // Optionally, if Rust and OCaml types don't match:
///     // RustType => OCamlType { ... }
///     Movement {
///         Movement::StepLeft,
///         Movement::StepRight,
///         // Tag field names are mandatory
///         Movement::Rotate(rotation: OCamlFloat),
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_to_ocaml_variant {
    ($rust_typ:ty => $ocaml_typ:ty {
        $($t:tt)*
    }) => {
        unsafe impl $crate::ToOCaml<$ocaml_typ> for $rust_typ {
            fn to_ocaml<'a>(&self, cr: &'a mut $crate::OCamlRuntime) -> $crate::OCaml<'a, $ocaml_typ> {
                $crate::ocaml_alloc_variant! {
                    cr, self => {
                        $($t)*
                    }
                }
            }
        }
    };

    ($both_typ:ty {
        $($t:tt)*
    }) => {
        $crate::impl_to_ocaml_variant!{
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Implements [`FromOCaml`] for mapping an OCaml variant into a Rust enum.
///
/// It is important that the order of the fields remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement = [
/// //        | `StepLeft
/// //        | `StepRight
/// //        | `Rotate of float
/// //      ]
///
/// impl_from_ocaml_polymorphic_variant! {
///     // Optionally, if Rust and OCaml types don't match:
///     // OCamlType => RustType { ... }
///     Movement {
///         StepLeft  => Movement::StepLeft,
///         StepRight => Movement::StepRight,
///         // Tag field names are mandatory
///         Rotate(rotation: OCamlFloat)
///                   => Movement::Rotate(rotation),
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_from_ocaml_polymorphic_variant {
    ($ocaml_typ:ty => $rust_typ:ty {
        $($t:tt)*
    }) => {
        unsafe impl $crate::FromOCaml<$ocaml_typ> for $rust_typ {
            fn from_ocaml(v: $crate::OCaml<$ocaml_typ>) -> Self {
                let result = $crate::ocaml_unpack_polymorphic_variant! {
                    v => {
                        $($t)*
                    }
                };

                let msg = concat!(
                    "Failure when unpacking an OCaml<", stringify!($ocaml_typ), "> polymorphic variant into ",
                    stringify!($rust_typ), " (unexpected tag value)");

                result.expect(msg)
            }
        }
    };

    ($both_typ:ty {
        $($t:tt)*
    }) => {
        $crate::impl_from_ocaml_polymorphic_variant!{
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Unpacks an OCaml polymorphic variant and maps it into a Rust enum.
///
/// # Note
///
/// Unlike with [`ocaml_unpack_record!`], the result of [`ocaml_unpack_polymorphic_variant!`] is a `Result` value.
/// An error will be returned in the case of an unexpected tag value. This may change in the future.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_ocaml_polymorphic_movement(unit: ()) -> Movement; }
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement = [
/// //        | `StepLeft
/// //        | `StepRight
/// //        | `Rotate of float
/// //      ]
///
/// # fn unpack_polymorphic_variant_example(cr: &mut OCamlRuntime) {
/// let ocaml_polymorphic_variant_root = make_ocaml_polymorphic_movement(cr, &OCaml::unit());
/// let ocaml_polymorphic_variant = cr.get(&ocaml_polymorphic_variant_root);
/// let result = ocaml_unpack_polymorphic_variant! {
///     ocaml_polymorphic_variant => {
///         StepLeft  => Movement::StepLeft,
///         StepRight => Movement::StepRight,
///         // Tag field names are mandatory
///         Rotate(rotation: OCamlFloat)
///                   => Movement::Rotate(rotation),
///     }
/// }.unwrap();
/// // ...
/// # }
#[macro_export]
macro_rules! ocaml_unpack_polymorphic_variant {
    ($self:ident => {
        $($tag:ident $(($($slot_name:ident: $slot_typ:ty),+ $(,)?))? => $conv:expr),+ $(,)?
    }) => {
        (|| {
            $(
                $crate::unpack_polymorphic_variant_tag!(
                    $self, $tag $(($($slot_name: $slot_typ),+))? => $conv);
            )+

            Err("Invalid tag value found when converting from an OCaml polymorphic variant")
        })()
    };
}

// Internal utility macros

#[doc(hidden)]
#[macro_export]
macro_rules! repeat_slice {
    (@expr $value:expr;
     @accum [$($accum:expr),+];
     @rest) => {
         [$($accum),+]
     };

    (@expr $value:expr;
     @accum [$($accum:expr),+];
     @rest $_v1:ident $_v2:ident $_v3:ident $_v4:ident $_v5:ident $($vars:ident)*) => {
        $crate::repeat_slice!(
            @expr $value;
            @accum [$value, $value, $value, $value, $value, $($accum),+];
            @rest $($vars)*)
    };

    (@expr $value:expr;
        @accum [$($accum:expr),+];
        @rest $_v1:ident $($vars:ident)*) => {

        $crate::repeat_slice!(
            @expr $value;
            @accum [$value, $($accum),+];
            @rest $($vars)*)
    };

    ($value:expr, $field:ident $($vars:ident)*) => {
        $crate::repeat_slice!(
            @expr $value;
            @accum [$value];
            @rest $($vars)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! count_fields {
    () => {0usize};
    ($_f1:ident $_f2:ident $_f3:ident $_f4:ident $_f5:ident $($fields:ident)*) => {
        5usize + $crate::count_fields!($($fields)*)
    };
    ($field:ident $($fields:ident)*) => {1usize + $crate::count_fields!($($fields)*)};
}

#[doc(hidden)]
#[macro_export]
macro_rules! prepare_field_for_mapping {
    ($self:ident.$field:ident) => {
        $self.$field
    };

    ($self:ident.$field:ident => $conv_expr:expr) => {{
        let $field = &$self.$field;
        $conv_expr
    }};
}

// TODO: check generated machine code and see if it is worth it to generate a switch
#[doc(hidden)]
#[macro_export]
macro_rules! unpack_variant_tag {
    ($self:ident, $current_block_tag:ident, $current_long_tag:ident, $($tag:ident)::+) => {
        $crate::unpack_variant_tag!($self, $current_block_tag, $current_long_tag, $($tag)::+ => $($tag)::+)
    };

    ($self:ident, $current_block_tag:ident, $current_long_tag:ident, $($tag:ident)::+ => $conv:expr) => {
        if $self.is_long() && $crate::internal::int_val(unsafe { $self.raw() }) == $current_long_tag {
            return Ok($conv);
        }
        $current_long_tag += 1;
    };

    // Parens: tuple
    ($self:ident, $current_block_tag:ident, $current_long_tag:ident,
        $($tag:ident)::+ ($($slot_name:ident: $slot_typ:ty),+)) => {

        $crate::unpack_variant_tag!(
            $self, $current_block_tag, $current_long_tag,
            $($tag)::+ ($($slot_name: $slot_typ),+) => $($tag)::+($($slot_name),+))
    };

    // Braces: record
    ($self:ident, $current_block_tag:ident, $current_long_tag:ident,
        $($tag:ident)::+ {$($slot_name:ident: $slot_typ:ty),+}) => {

        $crate::unpack_variant_tag!(
            $self, $current_block_tag, $current_long_tag,
            $($tag)::+ {$($slot_name: $slot_typ),+} => $($tag)::+{$($slot_name),+})
    };

    // Parens: tuple
    ($self:ident, $current_block_tag:ident, $current_long_tag:ident,
        $($tag:ident)::+ ($($slot_name:ident: $slot_typ:ty),+) => $conv:expr) => {

        if $self.is_block() && $self.tag_value() == $current_block_tag {
            let mut current_field = 0;

            $(
                let $slot_name = unsafe { $self.field::<$slot_typ>(current_field).to_rust() };
                current_field += 1;
            )+

            return Ok($conv);
        }
        $current_block_tag += 1;
    };

    // Braces: record
    ($self:ident, $current_block_tag:ident, $current_long_tag:ident,
        $($tag:ident)::+ {$($slot_name:ident: $slot_typ:ty),+} => $conv:expr) => {

        if $self.is_block() && $self.tag_value() == $current_block_tag {
            let mut current_field = 0;

            $(
                let $slot_name = unsafe { $self.field::<$slot_typ>(current_field).to_rust() };
                current_field += 1;
            )+

            return Ok($conv);
        }
        $current_block_tag += 1;
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! ocaml_alloc_variant_match {
    // Base case, generate `match` expression
    ($cr:ident, $self:ident, $current_block_tag:expr, $current_long_tag:expr,

        @units {
            $({ $($unit_tag:ident)::+ @ $unit_tag_counter:expr })*
        }
        @blocks {
            $({ $($block_tag:ident)::+ ($($block_slot_name:ident: $block_slot_typ:ty),+) @ $block_tag_counter:expr })*
        }

        @pending
    ) => {
        match $self {
            $(
                $($unit_tag)::+ =>
                    unsafe { $crate::OCaml::new($cr, $crate::OCaml::of_i64_unchecked($unit_tag_counter as i64).raw()) },
            )*
            $(
                $($block_tag)::+($($block_slot_name),+) =>
                    $crate::ocaml_alloc_tagged_block!($cr, $block_tag_counter, $($block_slot_name: $block_slot_typ),+),
            )*
        }
    };

    // Found unit tag, add to accumulator and increment unit variant tag number
    ($cr:ident, $self:ident, $current_block_tag:expr, $current_long_tag:expr,

        @units { $($unit_tags_accum:tt)* }
        @blocks { $($block_tags_accum:tt)* }

        @pending
            { $($found_tag:ident)::+ }
            $($tail:tt)*
    ) => {
        $crate::ocaml_alloc_variant_match!{
            $cr, $self, $current_block_tag, {1u8 + $current_long_tag},

            @units {
                $($unit_tags_accum)*
                { $($found_tag)::+ @ $current_long_tag }
            }
            @blocks { $($block_tags_accum)* }

            @pending $($tail)*
        }
    };

    // Found block tag, add to accumulator and increment block variant tag number
    ($cr:ident, $self:ident, $current_block_tag:expr, $current_long_tag:expr,

        @units { $($unit_tags_accum:tt)* }
        @blocks { $($block_tags_accum:tt)* }

        @pending
            { $($found_tag:ident)::+ ($($found_slot_name:ident: $found_slot_typ:ty),+) }
            $($tail:tt)*
    ) => {
        $crate::ocaml_alloc_variant_match!{
            $cr, $self, {1u8 + $current_block_tag}, $current_long_tag,

            @units { $($unit_tags_accum)* }
            @blocks {
                $($block_tags_accum)*
                { $($found_tag)::+ ($($found_slot_name: $found_slot_typ),+) @ $current_block_tag }
            }

            @pending $($tail)*
        }
    };
}

// TODO: check generated machine code and see if it is worth it to generate a switch
#[doc(hidden)]
#[macro_export]
macro_rules! unpack_polymorphic_variant_tag {
    ($self:ident, $tag:ident => $conv:expr) => {
        #[allow(non_snake_case)]
        let $tag = $crate::polymorphic_variant_tag_hash!($tag);
        if $self.is_long() && unsafe { $self.raw() } == $tag {
            return Ok($conv);
        }
    };

    ($self:ident, $tag:ident($slot_name:ident: $slot_typ:ty) => $conv:expr) => {
        #[allow(non_snake_case)]
        let $tag = $crate::polymorphic_variant_tag_hash!($tag);

        if $self.is_block_sized(2) &&
            $self.tag_value() == $crate::internal::tag::TAG_POLYMORPHIC_VARIANT &&
            unsafe { $self.field::<$crate::OCamlInt>(0).raw() } == $tag {

            let $slot_name = unsafe { $self.field::<$slot_typ>(1).to_rust() };

            return Ok($conv);
        }
    };

    ($self:ident, $tag:ident($($slot_name:ident: $slot_typ:ty),+) => $conv:expr) => {
        #[allow(non_snake_case)]
        let $tag = $crate::polymorphic_variant_tag_hash!($tag);

        if $self.is_block_sized(2) &&
            $self.tag_value() == $crate::internal::tag::TAG_POLYMORPHIC_VARIANT &&
            unsafe { $self.field::<$crate::OCamlInt>(0).raw() } == $tag {

            let ($($slot_name),+) = unsafe { $self.field::<($($slot_typ),+)>(1).to_rust() };

            return Ok($conv);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! ocaml_closure_reference {
    ($var:ident, $name:ident) => {
        static NAME: &str = stringify!($name);
        static mut OC: Option<$crate::internal::OCamlClosure> = None;
        static INIT: ::std::sync::Once = ::std::sync::Once::new();
        let $var = unsafe {
            INIT.call_once(|| {
                OC = $crate::internal::OCamlClosure::named(NAME);
            });
            OC.unwrap_or_else(|| panic!("OCaml closure with name '{}' not registered", NAME))
        };
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! gcmark_result {
    ($cr:ident, $obj:expr) => {
        match $obj {
            Ok(t) => Ok(t.mark($cr).eval($cr)),
            Err(e) => Err(e),
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! default_to_ocaml_unit {
    // No return value, default to unit
    () => ($crate::OCaml<()>);

    // Return value specified
    ($rtyp:ty) => ($rtyp);
}

#[doc(hidden)]
#[macro_export]
macro_rules! default_to_unit {
    // No return value, default to unit
    () => {
        ()
    };

    // Return value specified
    ($rtyp:ty) => {
        $rtyp
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_rooted_args_init {
    // No more args
    () => ();

    // Nothing is done for unboxed floats
    ($arg:ident : f64) => ();

    ($arg:ident : f64, $($args:tt)*) =>
        ($crate::expand_rooted_args_init!($($args)*));

    // Other values are wrapped in `OCamlRef<T>` as given the same lifetime as the OCaml runtime handle borrow.
    ($arg:ident : $typ:ty) => {
        let $arg : $typ = unsafe { &$crate::BoxRoot::from_raw($arg) };
    };

    ($arg:ident : $typ:ty, $($args:tt)*) => {
        let $arg : $typ = unsafe { &$crate::BoxRoot::from_raw($arg) };
        $crate::expand_rooted_args_init!($($args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_exported_function {
    // Final expansions, with all argument types converted

    {
        @name $name:ident
        @cr $cr:ident
        @final_args { $($arg:ident : $typ:ty,)+ }
        @proc_args { $(,)? }
        @return { $($rtyp:tt)* }
        @body $body:block
        @original_args $($original_args:tt)*
    } => {
        #[no_mangle]
        pub extern "C" fn $name( $($arg: $typ),* ) -> $crate::expand_exported_function_return!($($rtyp)*) {
            let $cr = unsafe { &mut $crate::OCamlRuntime::recover_handle() };
            $crate::expand_rooted_args_init!($($original_args)*);
            $crate::expand_exported_function_body!(
                @body $body
                @return $($rtyp)*
            )
        }
    };

    // Args processing

    // Next arg is an unboxed float, leave as-is

    {
        @name $name:ident
        @cr $cr:ident
        @final_args { $($final_args:tt)* }
        @proc_args { $next_arg:ident : f64, $($proc_args:tt)* }
        @return { $($rtyp:tt)* }
        @body $body:block
        @original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!{
            @name $name
            @cr $cr
            @final_args { $($final_args)* $next_arg : f64, }
            @proc_args { $($proc_args)* }
            @return { $($rtyp)* }
            @body $body
            @original_args $($original_args)*
        }
    };

    // Next arg is not an unboxed float, replace with RawOCaml in output, add a root

    {
        @name $name:ident
        @cr $cr:ident
        @final_args { $($final_args:tt)* }
        @proc_args { $next_arg:ident : $typ:ty, $($proc_args:tt)* }
        @return { $($rtyp:tt)* }
        @body $body:block
        @original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!{
            @name $name
            @cr $cr
            @final_args { $($final_args)* $next_arg : $crate::RawOCaml, }
            @proc_args { $($proc_args)* }
            @return { $($rtyp)* }
            @body $body
            @original_args $($original_args)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_exported_function_body {
    { @body $body:block @return f64 } => {
        #[allow(unused_braces)]
        $body
    };

    { @body $body:block @return $rtyp:ty } => {{
        let retval : $rtyp = $body;
        unsafe { retval.raw() }
    }};

    { @body $body:block @return } => {
        $crate::expand_exported_function_body!(
            @body $body
            @return $crate::OCaml<()>
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_exported_function_return {
    () => {
        $crate::RawOCaml
    };

    (f64) => {
        f64
    };

    ($rtyp:ty) => {
        $crate::RawOCaml
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! polymorphic_variant_tag_hash {
    ($tag:ident) => {{
        static mut TAG_HASH: $crate::RawOCaml = 0;
        static INIT_TAG_HASH: std::sync::Once = std::sync::Once::new();
        unsafe {
            INIT_TAG_HASH.call_once(|| {
                TAG_HASH =
                    $crate::internal::caml_hash_variant(concat!(stringify!($tag), "\0").as_ptr())
            });
            TAG_HASH
        }
    }};
}
