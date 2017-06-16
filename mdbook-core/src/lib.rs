//! This crate contains the core components of `mdbook`, allowing third parties
//! to seamlessly integrate with the rest of the project while generating a
//! rendered version of your document.

// #![deny(missing_docs,
//         missing_debug_implementations,
//         missing_copy_implementations,
//         trivial_casts,
//         trivial_numeric_casts,
//         unsafe_code,
//         unused_import_braces,
//         unused_qualifications,
//         unstable_features)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate serde;

#[cfg(test)]
extern crate toml;

pub mod runner;
pub mod config;


pub mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error) #[doc = "A `std::io::Error`"];
        }
    }
}
