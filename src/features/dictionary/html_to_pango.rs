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

    let rgba = style_context.lookup_color(color_name).unwrap_or_else(|| {
        match color_name {
            "warning_bg_color" => RGBA::builder().red(0.98).green(0.68).blue(0.24).alpha(0.15).build(),
            "warning_fg_color" => RGBA::builder().red(0.98).green(0.68).blue(0.24).alpha(1.0).build(),
            _ => RGBA::builder().red(0.98).green(0.68).blue(0.24).alpha(1.0).build(), // Fallback warning amber
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

    // 2. Strip scripts, styles, and comments
    text = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap().replace_all(&text, "").to_string();
    text = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap().replace_all(&text, "").to_string();
    text = Regex::new(r"(?s)<!--.*?-->").unwrap().replace_all(&text, "").to_string();

    // 3. Class Transformations (`class="orth"` & `class="pos"`)
    let orth_re = Regex::new(r#"(?i)<span\b[^>]*\bclass="orth"[^>]*>(.*?)</span>"#).unwrap();
    text = orth_re.replace_all(&text, format!(r#"<span foreground="{primary_color}" weight="bold">$1</span>"#)).to_string();

    let pos_re = Regex::new(r#"(?i)<span\b[^>]*\bclass="pos"[^>]*>(.*?)</span>"#).unwrap();
    text = pos_re.replace_all(&text, format!(r#"<span foreground="{secondary_color}" style="italic">$1</span>"#)).to_string();

    // 4. Blockquotes (`<div class="cit">` and `<blockquote>`)
    let cit_open = Regex::new(r#"(?i)<div\s+class="cit"[^>]*>|<blockquote>"#).unwrap();
    text = cit_open.replace_all(&text, format!(r#"\n<span background="{quote_bg}" style="italic" indent="24000">\n"#)).to_string();

    let cit_close = Regex::new(r"(?i)</div>|</sub>|blockquote>").unwrap();
    text = Regex::new(r"(?i)</div>|</blockquote>").unwrap().replace_all(&text, "\n</span>\n").to_string();

    // 5. Semantic HTML tag conversions
    text = Regex::new(r"(?i)</?strong>").unwrap().replace_all(&text, "<b>").to_string();
    text = Regex::new(r"(?i)<em>(.*?)</em>").unwrap().replace_all(&text, "<i>$1</i>").to_string();
    text = Regex::new(r"(?i)<cite>(.*?)</cite>").unwrap().replace_all(&text, "<i>$1</i>").to_string();
    text = Regex::new(r"(?i)<small>(.*?)</small>").unwrap().replace_all(&text, r#"<span size="small">$1</span>"#).to_string();
    text = Regex::new(r"(?i)<sup>(.*?)</sup>").unwrap().replace_all(&text, r#"<span size="small" rise="6000">$1</span>"#).to_string();
    text = Regex::new(r"(?i)<sub>(.*?)</sub>").unwrap().replace_all(&text, r#"<span size="small" rise="-6000">$1</span>"#).to_string();

    // Linebreaks and Paragraphs
    text = Regex::new(r"(?i)<br\s*/?>").unwrap().replace_all(&text, "\n").to_string();
    text = Regex::new(r"(?i)</p>").unwrap().replace_all(&text, "\n\n").to_string();
    text = Regex::new(r"(?i)<p[^>]*>").unwrap().replace_all(&text, "").to_string();

    // 6. STRIP UNKNOWN HTML CONTAINERS & ATTRIBUTES
    // Remove wrapper tags like `<span class="def">`, `<div class="...">`, etc.
    text = Regex::new(r#"(?i)<span\b[^>]*class="(?!orth|pos)[^"]*"[^>]*>"#).unwrap().replace_all(&text, "").to_string();
    text = Regex::new(r"(?i)</?div[^>]*>").unwrap().replace_all(&text, "").to_string();
    text = Regex::new(r#"(?i)<span\b(?![^>]*\b(foreground|background|size|rise|style|weight)\b)[^>]*>"#).unwrap().replace_all(&text, "").to_string();

    // Clean up orphan closing spans if outer wrappers were removed
    // (Only keep valid Pango tags)

    // 7. Whitespace normalization
    text = Regex::new(r"[ \t]+").unwrap().replace_all(&text, " ").to_string();
    text = Regex::new(r"\n\s*\n").unwrap().replace_all(&text, "\n\n").to_string();

    // 8. Entity cleanup
    text = text.replace("&nbsp;", " ");

    text.trim().to_string()
}

// ============================================================================
// 3. Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_container_stripping() {
        let input = r#"<span class="def"><b>VA'RIOUS</b>, <i>adjective</i> [Latin varius.]</span>"#;
        let markup = html_to_pango_markup(input, None, "#F5C211", "#E5A50A", "#303030");
        assert_eq!(markup, "<b>VA'RIOUS</b>, <i>adjective</i> [Latin varius.]");
    }
}