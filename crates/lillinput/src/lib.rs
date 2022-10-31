//! Library for connecting libinput gestures to i3 and others.
//!
//! `lillinput` is a small for utility for connecting `libinput` gestures to:
//! * commands for the `i3` tiling window manager `IPC` interface
//! * shell commands
//!
//! This crate provides the library. See also the [`lillinput-cli`] crate for
//! the commandline interface.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown
)]

pub mod actions;
pub mod controllers;
pub mod events;
#[cfg(test)]
pub mod test_utils;
