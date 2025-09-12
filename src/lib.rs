#![allow(stable_features)]
#![feature(iter_advance_by)]
#![feature(let_chains)]
#![feature(maybe_uninit_array_assume_init)]
#![no_std]

#[cfg(feature = "de")]
pub mod de;

#[cfg(feature = "derive")]
pub mod derive;

#[cfg(feature = "ser")]
pub mod ser;
