use adw::prelude::*;
use gettextrs::gettext;

#[derive(Debug)]
pub(crate) struct StudyView {}

#[derive(Debug)]
pub enum StudyViewInput {
    SetSidebarCollapsed(bool),
}

#[derive(Debug)]
pub enum StudyViewOutput {
    ShowError(String),
}

#[relm4::component(pub)]
impl relm4::Component for StudyView {
    type Init = ();
    type Input = StudyViewInput;
    type Output = StudyViewOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage{
            set_title: &gettext("Study"),
            #[wrap(Some)]
            set_child=&adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_title_widget: Some(&adw::WindowTitle::new(&gettext("Study"), ""))
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }
}
