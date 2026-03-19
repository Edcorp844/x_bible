pub(crate) struct ColorGenerator {}

impl ColorGenerator {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn generate_book_cover_color_according_to_language(lang_name: &str) -> &'static str {
        match lang_name {
            // --- Existing Languages (Refined for Calmness) ---
            "English" => "#4a5d4e",  // Muted Sage Green (Calming & Organic)
            "Japanese" => "#8c4a4a", // Dusty Rose/Red (Softened from Ruby)
            "Ancient Greek" | "Greek, Modern (1453-)" => "#455a64", // Slate Blue (Professional)
            "Hebrew" => "#6d5a41",   // Warm Sandstone Brown
            "Spanish" | "Castilian" => "#a15d48", // Terracotta (Warmer, less aggressive)
            "Latin" => "#514a75",    // Dusk Purple
            "French" => "#4a6a8a",   // Steel Blue
            "German" => "#37474f",   // Deep Anthracite (Soft Black)

            // --- New Languages ---
            "Burmese" => "#7e526e",    // Plum/Orchid (Sophisticated)
            "Vietnamese" => "#2d5a5e", // Deep Teal (Quiet and focused)
            "Korean" => "#454d66",     // Midnight Navy
            "Arabic" => "#967d44",     // Muted Gold/Brass (Rich but calm)

            // --- Defaults ---
            "Unknown" => "#546e7a", // Blue-Grey
            _ => "#4f5b62",         // Default Slate
        }
    }
}
