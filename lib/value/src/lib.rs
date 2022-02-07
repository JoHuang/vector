//! The `value` crate contains types shared across Vector libraries to support it's use of `Value`
//! and the closely linked `Kind` in support of progressive type checking.

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::pedantic,
    future_incompatible,
    missing_docs,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rustdoc::bare_urls,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::private_intra_doc_links,
    unused
)]
#![allow(
    clippy::cargo_common_metadata,
    clippy::single_match_else,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    unreachable_code,
    unused_variables
)]
pub mod kind;
mod value;

pub use self::value::{Value, ValueRegex};
pub use kind::Kind;