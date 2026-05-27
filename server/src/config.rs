use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! config_field {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            pub fn $name(&self) -> std::sync::Arc<$ty> {
                self.[<$name _field>].load_full()
            }
            pub fn [<set_ $name>](&self, value: $ty) {
                self.[<$name _field>].store(std::sync::Arc::new(value));
            }
        }
    };
}

macro_rules! config_field_deref {
    ($name:ident, $ty:ty, $inner:ty) => {
        paste::paste! {
            pub fn $name(&self) -> $inner {
                let arc = self.[<$name _field>].load_full();
                (*arc).clone()
            }
            pub fn [<set_ $name>](&self, value: $ty) {
                self.[<$name _field>].store(std::sync::Arc::new(value));
            }
        }
    };
}

pub(crate) struct Config {
    ignored_loc_regex_field: Arc<arc_swap::ArcSwap<Vec<regex::Regex>>>,
    ignored_files_regex_field: Arc<arc_swap::ArcSwap<Vec<regex::Regex>>>,
    workspace_scan_enabled_field: AtomicBool,
    styling_enabled_field: AtomicBool,
    cosmetic_loc_indent_field: AtomicBool,
    game_path_field: Arc<arc_swap::ArcSwap<Option<String>>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            ignored_loc_regex_field: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
            ignored_files_regex_field: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
            workspace_scan_enabled_field: AtomicBool::new(false),
            styling_enabled_field: AtomicBool::new(true),
            cosmetic_loc_indent_field: AtomicBool::new(false),
            game_path_field: Arc::new(arc_swap::ArcSwap::from_pointee(None)),
        }
    }

    config_field!(ignored_loc_regex, Vec<regex::Regex>);
    config_field!(ignored_files_regex, Vec<regex::Regex>);
    pub fn workspace_scan_enabled(&self) -> bool {
        self.workspace_scan_enabled_field.load(Ordering::Relaxed)
    }

    pub fn set_workspace_scan_enabled(&self, value: bool) {
        self.workspace_scan_enabled_field
            .store(value, Ordering::Relaxed);
    }

    pub fn styling_enabled(&self) -> bool {
        self.styling_enabled_field.load(Ordering::Relaxed)
    }

    pub fn set_styling_enabled(&self, value: bool) {
        self.styling_enabled_field.store(value, Ordering::Relaxed);
    }

    pub fn cosmetic_loc_indent(&self) -> bool {
        self.cosmetic_loc_indent_field.load(Ordering::Relaxed)
    }

    pub fn set_cosmetic_loc_indent(&self, value: bool) {
        self.cosmetic_loc_indent_field
            .store(value, Ordering::Relaxed);
    }

    config_field_deref!(game_path, Option<String>, Option<String>);
}
