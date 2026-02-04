use adw::prelude::*;
use relm4::{FactorySender, prelude::*};

use crate::features::bible::components::page::{helpers::Verse, word::AddedWordStyle};

// --- VERSE FACTORY ---
#[relm4::factory(pub)]
impl FactoryComponent for Verse {
    type Init = Verse;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 12,
            set_hexpand: true,

            gtk::Label {
                add_css_class: "verser-number",
                set_markup: &format!(
                    "<span size='large'>{}</span>",
                    self.number
                ),
                set_valign: gtk::Align::Start,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,

                #[local_ref]
                word_flow -> adw::WrapBox {
                    set_line_spacing: 12,
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    //set_margin_start: if self.is_paragraph_start { 40 } else { 0 },
                },



                #[local_ref]
                notes_container -> adw::WrapBox{
                    #[watch]
                    set_visible: !self.notes.is_empty(),
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        _root: Self::Root,
        _returned_widget: &gtk::Widget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        // Create the WrapBox that will hold flowing words
        let word_flow_box = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        // Populate words directly (no factory)
        for word in &self.words {
            let word = word.build_widget(AddedWordStyle::Brackets);

            word_flow_box.append(&word);
        }

        // 🔑 This is the injection point the macro expects
        let word_flow = &word_flow_box;
        let notes_container = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();
       // print!("\n{} ", self.number);
        for word in self.notes.clone() {
            let note_label = gtk::Label::builder()
                .wrap(true)
                .margin_end(16)
                .css_classes(vec!["verse-note"])
                .build();
            note_label.set_markup(
                format!("<span size='large' foreground='#d71452'><span size='small' foreground='#c314d7'><i>Note on Verse {}: </i></span><i>{}</i></span>",self.number, word).as_str(),
            );
            notes_container.append(&note_label);
        }
        let widgets = view_output!();

        widgets
    }
}
