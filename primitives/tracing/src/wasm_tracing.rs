// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

#![no_std]

use sp_std::{
    vec
};

#[cfg(not(feature = "with-tracing"))]
mod inner {
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __tracing_mk_span {
        (target: $target:expr, parent: $parent:expr, $lvl:expr, $name:expr, $($fields:tt)*) => { {} };
        (target: $target:expr, $lvl:expr, $name:expr, $($fields:tt)*) => { {} };
    }


    /// Constructs a new `Event` – noop
    #[macro_export]
    macro_rules! event {
        (target: $target:expr, parent: $parent:expr, $lvl:expr, { $($fields:tt)* } )=> { {} };

        (target: $target:expr, parent: $parent:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => { {} };
        (target: $target:expr, parent: $parent:expr, $lvl:expr, $($k:ident).+ = $($fields:tt)* ) =>  { {} };
        (target: $target:expr, parent: $parent:expr, $lvl:expr, $($arg:tt)+) => { {} };
        (target: $target:expr, $lvl:expr, { $($fields:tt)* } )=> {{}};
        (target: $target:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) =>  { {} };
        (target: $target:expr, $lvl:expr, $($k:ident).+ = $($fields:tt)* ) => { {} };
        (target: $target:expr, $lvl:expr, $($arg:tt)+ ) => { {} };
        (parent: $parent:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => { {} };
        (parent: $parent:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => { {} };
        (parent: $parent:expr, $lvl:expr, $($k:ident).+ = $($field:tt)*) => { {} };
        (parent: $parent:expr, $lvl:expr, ?$($k:ident).+ = $($field:tt)*) => { {} };
        (parent: $parent:expr, $lvl:expr, %$($k:ident).+ = $($field:tt)*) => { {} };
        (parent: $parent:expr, $lvl:expr, $($k:ident).+, $($field:tt)*) => { {} };
        (parent: $parent:expr, $lvl:expr, %$($k:ident).+, $($field:tt)*) => { {} };
        (parent: $parent:expr, $lvl:expr, ?$($k:ident).+, $($field:tt)*) => { {} };
        (parent: $parent:expr, $lvl:expr, $($arg:tt)+ ) => { {} };
        ( $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => { {} };
        ( $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => { {} };
        ($lvl:expr, $($k:ident).+ = $($field:tt)*) => { {} };
        ($lvl:expr, $($k:ident).+, $($field:tt)*) => { {} };
        ($lvl:expr, ?$($k:ident).+, $($field:tt)*) => { {} };
        ($lvl:expr, %$($k:ident).+, $($field:tt)*) => { {} };
        ($lvl:expr, ?$($k:ident).+) => { {} };
        ($lvl:expr, %$($k:ident).+) => { {} };
        ($lvl:expr, $($k:ident).+) => { {} };
        ( $lvl:expr, $($arg:tt)+ ) => { {} };
    }
}

#[cfg(feature = "with-tracing")]
mod inner {
    use core::{
        module_path, concat, format_args, file, line,
    };
    use crate::{WasmMetadata};

    // just a simplistic holder for span and entered spans
    // that exits on drop
    pub struct Span(u64);     // 0 means no item
    pub struct Entered(u64);  // 0 means no item

    impl Span {
        pub fn enter(self) -> Entered {
            if self.0 != 0 {
                crate::get_tracing_subscriber().map(|t|  t.enter(self.0));
            }
            Entered(self.0)
        }
    }

    impl Entered {
        pub fn exit(&mut self) {
            if self.0 != 0 {
                crate::get_tracing_subscriber().map(|t| t.exit(self.0));
            }
        }
    }

    impl Drop for Entered { 
        fn drop(&mut self) {
            self.exit();
        }
    }

    /// Constructs a new `Event`.
    ///
    /// The event macro is invoked with a `Level` and up to 32 key-value fields.
    /// Optionally, a format string and arguments may follow the fields; this will
    /// be used to construct an implicit field named "message".
    ///
    /// See [the top-level documentation][lib] for details on the syntax accepted by
    /// this macro.
    ///
    /// [lib]: index.html#using-the-macros
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing::{event, Level};
    ///
    /// # fn main() {
    /// let data = (42, "forty-two");
    /// let private_data = "private";
    /// let error = "a bad error";
    ///
    /// event!(Level::ERROR, %error, "Received error");
    /// event!(
    ///     target: "app_events",
    ///     Level::WARN,
    ///     private_data,
    ///     ?data,
    ///     "App warning: {}",
    ///     error
    /// );
    /// event!(Level::INFO, the_answer = data.0);
    /// # }
    /// ```
    ///
    // /// Note that *unlike `span!`*, `event!` requires a value for all fields. As
    // /// events are recorded immediately when the macro is invoked, there is no
    // /// opportunity for fields to be recorded later. A trailing comma on the final
    // /// field is valid.
    // ///
    // /// For example, the following does not compile:
    // /// ```rust,compile_fail
    // /// # #[macro_use]
    // /// # extern crate tracing;
    // /// # use tracing::Level;
    // /// # fn main() {
    // /// event!(Level::INFO, foo = 5, bad_field, bar = "hello")
    // /// #}
    // /// ```
    #[macro_export]
    macro_rules! event {
        (target: $target:expr, parent: $parent:expr, $lvl:expr, { $($fields:tt)* } )=> ({
            {
                if $crate::level_enabled!($lvl) {
                    #[allow(unused_imports)]
                    use $crate::{WasmMetadata, WasmEvent, WasmValue, WasmValueSet};
                    let metadata = WasmMetadata {
                        name: concat!(
                            "event ",
                            file!(),
                            ":",
                            line!()
                        ).as_bytes().to_vec(),
                        file: file!().as_bytes().to_vec(),
                        line: line!(),
                        is_span: false,
                        target: $target,
                        level: $lvl,
                        module_path: module_path().as_bytes().to_vec(),
                        fields: $($fields)*
                    };
                    if $crate::is_enabled!(&metdata) {
                        $crate::get_tracing_subscriber().map(|t| t.event(WasmEvent {
                            parent: $parent,
                            metadata,
                            &$crate::valueset!(meta.fields(), $($fields)*)
                        }));
                    }
                }
            }
        });

        (target: $target:expr, parent: $parent:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => ({
            $crate::event!(
                target: $target,
                parent: $parent,
                $lvl,
                { message = format_args!($($arg)+), $($fields)* }
            )
        });
        (target: $target:expr, parent: $parent:expr, $lvl:expr, $($k:ident).+ = $($fields:tt)* ) => (
            $crate::event!(target: $target, parent: $parent, $lvl, { $($k).+ = $($fields)* })
        );
        (target: $target:expr, parent: $parent:expr, $lvl:expr, $($arg:tt)+) => (
            $crate::event!(target: $target, parent: $parent, $lvl, { $($arg)+ })
        );
        (target: $target:expr, $lvl:expr, { $($fields:tt)* } )=> ({
            {
                if $crate::level_enabled!($lvl) {
                    #[allow(unused_imports)]
                    use $crate::{WasmMetadata, WasmEvent, WasmValue, WasmValueSet};
                    let metadata = WasmMetadata {
                        name: concat!(
                            "event ",
                            file!(),
                            ":",
                            line!()
                        ).as_bytes().to_vec(),
                        file: file!().as_bytes().to_vec(),
                        line: line!(),
                        is_span: false,
                        target: $target,
                        level: $lvl,
                        module_path: module_path().as_bytes().to_vec(),
                        fields: $($fields)*
                    };
                    if $crate::is_enabled!(&metdata) {
                        $crate::get_tracing_subscriber().map(|t| t.event(WasmEvent {
                            parent: None,
                            metadata,
                            &$crate::valueset!(meta.fields(), $($fields)*)
                        }));
                    }
                }
            }
        });
        (target: $target:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => ({
            $crate::event!(
                target: $target,
                $lvl,
                { message = format_args!($($arg)+), $($fields)* }
            )
        });
        (target: $target:expr, $lvl:expr, $($k:ident).+ = $($fields:tt)* ) => (
            $crate::event!(target: $target, $lvl, { $($k).+ = $($fields)* })
        );
        (target: $target:expr, $lvl:expr, $($arg:tt)+ ) => (
            $crate::event!(target: $target, $lvl, { $($arg)+ })
        );
        (parent: $parent:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => (
            $crate::event!(
                target: module_path!(),
                parent: $parent,
                $lvl,
                { message = format_args!($($arg)+), $($fields)* }
            )
        );
        (parent: $parent:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => (
            $crate::event!(
                target: module_path!(),
                $lvl,
                parent: $parent,
                { message = format_args!($($arg)+), $($fields)* }
            )
        );
        (parent: $parent:expr, $lvl:expr, $($k:ident).+ = $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                parent: $parent,
                $lvl,
                { $($k).+ = $($field)*}
            )
        );
        (parent: $parent:expr, $lvl:expr, ?$($k:ident).+ = $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                parent: $parent,
                $lvl,
                { ?$($k).+ = $($field)*}
            )
        );
        (parent: $parent:expr, $lvl:expr, %$($k:ident).+ = $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                parent: $parent,
                $lvl,
                { %$($k).+ = $($field)*}
            )
        );
        (parent: $parent:expr, $lvl:expr, $($k:ident).+, $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                parent: $parent,
                $lvl,
                { $($k).+, $($field)*}
            )
        );
        (parent: $parent:expr, $lvl:expr, %$($k:ident).+, $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                parent: $parent,
                $lvl,
                { %$($k).+, $($field)*}
            )
        );
        (parent: $parent:expr, $lvl:expr, ?$($k:ident).+, $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                parent: $parent,
                $lvl,
                { ?$($k).+, $($field)*}
            )
        );
        (parent: $parent:expr, $lvl:expr, $($arg:tt)+ ) => (
            $crate::event!(target: module_path!(), parent: $parent, $lvl, { $($arg)+ })
        );
        ( $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => (
            $crate::event!(
                target: module_path!(),
                $lvl,
                { message = format_args!($($arg)+), $($fields)* }
            )
        );
        ( $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => (
            $crate::event!(
                target: module_path!(),
                $lvl,
                { message = format_args!($($arg)+), $($fields)* }
            )
        );
        ($lvl:expr, $($k:ident).+ = $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                $lvl,
                { $($k).+ = $($field)*}
            )
        );
        ($lvl:expr, $($k:ident).+, $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                $lvl,
                { $($k).+, $($field)*}
            )
        );
        ($lvl:expr, ?$($k:ident).+, $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                $lvl,
                { ?$($k).+, $($field)*}
            )
        );
        ($lvl:expr, %$($k:ident).+, $($field:tt)*) => (
            $crate::event!(
                target: module_path!(),
                $lvl,
                { %$($k).+, $($field)*}
            )
        );
        ($lvl:expr, ?$($k:ident).+) => (
            $crate::event!($lvl, ?$($k).+,)
        );
        ($lvl:expr, %$($k:ident).+) => (
            $crate::event!($lvl, %$($k).+,)
        );
        ($lvl:expr, $($k:ident).+) => (
            $crate::event!($lvl, $($k).+,)
        );
        ( $lvl:expr, $($arg:tt)+ ) => (
            $crate::event!(target: module_path!(), $lvl, { $($arg)+ })
        );
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! __tracing_mk_span {
        (target: $target:expr, parent: $parent:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
            {
                if $crate::level_enabled!($lvl) {
                    #[allow(unused_imports)]
                    use $crate::{WasmMetadata, WasmAttributes, WasmValue, WasmValueSet};
                    let metadata = WasmMetadata {
                        name: $name.as_bytes().to_vec(),
                        file: file!().as_bytes().to_vec(),
                        line: line!(),
                        is_span: true,
                        target: $target,
                        level: $lvl,
                        module_path: module_path().as_bytes().to_vec(),
                        fields: $($fields)*
                    };
                    if $crate::is_enabled!(metadata) {
                        let span_id = $crate::get_tracing_subscriber().map(|t| t.new_span(WasmAttributes{
                            parent: Some($parent),
                            metadata,
                            &$crate::valueset!(meta.fields(), $($fields)*)
                        })).unwrap_or_default();
                        $crate::Span(span_id)
                    } else {
                        $crate::Span(0)
                    }
                } else {
                    $crate::Span(0)
                }
            }
        };
        (target: $target:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
            {
                if $crate::level_enabled!($lvl) {
                    #[allow(unused_imports)]
                    use $crate::{WasmMetadata, WasmAttributes, WasmValue, WasmValueSet};
                    let metadata = WasmMetadata {
                        name: $name.as_bytes().to_vec(),
                        file: file!().as_bytes().to_vec(),
                        line: line!(),
                        is_span: true,
                        target: $target,
                        level: $lvl,
                        module_path: module_path().as_bytes().to_vec(),
                        fields: $($fields)*
                    };
                    if $crate::is_enabled!(metadata) {
                        let span_id = $crate::get_tracing_subscriber().map(|t| t.new_span(WasmAttributes{
                            parent: Some($parent),
                            metadata,
                            &$crate::valueset!(meta.fields(), $($fields)*)
                        })).unwrap_or_default();
                        $crate::Span(span_id)
                    } else {
                        $crate::Span(0)
                    }
                } else {
                    $crate::Span(0)
                }
            }
        };
    }
}

pub use inner::*;

/// Constructs a new span.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: index.html#using-the-macros
///
/// # Examples
///
/// Creating a new span:
/// ```
/// # use tracing::{span, Level};
/// # fn main() {
/// let span = span!(Level::TRACE, "my span");
/// let _enter = span.enter();
/// // do work inside the span...
/// # }
/// ```
#[macro_export]
macro_rules! span {
    (target: $target:expr, parent: $parent:expr, $lvl:expr, $name:expr) => {
        $crate::span!(target: $target, parent: $parent, $lvl, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
        $crate::__tracing_mk_span!(target: $target, parent: $parent, $lvl, $name, $($fields)*)
    };
    (target: $target:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
        $crate::__tracing_mk_span!(target: $target, $lvl, $name, $($fields)*)
    };
    (target: $target:expr, parent: $parent:expr, $lvl:expr, $name:expr) => {
        $crate::span!(target: $target, parent: $parent, $lvl, $name,)
    };
    (parent: $parent:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            $name,
            $($fields)*
        )
    };
    (parent: $parent:expr, $lvl:expr, $name:expr) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            $name,
        )
    };
    (target: $target:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
        $crate::span!(
            target: $target,
            $lvl,
            $name,
            $($fields)*
        )
    };
    (target: $target:expr, $lvl:expr, $name:expr) => {
        $crate::span!(target: $target, $lvl, $name,)
    };
    ($lvl:expr, $name:expr, $($fields:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $lvl,
            $name,
            $($fields)*
        )
    };
    ($lvl:expr, $name:expr) => {
        $crate::span!(
            target: module_path!(),
            $lvl,
            $name,
        )
    };
}

/// Constructs a span at the trace level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: index.html#using-the-macros
/// [attributes]: index.html#configuring-attributes
/// [Fields]: index.html#recording-fields
/// [`span!`]: macro.span.html
///
/// # Examples
///
/// ```rust
/// # use tracing::{trace_span, span, Level};
/// # fn main() {
/// trace_span!("my_span");
/// // is equivalent to:
/// span!(Level::TRACE, "my_span");
/// # }
/// ```
///
/// ```rust
/// # use tracing::{trace_span, span, Level};
/// # fn main() {
/// let span = trace_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! trace_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::Level::TRACE,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::trace_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::trace_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::Level::TRACE,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::trace_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::Level::TRACE,
            $name,
            $($field)*
        )
    };
    ($name:expr) => { $crate::trace_span!($name,) };
}

