use adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug, Clone)]
pub struct AnnotationColor {
    pub color: String,
}

#[derive(Debug, Clone)]
pub enum AnnotationOutput {
    Selected(String),
}

#[relm4::component(pub)]
impl Component for AnnotationColor {
    type Init = String;
    type Input = String;
    type Output = AnnotationOutput;
    type CommandOutput = ();

    view! {
        #[root]
        // Using a Box/Frame is much more stable for dynamic background colors
        #[name = "swatch"]
        gtk::Box {
            set_width_request: 32,
            set_height_request: 32,
            set_cursor_from_name: Some("pointer"),
            add_css_class: "annotation-swatch",

            // Set initial rounded corners via CSS
            inline_css: "border-radius: 15px; border: 1px solid rgba(0,0,0,0.1);",

            add_controller = gtk::GestureClick {
                set_button: 1,
                connect_released[sender, hex = model.color.clone()] => move |gesture, _, _, _| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    let _ = sender.output(AnnotationOutput::Selected(hex.clone()));
                }
            }
        }
    }

    fn init(
        color: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { color };
        let widgets = view_output!();

        // Apply the color immediately on init
        let hex = model.color.clone();
        widgets.swatch.inline_css(&format!(
            "background-color: {}; border-radius: 15px; border: 1px solid rgba(0,0,0,0.1);",
            hex
        ));

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.color = message;

        // Update the UI dynamically using the widgets reference
        widgets.swatch.inline_css(&format!(
            "background-color: {}; border-radius: 15px; border: 1px solid rgba(0,0,0,0.1);",
            self.color
        ));
    }
}
