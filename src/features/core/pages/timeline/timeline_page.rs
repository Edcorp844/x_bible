use std::sync::Arc;

use adw::prelude::*;
use relm4::{Component, ComponentParts};
use xbible_engine::engines::xbible_engine::engine::XBibleEngine;

pub struct TimelinePage {
    pub(crate) engine: Arc<XBibleEngine>,
}

#[derive(Clone, Debug)]
pub enum TimelinePageInput {}

#[derive(Clone, Debug)]
pub enum TimelinePageOutput {
    ToggleSidebar,
}

#[relm4::component(pub)]
impl Component for TimelinePage {
    type Init = Arc<XBibleEngine>;
    type Input = TimelinePageInput;
    type Output = TimelinePageOutput;
    type CommandOutput = ();

    fn init(
        engine: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let model = TimelinePage { engine };

        //let timeline = TimelineData::new();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    view! {
         #[root]
            adw::NavigationPage {
                set_title: "Audio Bible",


                #[wrap(Some)]
                set_child = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle { set_title: "Audio Bible" },
                        set_show_title: false,
                        add_css_class: "flat",

                        pack_start = &gtk::ToggleButton {
                            set_icon_name: "sidebar-show-symbolic",
                            connect_clicked[sender] => move |_| {
                                let _ = sender.output(TimelinePageOutput::ToggleSidebar);
                            }
                        }
                    },

                    #[wrap(Some)]
                    set_content = &adw::Clamp {
                        set_maximum_size: 1500,
                        set_tightening_threshold: 1000,

                        #[wrap(Some)]
                        set_child = &gtk::Box {}

                    }

                }
            }
    }
}