/// Constructs a span at the debug level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: index.html#using-the-macros
/// [attributes]: index.html#configuring-attributes
/// [Fields]: index.html#recording-fields
/// [`span!`]: macro.span.html
///
/// # Examples
///
/// ```rust
/// # use tracing::{debug_span, span, Level};
/// # fn main() {
/// debug_span!("my_span");
/// // is equivalent to:
/// span!(Level::DEBUG, "my_span");
/// # }
/// ```
///
/// ```rust
/// # use tracing::debug_span;
/// # fn main() {
/// let span = debug_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! debug_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::Level::DEBUG,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::debug_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::debug_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::Level::DEBUG,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::debug_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::Level::DEBUG,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {$crate::debug_span!($name,)};
}

/// Constructs a span at the info level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: index.html#using-the-macros
/// [attributes]: index.html#configuring-attributes
/// [Fields]: index.html#recording-fields
/// [`span!`]: macro.span.html
///
/// # Examples
///
/// ```rust
/// # use tracing::{span, info_span, Level};
/// # fn main() {
/// info_span!("my_span");
/// // is equivalent to:
/// span!(Level::INFO, "my_span");
/// # }
/// ```
///
/// ```rust
/// # use tracing::info_span;
/// # fn main() {
/// let span = info_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! info_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::Level::INFO,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::info_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::info_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::Level::INFO,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::info_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::Level::INFO,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {$crate::info_span!($name,)};
}

