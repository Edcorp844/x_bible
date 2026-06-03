use std::sync::Arc;

use relm4::prelude::*;
use xbible_engine::engines::xbible_engine::engine::XBibleEngine;

pub struct SearchPage {
    pub(crate) _engine: Arc<XBibleEngine>,
}

#[relm4::component(pub)]
impl Component for SearchPage {
    type Init = Arc<XBibleEngine>;
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
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { _engine: init };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        _message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
    }
}
