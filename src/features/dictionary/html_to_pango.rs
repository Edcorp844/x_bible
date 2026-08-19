use gtk::gdk::RGBA;
use gtk::prelude::*;
use regex::Regex;

// ============================================================================
// 1. Theme Color Extraction Helper
// ============================================================================

/// Resolves a named GTK/Libadwaita theme color (e.g., "warning_color", "warning_bg_color")
/// into a standard hex string format (`#RRGGBB`) using the widget's current `StyleContext`.
pub fn get_theme_color_hex(widget: &impl IsA<gtk::Widget>, color_name: &str) -> String {
    let style_context = widget.style_context();

    // Query the named color with fallback defaults if the theme color isn't present
    let rgba = style_context.lookup_color(color_name).unwrap_or_else(|| {
        match color_name {
            "warning_bg_color" => RGBA::builder().red(0.98).green(0.68).blue(0.24).alpha(0.15).build(),
            "warning_fg_color" => RGBA::builder().red(0.98).green(0.68).blue(0.24).alpha(1.0).build(),
            _ => RGBA::builder().red(0.98).green(0.68).blue(0.24).alpha(1.0).build(), // Fallback warning yellow/amber
        }
    });

    rgba_to_hex(&rgba)
}

/// Converts a `gdk::RGBA` struct into an uppercase hex string (`#RRGGBB`).
pub fn rgba_to_hex(rgba: &RGBA) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (rgba.red() * 255.0).round() as u8,
        (rgba.green() * 255.0).round() as u8,
        (rgba.blue() * 255.0).round() as u8
    )
}

// ============================================================================
// 2. HTML to Pango Markup Converter
// ============================================================================

/// Converts dictionary HTML content into valid Pango markup for GTK4 labels.
///
/// Accepts dynamic hex theme colors (such as those retrieved via `get_theme_color_hex`):
/// - `primary_color`: Applied to orthographic forms (`class="orth"`)
/// - `secondary_color`: Applied to part-of-speech tags (`class="pos"`)
/// - `quote_bg`: Applied to block quotes (`<div class="cit">` / `<blockquote>`)
pub fn html_to_pango_markup(
    html: &str,
    key: Option<&str>,
    primary_color: &str,
    secondary_color: &str,
    quote_bg: &str,
) -> String {
    let mut text = html.trim().to_string();

    // 1. Strip search key prefix/suffix if present (case-insensitive)
    if let Some(k) = key {
        let key_lower = k.trim().to_lowercase();
        if !key_lower.is_empty() {
            if text.to_lowercase().starts_with(&key_lower) {
                text = text[key_lower.len()..].trim().to_string();
            }
            if text.to_lowercase().ends_with(&key_lower) {
                let new_len = text.len() - key_lower.len();
                text = text[..new_len].trim().to_string();
            }
        }
    }

    // 2. Strip script, style tags, and HTML comments along with contents
    text = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap().replace_all(&text, "").to_string();
    text = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap().replace_all(&text, "").to_string();
    text = Regex::new(r"(?s)<!--.*?-->").unwrap().replace_all(&text, "").to_string();

    // 3. Class Transformations (orth & pos styling)
    let orth_re = Regex::new(r#"(?i)<([^>]*)\bclass="orth"([^>]*)>"#).unwrap();
    text = orth_re.replace_all(&text, format!(r#"<$1 foreground="{primary_color}" weight="bold"$2>"#)).to_string();

    let pos_re = Regex::new(r#"(?i)<([^>]*)\bclass="pos"([^>]*)>"#).unwrap();
    text = pos_re.replace_all(&text, format!(r#"<$1 foreground="{secondary_color}" style="italic"$2>"#)).to_string();

    // 4. Transform Blockquotes (`<div class="cit">` and `<blockquote>`)
    let cit_open = Regex::new(r#"(?i)<div\s+class="cit"[^>]*>|<blockquote>"#).unwrap();
    text = cit_open.replace_all(&text, format!(r#"\n<span background="{quote_bg}" style="italic" indent="24000">\n"#)).to_string();

    let cit_close = Regex::new(r#"(?i)</div>|</sub>|</blockquote>"#).unwrap();
    let cit_close_div = Regex::new(r#"(?i)</div>|blockquote>"#).unwrap();
    text = Regex::new(r"(?i)</div>|</blockquote>").unwrap().replace_all(&text, "\n</span>\n").to_string();

    // 5. Semantic tag conversions to GTK/Pango equivalents
    text = Regex::new(r"(?i)</?strong>").unwrap().replace_all(&text, "<b>").to_string();
    text = Regex::new(r"(?i)<strong>(.*?)</strong>").unwrap().replace_all(&text, "<b>$1</b>").to_string();

    text = Regex::new(r"(?i)<em>(.*?)</em>").unwrap().replace_all(&text, "<i>$1</i>").to_string();
    text = Regex::new(r"(?i)<cite>(.*?)</cite>").unwrap().replace_all(&text, "<i>$1</i>").to_string();

    text = Regex::new(r"(?i)<small>(.*?)</small>").unwrap().replace_all(&text, r#"<span size="small">$1</span>"#).to_string();
    text = Regex::new(r"(?i)<sup>(.*?)</sup>").unwrap().replace_all(&text, r#"<span size="small" rise="6000">$1</span>"#).to_string();
    text = Regex::new(r"(?i)<sub>(.*?)</sub>").unwrap().replace_all(&text, r#"<span size="small" rise="-6000">$1</span>"#).to_string();

    // Linebreaks and Paragraphs
    text = Regex::new(r"(?i)<br\s*/?>").unwrap().replace_all(&text, "\n").to_string();
    text = Regex::new(r"(?i)</p>").unwrap().replace_all(&text, "\n\n").to_string();
    text = Regex::new(r"(?i)<p[^>]*>").unwrap().replace_all(&text, "").to_string();

    // 6. Strip unhandled div/span containers but preserve their child tags
    text = Regex::new(r"(?i)</?div[^>]*>").unwrap().replace_all(&text, "").to_string();

    // 7. Collapse excessive horizontal whitespace while preserving single newlines
    text = Regex::new(r"[ \t]+").unwrap().replace_all(&text, " ").to_string();
    text = Regex::new(r"\n\s*\n").unwrap().replace_all(&text, "\n\n").to_string();

    // 8. Escape Pango-reserved structural entities
    text = text.replace("&nbsp;", " ");

    text.trim().to_string()
}