/// Constructs a span at the warn level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: index.html#using-the-macros
/// [attributes]: index.html#configuring-attributes
/// [Fields]: index.html#recording-fields
/// [`span!`]: macro.span.html
///
/// # Examples
///
/// ```rust
/// # use tracing::{warn_span, span, Level};
/// # fn main() {
/// warn_span!("my_span");
/// // is equivalent to:
/// span!(Level::WARN, "my_span");
/// # }
/// ```
///
/// ```rust
/// use tracing::warn_span;
/// # fn main() {
/// let span = warn_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! warn_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::Level::WARN,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::warn_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::warn_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::Level::WARN,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::warn_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::Level::WARN,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {$crate::warn_span!($name,)};
}
/// Constructs a span at the error level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: index.html#using-the-macros
/// [attributes]: index.html#configuring-attributes
/// [Fields]: index.html#recording-fields
/// [`span!`]: macro.span.html
///
/// # Examples
///
/// ```rust
/// # use tracing::{span, error_span, Level};
/// # fn main() {
/// error_span!("my_span");
/// // is equivalent to:
/// span!(Level::ERROR, "my_span");
/// # }
/// ```
///
/// ```rust
/// # use tracing::error_span;
/// # fn main() {
/// let span = error_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! error_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::Level::ERROR,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::error_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::error_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::Level::ERROR,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::error_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::Level::ERROR,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {$crate::error_span!($name,)};
}

