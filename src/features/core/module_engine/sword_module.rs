#[derive(Debug, Clone)]
pub struct SwordModule {
    pub name: String,
    pub description: String,
    pub category: String,
    pub language: String,
    pub version: String,
    pub delta: String,
    pub cipher_key: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleBook {
    pub name: String,
    pub chapters: Vec<ModuleChapter>,
}

#[derive(Debug, Clone)]
pub struct ModuleChapter {
    pub number: i32,
    pub verse_count: i32,
}
