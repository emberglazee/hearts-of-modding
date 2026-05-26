use std::sync::Arc;

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
    workspace_scan_enabled_field: Arc<arc_swap::ArcSwap<bool>>,
    styling_enabled_field: Arc<arc_swap::ArcSwap<bool>>,
    cosmetic_loc_indent_field: Arc<arc_swap::ArcSwap<bool>>,
    game_path_field: Arc<arc_swap::ArcSwap<Option<String>>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            ignored_loc_regex_field: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
            ignored_files_regex_field: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
            workspace_scan_enabled_field: Arc::new(arc_swap::ArcSwap::from_pointee(false)),
            styling_enabled_field: Arc::new(arc_swap::ArcSwap::from_pointee(true)),
            cosmetic_loc_indent_field: Arc::new(arc_swap::ArcSwap::from_pointee(false)),
            game_path_field: Arc::new(arc_swap::ArcSwap::from_pointee(None)),
        }
    }

    config_field!(ignored_loc_regex, Vec<regex::Regex>);
    config_field!(ignored_files_regex, Vec<regex::Regex>);
    config_field_deref!(workspace_scan_enabled, bool, bool);
    config_field_deref!(styling_enabled, bool, bool);
    config_field_deref!(cosmetic_loc_indent, bool, bool);
    config_field_deref!(game_path, Option<String>, Option<String>);
}
