use gettextrs::gettext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XBibleNavigationPage {
    Study,
    Store,
}

impl XBibleNavigationPage {
    pub fn from_name(name: &str) -> Self {
        let localized_name = gettext(name);

        if localized_name == gettext("Study") {
            Self::Study
        } else if localized_name == gettext("Store") {
            Self::Store
        } else {
            Self::Store
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::Study => gettext("Study"),
            Self::Store => gettext("Store"),
        }
    }

    pub fn to_key(&self) -> String {
        match self {
            Self::Study => "Study".to_string(),
            Self::Store => "Store".to_string(),
        }
    }

    pub fn icon_name(&self) -> String {
        match self {
            Self::Study => "bible-read".to_string(),
            Self::Store => "my-store".to_string(),
        }
    }
}
