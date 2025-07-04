use egui::{
    Color32, CornerRadius, FontFamily, FontId, Rect, Response, Stroke, StrokeKind, Ui, Vec2,
};

// PADDING & SPACING
pub const SPACE_AMOUNT: f32 = 8.0;
pub const PANEL_SPACING: f32 = 5.0;
pub const PANEL_PADDING: f32 = 8.0;

// Colors
pub const BACKGROUND_COLOR: Color32 = Color32::from_rgb(80, 80, 120);
pub const BACKGROUND_COLOR_FOCUSED: Color32 = Color32::from_rgb(60, 60, 80);
pub const BACKGROUND_COLOR_HOVERED: Color32 = Color32::from_rgb(90, 90, 130);
pub const BACKGROUND_COLOR_ACTIVE: Color32 = Color32::from_rgb(50, 50, 70);

pub const TEXT_COLOR: Color32 = Color32::from_rgb(220, 220, 220);
pub const TEXT_COLOR_DISABLED: Color32 = Color32::from_rgb(120, 120, 120);
pub const TEXT_COLOR_ACCENT: Color32 = Color32::from_rgb(255, 200, 100);

pub const BORDER_COLOR: Color32 = Color32::from_rgb(100, 100, 140);
pub const BORDER_COLOR_FOCUSED: Color32 = Color32::from_rgb(150, 150, 180);

pub const BUTTON_COLOR: Color32 = Color32::from_rgb(70, 70, 110);
pub const BUTTON_COLOR_HOVERED: Color32 = Color32::from_rgb(85, 85, 125);
pub const BUTTON_COLOR_ACTIVE: Color32 = Color32::from_rgb(55, 55, 95);

// Stroke
pub const STANDARD_STROKE: f32 = 1.0;
pub const FOCUSED_STROKE: f32 = 2.0;
pub const THIN_STROKE: f32 = 3.0;

// Spacing and sizing
pub const STANDARD_SPACING: f32 = 8.0;
pub const SMALL_SPACING: f32 = 4.0;
pub const LARGE_SPACING: f32 = 16.0;

pub const BUTTON_HEIGHT: f32 = 24.0;
pub const INPUT_HEIGHT: f32 = 20.0;

// Rounding
pub const STANDARD_ROUNDING: CornerRadius = CornerRadius::same(4);
pub const SMALL_ROUNDING: CornerRadius = CornerRadius::same(2);
pub const LARGE_ROUNDING: CornerRadius = CornerRadius::same(8);

// Font sizes
pub const FONT_SIZE_SMALL: f32 = 10.0;
pub const FONT_SIZE_NORMAL: f32 = 12.0;
pub const FONT_SIZE_LARGE: f32 = 16.0;
pub const FONT_SIZE_HEADING: f32 = 20.0;

// Font IDs (you can customize these based on your needs)
pub const FONT_SMALL: FontId = FontId::new(FONT_SIZE_SMALL, FontFamily::Proportional);
pub const FONT_NORMAL: FontId = FontId::new(FONT_SIZE_NORMAL, FontFamily::Proportional);
pub const FONT_LARGE: FontId = FontId::new(FONT_SIZE_LARGE, FontFamily::Proportional);
pub const FONT_HEADING: FontId = FontId::new(FONT_SIZE_HEADING, FontFamily::Proportional);
pub const FONT_MONO: FontId = FontId::new(FONT_SIZE_NORMAL, FontFamily::Monospace);

// Helper function to apply your theme to the entire app
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Window styling
    style.visuals.window_fill = BACKGROUND_COLOR;
    style.visuals.window_stroke = Stroke::new(STANDARD_STROKE, TEXT_COLOR);

    // Panel styling
    style.visuals.panel_fill = BACKGROUND_COLOR;

    // Button styling
    style.visuals.widgets.inactive.bg_fill = BUTTON_COLOR;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(STANDARD_STROKE, TEXT_COLOR);
    style.visuals.widgets.inactive.corner_radius = STANDARD_ROUNDING;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(STANDARD_STROKE, TEXT_COLOR);

    style.visuals.widgets.hovered.bg_fill = BUTTON_COLOR_HOVERED;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(STANDARD_STROKE, TEXT_COLOR_ACCENT);
    style.visuals.widgets.hovered.corner_radius = STANDARD_ROUNDING;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_COLOR);

    style.visuals.widgets.active.bg_fill = BUTTON_COLOR_ACTIVE;
    style.visuals.widgets.active.bg_stroke = Stroke::new(STANDARD_STROKE, TEXT_COLOR_ACCENT);
    style.visuals.widgets.active.corner_radius = STANDARD_ROUNDING;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_COLOR);

    // Text styling
    style.visuals.override_text_color = Some(TEXT_COLOR);

    // Spacing
    style.spacing.item_spacing = Vec2::splat(STANDARD_SPACING);
    style.spacing.button_padding = Vec2::new(STANDARD_SPACING, SMALL_SPACING);

    ctx.set_style(style);
}

pub fn apply_theme_on_rect(
    ui: &mut Ui,
    rect: Rect,
    response: &Response,
    text: &str,
    is_active: bool,
) {
    let bg_color = if is_active {
        BACKGROUND_COLOR_ACTIVE
    } else if response.hovered() {
        BACKGROUND_COLOR_HOVERED
    } else {
        BACKGROUND_COLOR
    };

    let border_color = if is_active || response.hovered() {
        BORDER_COLOR_FOCUSED
    } else {
        BORDER_COLOR
    };

    let text_color = if response.hovered() {
        TEXT_COLOR_ACCENT
    } else {
        TEXT_COLOR
    };

    // Draw background
    ui.painter().rect_filled(rect, STANDARD_ROUNDING, bg_color);

    // Draw border
    ui.painter().rect_stroke(
        rect,
        STANDARD_ROUNDING,
        Stroke::new(1.0, border_color),
        StrokeKind::Middle,
    );

    // Draw text
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FONT_NORMAL,
        text_color,
    );
}