/// Constructs an event at the trace level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: macro.event.html
/// [lib]: index.html#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::trace;
/// # #[derive(Debug, Copy, Clone)] struct Position { x: f32, y: f32 }
/// # impl Position {
/// # const ORIGIN: Self = Self { x: 0.0, y: 0.0 };
/// # fn dist(&self, other: Position) -> f32 {
/// #    let x = (other.x - self.x).exp2(); let y = (self.y - other.y).exp2();
/// #    (x + y).sqrt()
/// # }
/// # }
/// # fn main() {
/// let pos = Position { x: 3.234, y: -1.223 };
/// let origin_dist = pos.dist(Position::ORIGIN);
///
/// trace!(position = ?pos, ?origin_dist);
/// trace!(
///     target: "app_events",
///     position = ?pos,
///     "x is {} and y is {}",
///     if pos.x >= 0.0 { "positive" } else { "negative" },
///     if pos.y >= 0.0 { "positive" } else { "negative" }
/// );
/// # }
/// ```
#[macro_export]
macro_rules! trace {
    (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::TRACE, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::TRACE, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::TRACE, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::TRACE, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::TRACE, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::TRACE,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::Level::TRACE, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::TRACE, { $($k).+ $($field)+ })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::TRACE, { $($k).+ $($field)+ })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::TRACE, { $($k).+ $($field)+ })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::TRACE, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { $($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            {},
            $($arg)+
        )
    );
}

