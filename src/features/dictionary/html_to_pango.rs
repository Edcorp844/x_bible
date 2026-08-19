use regex::Regex;

/// Converts HTML to Pango markup for GTK labels
pub fn html_to_pango_markup(html: &str) -> String {
    let mut result = html.to_string();

    // Remove the search key prefix/suffix (case-insensitive)
    // This is typically the word being looked up
    result = result.trim().to_string();

    // Basic HTML to Pango conversions
    
    // Bold: <b> or <strong> → <b>
    result = Regex::new(r"</?strong>").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"<b>(.*?)</b>").unwrap().replace_all(&result, "<b>$1</b>").to_string();

    // Italic: <i>, <em>, <cite> → <i>
    result = Regex::new(r"<em>(.*?)</em>").unwrap().replace_all(&result, "<i>$1</i>").to_string();
    result = Regex::new(r"<i>(.*?)</i>").unwrap().replace_all(&result, "<i>$1</i>").to_string();
    result = Regex::new(r"<cite>(.*?)</cite>").unwrap().replace_all(&result, "<i>$1</i>").to_string();

    // Underline: <u> → <u>
    result = Regex::new(r"<u>(.*?)</u>").unwrap().replace_all(&result, "<u>$1</u>").to_string();

    // Small text: <small> → smaller font (handled by CSS, but we can use span)
    result = Regex::new(r"<small>(.*?)</small>").unwrap().replace_all(&result, "<span font_size='smaller'>$1</span>").to_string();

    // Superscript: <sup> → handled as regular text with smaller font
    result = Regex::new(r"<sup>(.*?)</sup>").unwrap().replace_all(&result, "<span font_size='smaller' rise='3000'>$1</span>").to_string();

    // Subscript: <sub>
    result = Regex::new(r"<sub>(.*?)</sub>").unwrap().replace_all(&result, "<span font_size='smaller' rise='-3000'>$1</span>").to_string();

    // Remove other HTML tags that don't have Pango equivalents
    // Keep content but remove tags like <div>, <span class="...">, <p>, etc.
    
    // <span class="..."> → <span>
    result = Regex::new(r#"<span[^>]*class="([^"]*)"[^>]*>"#).unwrap().replace_all(&result, "<span>").to_string();
    
    // <div> and </div> → remove but keep content
    result = Regex::new(r"</?div[^>]*>").unwrap().replace_all(&result, "").to_string();

    // <p> and </p> → remove but keep content (add newline)
    result = Regex::new(r"</p>").unwrap().replace_all(&result, "\n").to_string();
    result = Regex::new(r"<p[^>]*>").unwrap().replace_all(&result, "").to_string();

    // <br> and <br/> → newline
    result = Regex::new(r"<br\s*/?>").unwrap().replace_all(&result, "\n").to_string();

    // Remove line breaks and excessive whitespace
    result = Regex::new(r"\s+").unwrap().replace_all(&result, " ").to_string();

    // Decode common HTML entities
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&quot;", "\"");
    result = result.replace("&#39;", "'");
    result = result.replace("&apos;", "'");
    result = result.replace("&nbsp;", " ");

    // Remove any remaining HTML comments
    result = Regex::new(r"<!--.*?-->").unwrap().replace_all(&result, "").to_string();

    // Remove script and style tags with their content
    result = Regex::new(r"<script[^>]*>.*?</script>").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"<style[^>]*>.*?</style>").unwrap().replace_all(&result, "").to_string();

    // Remove any remaining unclosed tags
    result = Regex::new(r"<[^>]*>").unwrap().replace_all(&result, "").to_string();

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold_conversion() {
        let html = "<b>bold text</b>";
        let result = html_to_pango_markup(html);
        assert!(result.contains("<b>bold text</b>"));
    }

    #[test]
    fn test_italic_conversion() {
        let html = "<em>italic text</em>";
        let result = html_to_pango_markup(html);
        assert!(result.contains("<i>italic text</i>"));
    }

    #[test]
    fn test_html_entities() {
        let html = "&amp; &lt; &gt;";
        let result = html_to_pango_markup(html);
        assert_eq!(result, "& < >");
    }
}