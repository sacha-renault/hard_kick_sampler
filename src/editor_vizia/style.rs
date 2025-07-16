use nih_plug_vizia::vizia::style::Color;

pub const PANEL_SPACING: f32 = 16.0;
pub const MAIN_PADDING: f32 = 16.0;
pub const PANEL_PADDING: f32 = 8.0;
pub const BORDER_RADIUS: f32 = 10.0;

pub const THEMES_VAR: &[(&str, &str)] = &[
    ("background-color", "#0f0f0f"),
    ("background-secondary", "#1a1a1a"),
    ("background-tertiary", "#2d1b2e"),
    ("primary-color", "#e91e63"),
    ("secondary-color", "#9c27b0"),
    ("accent-color", "#673ab7"),
    ("accent-secondary", "#4a148c"),
    ("text-primary", "#ffffff"),
    ("text-secondary", "#cccccc"),
    ("text-accent", "#f06292"),
    ("border-color", "#2d1b2e"),
    ("border-light", "#4a148c"),
    ("hover-color", "#673ab7"),
    ("active-color", "#e91e63"),
    ("shadow-color", "#0f0f0f"),
    ("gradient-start", "#e91e63"),
    ("gradient-mid", "#9c27b0"),
    ("gradient-end", "#673ab7"),
    ("surface-color", "#1a1a1a"),
    ("surface-elevated", "#2d1b2e"),
    // Extras
    ("focus-color", "#e91e63"),
    ("disabled-color", "#555555"),
    ("primary-alpha-25", "#e91e634d"),
    ("primary-alpha-20", "#e91e6333"),
    ("primary-alpha-10", "#e91e631a"),
    ("primary-alpha-50", "#e91e6380"),
    ("secondary-alpha-25", "#9c27b040"),
    ("background-alpha-08", "#1a1a1a14"),
    ("shadow-alpha-22", "#0f0f0f38"),
];
