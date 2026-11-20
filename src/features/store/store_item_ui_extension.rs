use xbible_engine::features::store::StoreItem;

use crate::core::ui::color::RGBColor;

pub(crate) trait StoreItemUIExt {
    /// DJB2 hash over `item.id` mapped onto 8 distinct cover hues, matching
    /// `StoreItem.safeHue` / `StoreItem.safeColor`.
    fn safe_hue(&self, item_id: &str) -> f64 {
        let mut h: u64 = 5381;
        for b in item_id.as_bytes() {
            h = h.wrapping_shl(5).wrapping_add(h).wrapping_add(*b as u64);
        }
        const HUES: [f64; 8] = [0.02, 0.08, 0.14, 0.32, 0.55, 0.65, 0.78, 0.88];
        HUES[(h as usize) % HUES.len()]
    }

    /// Converts HSV (`h` in `[0, 1)`, `s` and `v` in `[0, 1]`) to `(r, g, b)` in `[0.0, 1.0]`.
    fn hsv_to_rgb(&self, h: f64, s: f64, v: f64) -> RGBColor {
        let h6 = (h.rem_euclid(1.0)) * 6.0;
        let sector = h6.floor() as i32;
        let f = h6 - sector as f64;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        match sector.rem_euclid(6) {
            0 => RGBColor::new(v, t, p),
            1 => RGBColor::new(q, v, p),
            2 => RGBColor::new(p, v, t),
            3 => RGBColor::new(p, q, v),
            4 => RGBColor::new(t, p, v),
            _ => RGBColor::new(v, p, q),
        }
    }

    fn safe_rgb(&self) -> RGBColor;
    fn language_display(&self) -> String;
}

impl StoreItemUIExt for StoreItem {
    fn safe_rgb(&self) -> RGBColor {
        self.hsv_to_rgb(self.safe_hue(&self.id), 0.55, 0.65)
    }

    fn language_display(&self) -> String {
        if let Some(ref name) = self.language_name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        self.language.clone()
    }
}
