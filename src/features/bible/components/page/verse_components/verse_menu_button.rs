use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender};

pub struct VerseMenuButton {
    label: String,
    icon: String,
}

#[relm4::component(pub)]
impl Component for VerseMenuButton {
    type Init = (String, String);
    type Input = ();
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_spacing: 12,
            set_orientation: gtk::Orientation::Vertical,

            gtk::ToggleButton{
                set_icon_name: model.icon.as_str(),
            },

            gtk::Label{
                set_markup: &format!("<span size='x-small'>{}</span>", model.label),
                set_halign: gtk::Align::Center,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (label, icon) = init;

        let model = Self { label, icon };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
