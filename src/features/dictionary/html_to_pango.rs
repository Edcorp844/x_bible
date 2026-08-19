use regex::Regex;

/// Converts HTML to valid Pango markup for GTK labels without dynamic color injection.
pub fn html_to_pango_markup(html: &str) -> String {
    let mut result = html.trim().to_string();

    // 1. Remove script, style, and comments first (along with their contents)
    result = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"(?s)<!--.*?-->").unwrap().replace_all(&result, "").to_string();

    // 2. Semantic tag conversions to standard Pango tags
    result = Regex::new(r"(?i)</?strong>").unwrap().replace_all(&result, "").to_string();
    
    // Bold, Italic, Underline
    result = Regex::new(r"(?i)<em>(.*?)</em>").unwrap().replace_all(&result, "<i>$1</i>").to_string();
    result = Regex::new(r"(?i)<cite>(.*?)</cite>").unwrap().replace_all(&result, "<i>$1</i>").to_string();
    result = Regex::new(r"(?i)<u>(.*?)</u>").unwrap().replace_all(&result, "<u>$1</u>").to_string();

    // Font sizing and positioning (using Pango's standard font_scale & rise attributes)
    result = Regex::new(r"(?i)<small>(.*?)</small>").unwrap().replace_all(&result, r#"<span font_scale="small">$1</span>"#).to_string();
    result = Regex::new(r"(?i)<sup>(.*?)</sup>").unwrap().replace_all(&result, r#"<span font_scale="small" rise="3000">$1</span>"#).to_string();
    result = Regex::new(r"(?i)<sub>(.*?)</sub>").unwrap().replace_all(&result, r#"<span font_scale="small" rise="-3000">$1</span>"#).to_string();

    // 3. Structure, Paragraphs & Line breaks
    result = Regex::new(r"(?i)<br\s*/?>").unwrap().replace_all(&result, "\n").to_string();
    result = Regex::new(r"(?i)</p>").unwrap().replace_all(&result, "\n\n").to_string();
    result = Regex::new(r"(?i)<p[^>]*>").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"(?i)</?div[^>]*>").unwrap().replace_all(&result, "").to_string();

    // 4. Strip unknown wrapper spans (like <span class="def">) while preserving valid Pango spans
    result = Regex::new(r#"(?i)<span\b[^>]*class="[^"]*"[^>]*>"#).unwrap().replace_all(&result, "").to_string();

    // 5. Clean up non-Pango HTML tags remaining in the string
    // Strip tags that aren't <b>, <i>, <u>, or <span>
    let unsupported_tags = Regex::new(r#"(?i)</?(?!(?:b|i|u|span)\b)[a-z1-6]+[^>]*>"#).unwrap();
    result = unsupported_tags.replace_all(&result, "").to_string();

    // 6. Whitespace cleanup (preserve linebreaks!)
    result = Regex::new(r"[ \t]+").unwrap().replace_all(&result, " ").to_string();
    result = Regex::new(r"\n\s*\n+").unwrap().replace_all(&result, "\n\n").to_string();

    // 7. Entity decoding safe for Pango (do NOT unescape & to raw ampersands!)
    result = result.replace("&nbsp;", " ");
    result = result.replace("&quot;", "\"");
    result = result.replace("&#39;", "'");
    result = result.replace("&apos;", "'");

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold_conversion() {
        let html = "<b>bold text</b>";
        let result = html_to_pango_markup(html);
        assert_eq!(result, "<b>bold text</b>");
    }

    #[test]
    fn test_italic_conversion() {
        let html = "<em>italic text</em>";
        let result = html_to_pango_markup(html);
        assert_eq!(result, "<i>italic text</i>");
    }

    #[test]
    fn test_container_stripping() {
        let html = r#"<span class="def"><b>VA'RIOUS</b>, <i>adjective</i></span>"#;
        let result = html_to_pango_markup(html);
        assert_eq!(result, "<b>VA'RIOUS</b>, <i>adjective</i>");
    }

    #[test]
    fn test_linebreaks_preserved() {
        let html = "Line 1<br>Line 2<p>Paragraph</p>";
        let result = html_to_pango_markup(html);
        assert_eq!(result, "Line 1\nLine 2\n\nParagraph");
    }
}