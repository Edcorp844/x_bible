use gtk::prelude::*;

use crate::features::{bible::components::page::{
    biblepage_model::{BiblePage, StudyInput},
    helpers::AvailableFonts,
    verse_components::verse::VerseInputMessage,
}, core::display_configurations::config::TextConfig};

impl BiblePage {
    pub fn font_menu_widget(
        font: AvailableFonts,
        sender: relm4::ComponentSender<Self>,
        config: TextConfig
    ) -> gtk::Box {
        let is_active = font == config.read().unwrap().font();
        let name = font.to_string();

        let font_box = gtk::Box::builder()
            .css_classes(vec!["menu-font-option", "clickable"])
            .cursor(&gtk::gdk::Cursor::from_name("pointer", None).unwrap())
            .build();

        if is_active {
            font_box.add_css_class("menu-font-option-active");
        }

        let markup = match font {
            AvailableFonts::System => format!("<span size='small'>{}</span>", name),
            _ => format!("<span face='{0}' size='small'>{0}</span>", name),
        };

        font_box.append(&gtk::Label::builder().use_markup(true).label(markup).build());

        let click = gtk::GestureClick::new();
        let font_clone = font.clone();
        click.connect_released(move |_, _, _, _| {
            sender.input(StudyInput::ToggleDisplay(VerseInputMessage::ChangeFont(
                font_clone.clone(),
            )));
        });

        font_box.add_controller(click);
        font_box
    }

    pub fn populate_fonts_container(
        container: &gtk::Box,
        sender: relm4::ComponentSender<Self>,
        config: TextConfig,
    ) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        for font in AvailableFonts::all() {
            let widget = Self::font_menu_widget(font, sender.clone(), config.clone());
            container.append(&widget);
        }
    }
}