/// Constructs an event at the debug level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: macro.event.html
/// [lib]: index.html#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::debug;
/// # fn main() {
/// # #[derive(Debug)] struct Position { x: f32, y: f32 }
///
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!(?pos.x, ?pos.y);
/// debug!(target: "app_events", position = ?pos, "New position");
/// # }
/// ```
#[macro_export]
macro_rules! debug {
    (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::DEBUG, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::DEBUG, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::DEBUG, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::DEBUG, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::DEBUG, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::DEBUG,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::Level::DEBUG, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::DEBUG, { $($k).+ $($field)+ })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::DEBUG, { $($k).+ $($field)+ })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::DEBUG, { $($k).+ $($field)+ })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::DEBUG, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { $($k).+ = $($field)*}
        )
    );
    (?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { ?$($k).+ = $($field)*}
        )
    );
    (%$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { %$($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::DEBUG,
            {},
            $($arg)+
        )
    );
}

/// Constructs an event at the info level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: macro.event.html
/// [lib]: index.html#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::info;
/// # // this is so the test will still work in no-std mode
/// # #[derive(Debug)]
/// # pub struct Ipv4Addr;
/// # impl Ipv4Addr { fn new(o1: u8, o2: u8, o3: u8, o4: u8) -> Self { Self } }
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// use tracing::field;
///
/// let addr = Ipv4Addr::new(127, 0, 0, 1);
/// let conn = Connection { port: 40, speed: 3.20 };
///
/// info!(conn.port, "connected to {:?}", addr);
/// info!(
///     target: "connection_events",
///     ip = ?addr,
///     conn.port,
///     ?conn.speed,
/// );
/// # }
/// ```
#[macro_export]
macro_rules! info {
     (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::INFO, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::INFO, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::INFO, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::INFO, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::INFO, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::INFO,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::Level::INFO, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::INFO, { $($k).+ $($field)+ })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::INFO, { $($k).+ $($field)+ })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::INFO, { $($k).+ $($field)+ })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::INFO, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { $($k).+ = $($field)*}
        )
    );
    (?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { ?$($k).+ = $($field)*}
        )
    );
    (%$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { %$($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::INFO,
            {},
            $($arg)+
        )
    );
}

