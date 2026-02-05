use adw::prelude::*;
use relm4::{prelude::*, view};

use crate::features::bible::transilation_engines::osis_engine::{
    components::word::AddedWordStyle, helpers::Verse,
};

pub struct Page {
    verses: FactoryVecDeque<Verse>,
    added_style: AddedWordStyle,
}

#[derive(Debug)]
pub enum PageInput {
    LoadChapter(String),
    ToggleAddedStyle,
}

#[relm4::component(pub)]
impl Component for Page {
    type Init = Vec<Verse>;
    type Input = PageInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box{
        gtk::ScrolledWindow {
            set_vexpand: true,
            set_hscrollbar_policy: gtk::PolicyType::Never, 

            #[local_ref]
            verse_list -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 30,
                set_spacing: 8,
            }
        }
    }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Page> {
        let verses_data = init;
        let verse_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let mut verses = FactoryVecDeque::builder()
            .launch(verse_container.clone())
            .detach();

        {
            let mut guard = verses.guard();
            for v in verses_data {
                guard.push_back(v);
            }
        }

        let model = Page {
            verses,
            added_style: AddedWordStyle::Italic,
        };
        let verse_list = &verse_container;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
