use crate::features::{store::store_view::StoreView, study::study_view::StudyView};
use relm4::{ComponentController, Controller};

#[derive(Debug)]

pub enum PageController {
    Study(Controller<StudyView>),
    Store(Controller<StoreView>),
}

impl PageController {
    pub(crate) fn widget(&self) -> &adw::NavigationPage {
        match self {
            Self::Study(c) => c.widget(),
            Self::Store(c) => c.widget(),
        }
    }
}