/// Constructs an event at the warn level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: macro.event.html
/// [lib]: index.html#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::warn;
/// # fn main() {
///
/// let warn_description = "Invalid Input";
/// let input = &[0x27, 0x45];
///
/// warn!(?input, warning = warn_description);
/// warn!(
///     target: "input_events",
///     warning = warn_description,
///     "Received warning for input: {:?}", input,
/// );
/// # }
/// ```
#[macro_export]
macro_rules! warn {
     (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::WARN, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::WARN, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::WARN, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::WARN, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::WARN, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::WARN,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::Level::WARN, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::WARN, { $($k).+ $($field)+ })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::WARN, { $($k).+ $($field)+ })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::WARN, { $($k).+ $($field)+ })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::WARN, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { $($k).+ = $($field)*}
        )
    );
    (?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { ?$($k).+ = $($field)*}
        )
    );
    (%$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { %$($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::WARN,
            {},
            $($arg)+
        )
    );
}

/// Constructs an event at the error level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: macro.event.html
/// [lib]: index.html#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::error;
/// # fn main() {
///
/// let (err_info, port) = ("No connection", 22);
///
/// error!(port, error = %err_info);
/// error!(target: "app_events", "App Error: {}", err_info);
/// error!({ info = err_info }, "error on port: {}", port);
/// # }
/// ```
#[macro_export]
macro_rules! error {
     (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::ERROR, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::ERROR, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::ERROR, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::ERROR, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::Level::ERROR, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::Level::ERROR,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::Level::ERROR, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::ERROR, { $($k).+ $($field)+ })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::ERROR, { $($k).+ $($field)+ })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::ERROR, { $($k).+ $($field)+ })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::Level::ERROR, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { $($k).+ = $($field)*}
        )
    );
    (?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { ?$($k).+ = $($field)*}
        )
    );
    (%$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::TRACE,
            { %$($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::Level::ERROR,
            {},
            $($arg)+
        )
    );
}


#[macro_export]
// TODO: determine if this ought to be public API?
#[doc(hidden)]
macro_rules! level_enabled {
    ($lvl:expr) => {
        // FIXME: use the runtime interface to figure this out
        true
        // $crate::dispatcher::has_been_set() && $lvl <= $crate::level_filters::STATIC_MAX_LEVEL
        
    };
}

