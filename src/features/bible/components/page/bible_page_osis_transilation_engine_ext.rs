use crate::features::bible::components::page::biblepage_model::BiblePage;

impl BiblePage {
    pub fn load_reference(&mut self, reference: &str) {
        let sections = self.engine.get_whole_chapter(&self.module, reference);
        let mut guard = self.sections.guard();
        guard.clear();
        for section in sections {
            guard.push_back((section, self.config.clone(), self.annotations.clone()));
        }
    }
}
