//! # freedesktop-icon-lookup
//!
//! A library for searching a path with given app name based on [Freedesktop icon lookup spec](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html).
//! The implementation eagarly scans all directories in advance so multiple lookups are executed way faster and without real filesystem scan.
//!
//! # Feature list
//!
//! - Multiple themes support, including inherited;
//! - Advanced lookup among all found
//!
//! # Example
//!
//! ```rust
//! # use std::path::PathBuf;
//! use freedesktop_icon_lookup::{Lookup, LookupParam};
//! # use freedesktop_icon_lookup::Result;
//!
//! # fn main() -> Result<()> {
//! let theme = "Adwaita";
//! let mut cache = Cache::new()?;
//! cache.load(theme)?;
//! let _: Option<PathBuf> = cache.lookup("firefox", theme)?;
//! # Ok(())
//! # }
//!
//! # Alternatives
//!
//! [freedesktop-icons](https://crates.io/crates/freedesktop-icons) might be a better option if you only need a few icons to search.
//! ```

pub use err::{Error, Result};
pub use lookup::{Cache, LookupParam};
pub use theme::IconInfo;

pub(crate) use directory::Directory;
pub(crate) use lookup::find_dir_icons;
pub(crate) use theme::{IconSearch, Theme};

mod directory;
mod err;
mod lookup;
mod theme;