#[macro_export]
// TODO: determine if this ought to be public API?
#[doc(hidden)]
macro_rules! is_enabled {
    ($metadata:expr) => {{
        $crate::get_tracing_subscriber().map(|t| t.enabled($metadata)).unwrap_or_false()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! valueset {

    // === base case ===
    (@ { $(,)* $($val:expr),* $(,)* }, $next:expr $(,)*) => {
        vec![ $($val),* ]
    };

    // === recursive case (more tts) ===

    // TODO(#1138): determine a new syntax for uninitialized span fields, and
    // re-enable this.
    // (@{ $(,)* $($out:expr),* }, $next:expr, $($k:ident).+ = _, $($rest:tt)*) => {
    //     $crate::valueset!(@ { $($out),*, (&$next, None) }, $next, $($rest)*)
    // };
    (@ { $(,)* $($out:expr),* }, $next:expr, $($k:ident).+ = ?$val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&debug(&$val) as &Value)) },
            $next,
            $($rest)*
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, $($k:ident).+ = %$val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&display(&$val) as &Value)) },
            $next,
            $($rest)*
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, $($k:ident).+ = $val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&$val as &Value)) },
            $next,
            $($rest)*
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, $($k:ident).+, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&$($k).+ as &Value)) },
            $next,
            $($rest)*
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, ?$($k:ident).+, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&debug(&$($k).+) as &Value)) },
            $next,
            $($rest)*
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, %$($k:ident).+, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&display(&$($k).+) as &Value)) },
            $next,
            $($rest)*
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, $($k:ident).+ = ?$val:expr) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&debug(&$val) as &Value)) },
            $next,
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, $($k:ident).+ = %$val:expr) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&display(&$val) as &Value)) },
            $next,
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, $($k:ident).+ = $val:expr) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&$val as &Value)) },
            $next,
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, $($k:ident).+) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&$($k).+ as &Value)) },
            $next,
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, ?$($k:ident).+) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&debug(&$($k).+) as &Value)) },
            $next,
        )
    };
    (@ { $(,)* $($out:expr),* }, $next:expr, %$($k:ident).+) => {
        $crate::valueset!(
            @ { $($out),*, (&$next, Some(&display(&$($k).+) as &Value)) },
            $next,
        )
    };
    // Remainder is unparseable, but exists --- must be format args!
    (@ { $(,)* $($out:expr),* }, $next:expr, $($rest:tt)+) => {
        $crate::valueset!(@ { (&$next, Some(&format_args!($($rest)+) as &Value)), $($out),* }, $next, )
    };

    // === entry ===
    ($fields:expr, $($kvs:tt)+) => {
        {
            #[allow(unused_imports)]
            use $crate::field::{debug, display, Value};
            let mut iter = $fields.iter();
            $fields.value_set($crate::valueset!(
                @ { },
                iter.next().expect("FieldSet corrupted (this is a bug)"),
                $($kvs)+
            ))
        }
    };
    ($fields:expr,) => {
        {
            $fields.value_set(&[])
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! fieldset {
    // == base case ==
    (@ { $(,)* $($out:expr),* $(,)* } $(,)*) => {
        vec![ $($out),* ]
    };

    // == recursive cases (more tts) ==
    (@ { $(,)* $($out:expr),* } $($k:ident).+ = ?$val:expr, $($rest:tt)*) => {
        $crate::fieldset!(@ { $($out),*, $crate::__tracing_stringify!($($k).+) } $($rest)*)
    };
    (@ { $(,)* $($out:expr),* } $($k:ident).+ = %$val:expr, $($rest:tt)*) => {
        $crate::fieldset!(@ { $($out),*, $crate::__tracing_stringify!($($k).+) } $($rest)*)
    };
    (@ { $(,)* $($out:expr),* } $($k:ident).+ = $val:expr, $($rest:tt)*) => {
        $crate::fieldset!(@ { $($out),*, $crate::__tracing_stringify!($($k).+) } $($rest)*)
    };
    // TODO(#1138): determine a new syntax for uninitialized span fields, and
    // re-enable this.
    // (@ { $($out:expr),* } $($k:ident).+ = _, $($rest:tt)*) => {
    //     $crate::fieldset!(@ { $($out),*, $crate::__tracing_stringify!($($k).+) } $($rest)*)
    // };
    (@ { $(,)* $($out:expr),* } ?$($k:ident).+, $($rest:tt)*) => {
        $crate::fieldset!(@ { $($out),*, $crate::__tracing_stringify!($($k).+) } $($rest)*)
    };
    (@ { $(,)* $($out:expr),* } %$($k:ident).+, $($rest:tt)*) => {
        $crate::fieldset!(@ { $($out),*, $crate::__tracing_stringify!($($k).+) } $($rest)*)
    };
    (@ { $(,)* $($out:expr),* } $($k:ident).+, $($rest:tt)*) => {
        $crate::fieldset!(@ { $($out),*, $crate::__tracing_stringify!($($k).+) } $($rest)*)
    };

    // Remainder is unparseable, but exists --- must be format args!
    (@ { $(,)* $($out:expr),* } $($rest:tt)+) => {
        $crate::fieldset!(@ { "message", $($out),*, })
    };

    // == entry ==
    ($($args:tt)*) => {
        $crate::fieldset!(@ { } $($args)*,)
    };

}

#[doc(hidden)]
#[macro_export]
macro_rules! __tracing_stringify {
    ($s:expr) => {
        stringify!($s)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! if_log_enabled {
    ($e:expr;) => {
        $crate::if_log_enabled! { $e }
    };
    ($if_log:block) => {
        $crate::if_log_enabled! { $if_log else {} }
    };
    ($if_log:block else $else_block:block) => {
        $else_block
    };
}