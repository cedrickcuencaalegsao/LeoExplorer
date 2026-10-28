use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use iced::Color;
use serde::Deserialize;

const TEMPLATE: &str = r##"# LeoExplorer configuration
#
# Icon colors use hex codes: #rgb, #rrggbb or #rrggbbaa
# Keys are icon names (the file names in assets/icons without .svg).
# Restart LeoExplorer after editing.

[icons]
# default = "#1a1a1a"
# house = "#ffffff"
# hard-drive = "#3b82f6"
# folder = "#b45309"
# file-text = "#6b7280"
"##;

#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    icons: HashMap<String, String>,
}

#[derive(Debug, Default)]
struct IconColors {
    map: HashMap<String, Color>,
    default: Option<Color>,
}

static ICON_COLORS: OnceLock<IconColors> = OnceLock::new();

pub fn config_path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        dirs::config_dir()
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
    };
    base.map(|dir| dir.join("leoexplorer").join("config.toml"))
}

pub fn init() {
    ICON_COLORS.get_or_init(load);
}

pub fn icon_color(name: &str) -> Option<Color> {
    let key = name.strip_suffix(".svg").unwrap_or(name);
    let colors = ICON_COLORS.get_or_init(load);
    colors.map.get(key).copied().or(colors.default)
}

pub fn parse_hex(value: &str) -> Option<Color> {
    let hex = value.trim().trim_start_matches('#');

    let expanded: String = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).collect(),
        6 | 8 => hex.to_string(),
        _ => return None,
    };

    let channel = |i: usize| u8::from_str_radix(&expanded[i..i + 2], 16).ok();

    let r = channel(0)?;
    let g = channel(2)?;
    let b = channel(4)?;
    let a = if expanded.len() == 8 { channel(6)? } else { 255 };

    Some(Color::from_rgba8(r, g, b, a as f32 / 255.0))
}

fn load() -> IconColors {
    let Some(path) = config_path() else {
        return IconColors::default();
    };

    if !path.exists() {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(&path, TEMPLATE);
        return IconColors::default();
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("config: cannot read {}: {err}", path.display());
            return IconColors::default();
        }
    };

    let raw: RawConfig = match toml::from_str(&text) {
        Ok(raw) => raw,
        Err(err) => {
            eprintln!("config: invalid TOML in {}: {err}", path.display());
            return IconColors::default();
        }
    };

    let mut colors = IconColors::default();

    for (name, value) in raw.icons {
        match parse_hex(&value) {
            Some(color) if name == "default" => colors.default = Some(color),
            Some(color) => {
                colors.map.insert(name, color);
            }
            None => eprintln!("config: ignoring invalid color {value:?} for icon {name:?}"),
        }
    }

    colors
}