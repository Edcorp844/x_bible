use crate::features::bible::components::page::biblepage_model::BiblePage;

impl BiblePage {
    pub fn make_css_preview_clss(&self, theme: String) -> String {
        format!("preview-area-{}", theme)
    }
}
