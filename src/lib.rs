//! Strict Rust port of the Dart `alfred_workflow` package.
//!
//! The crate is library-first and focuses on Alfred Script Filter JSON,
//! workflow output, file caching, plist user defaults, and GitHub updater
//! behavior with Dart-compatible wire formats.
//!
//! ```
//! use alfred_workflow_rs::{Item, Result, Workflow};
//!
//! # fn main() -> Result<()> {
//! let mut workflow = Workflow::new();
//! workflow.add_item(Item::with_arg("Search Google", "https://www.google.com"))?;
//!
//! assert_eq!(
//!     workflow.to_json_string()?,
//!     r#"{"items":[{"title":"Search Google","type":"default","valid":false,"arg":"https://www.google.com"}]}"#
//! );
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

/// Alfred action model types.
pub mod action;
/// Automatic Script Filter cache metadata.
pub mod automatic_cache;
/// File-backed workflow cache types.
pub mod cache;
/// Crate error and result types.
pub mod error;
/// Alfred Script Filter item model types.
pub mod item;
/// Alfred Script Filter result collection model.
pub mod items;
/// GitHub release updater support.
pub mod updater;
/// Alfred user configuration plist parsing.
pub mod user_config;
/// In-memory workflow builder and renderer.
pub mod workflow;

pub use action::{Action, ActionText, TypedAction};
pub use automatic_cache::AutomaticCache;
pub use cache::{FileCache, FileCacheBuilder, WorkflowCache};
pub use error::{Error, Result};
pub use item::{Icon, IconType, Item, ItemBuilder, ItemText, ItemType, Modifier, ModifierKey};
pub use items::Items;
pub use updater::{
    CommandOpener, GithubAsset, GithubRelease, GithubUser, Opener, Updater, UpdaterBuilder,
    parse_version_tag,
};
pub use user_config::{
    CheckBoxConfiguration, CheckBoxUserConfiguration, FilePickerConfiguration,
    FilePickerUserConfiguration, NumberSliderConfiguration, NumberSliderUserConfiguration,
    SelectConfiguration, SelectPair, SelectUserConfiguration, TextAreaConfiguration,
    TextAreaUserConfiguration, TextFieldConfiguration, TextFieldUserConfiguration,
    UserConfiguration, UserConfigurationType, UserPreferences,
};
pub use workflow::{RenderOptions, Workflow, WorkflowBuilder};
