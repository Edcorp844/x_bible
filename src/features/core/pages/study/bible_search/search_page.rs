use std::sync::Arc;
use relm4::prelude::*;

use crate::features::core::module_engine::sword_engine::SwordEngine;

pub struct SearchPage{
    pub(crate) engine: Arc<SwordEngine>,
}

#[relm4::component(pub)]
impl Component for SearchPage {
    type Init = Arc<SwordEngine>;
    type Input = ();
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box{
            
        }
    }

     fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let model = Self{
            engine: init
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    )
    {

    }

}