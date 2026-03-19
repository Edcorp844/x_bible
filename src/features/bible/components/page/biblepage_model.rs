use adw::prelude::*;
use gtk::glib::clone;
use relm4::prelude::*;
use std::sync::{Arc, RwLock};

use crate::features::bible::components::page::biblepage_settings::{
    BiblePageSettings, BiblePageState,
};
use crate::features::bible::components::page::helpers::{AddedWordStyle, PageDisplayConfig};
use crate::features::bible::components::page::section::{
    SectionInput, SectionModel, SectionOutput,
};
use crate::features::bible::components::page::verse_components::verse::VerseInputMessage;
use crate::features::bible::components::page::verse_components::verse_annotation::{
    AnnotationSettings, Annotations,
};
use crate::features::bible::components::page_theme::customize_theme_popup::{
    CustomizeThemeOutput, CustomizeThemePopup,
};
use crate::features::core::display_configurations::Config::TextConfig;
use crate::features::core::module_engine::sword_engine::SwordEngine;
use crate::features::core::module_engine::sword_engine_books_and_chapter_ext::{
    CategorizedBook, Testament,
};
use crate::features::core::module_engine::sword_engine_dictionary_ext::DictionaryQuery;
use crate::features::core::module_engine::sword_module::SwordModule;

pub struct BiblePage {
    pub(crate) engine: Arc<SwordEngine>,
    pub(crate) module: SwordModule,
    pub(crate) sections: FactoryVecDeque<SectionModel>,
    pub(crate) config: TextConfig,
    pub(crate) customize_theme_popup: Option<Controller<CustomizeThemePopup>>,
    pub(crate) annotations: Annotations,

    pub(crate) current_book_index: usize,
    pub(crate) current_book: CategorizedBook,
    pub(crate) current_chapter: i32,
}

#[derive(Debug)]
pub enum StudyInput {
    LoadReference(String),
    SetModule(SwordModule),
    SetBook(usize),
    SetChapter(i32),
    ToggleDisplay(VerseInputMessage),
    SetConfig(TextConfig),
    OpenCustomizethemePopup,
    CloseCustomizethemePopup,
}

#[derive(Debug)]
pub enum StudyPageOutput {
    ChangeTheme,
    LookupSelectedStrong(DictionaryQuery),
}

#[relm4::component(pub)]
impl Component for BiblePage {
    type Init = Arc<SwordEngine>;
    type Input = StudyInput;
    type Output = StudyPageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name="page_overlay"]
                gtk::Overlay {
                    set_vexpand: true,
                    #[watch]
                    set_css_classes: &[
                        "page-overlay",
                        &model.make_css_preview_clss(model.config.read().unwrap().theme())
                    ],

                 gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Fill,
                    set_hexpand: true,

                    gtk::Box {
                        add_css_class: "linked",
                        add_css_class: "rounded",
                        set_halign: gtk::Align::Center,
                        set_margin_bottom: 10,

                        gtk::MenuButton {
                            set_tooltip_text: Some("Select Version"),
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_spacing: 4,
                                #[name = "version_label"]
                                gtk::Label { set_label: &model.module.name },
                                gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                            },

                            #[name = "version_popover"]
                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                set_autohide: true,
                                gtk::ScrolledWindow {
                                    set_hscrollbar_policy: gtk::PolicyType::Never,
                                    set_min_content_width: 600,
                                    set_min_content_height: 400,
                                    set_max_content_height: 600,
                                    #[name = "bible_grid"]
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                    },
                                },
                            },
                        },

                        gtk::MenuButton {
                            set_tooltip_text: Some("Select Book"),
                            #[name = "book_popover"]
                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                gtk::ScrolledWindow {
                                    set_hscrollbar_policy: gtk::PolicyType::Never,
                                    set_min_content_width: 600,
                                    set_min_content_height: 400,
                                    set_max_content_height: 600,
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 10,
                                        set_margin_all: 12,
                                        #[name = "ot_container"]
                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_visible: false,
                                            gtk::Label {
                                                set_label: "Old Testament",
                                                set_halign: gtk::Align::Start,
                                                add_css_class: "title-4",
                                                set_margin_bottom: 8,
                                            },
                                            #[name = "ot_grid"]
                                            gtk::FlowBox {
                                                set_max_children_per_line: 4,
                                                set_min_children_per_line: 4,
                                                set_selection_mode: gtk::SelectionMode::None,
                                                set_column_spacing: 8,
                                                set_row_spacing: 8,
                                            },
                                        },
                                        #[name = "nt_container"]
                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_visible: false,
                                            set_margin_top: 20,
                                            gtk::Label {
                                                set_label: "New Testament",
                                                set_halign: gtk::Align::Start,
                                                add_css_class: "title-4",
                                                set_margin_bottom: 8,
                                            },
                                            #[name = "nt_grid"]
                                            gtk::FlowBox {
                                                set_max_children_per_line: 4,
                                                set_min_children_per_line: 4,
                                                set_selection_mode: gtk::SelectionMode::None,
                                                set_column_spacing: 8,
                                                set_row_spacing: 8,
                                            },
                                        },
                                    },
                                },
                            },
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_spacing: 4,
                                #[name = "book_label"]
                                gtk::Label { set_label: &model.current_book.name },
                                gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                            },
                        },

                        gtk::MenuButton {
                            set_tooltip_text: Some("Select Chapter"),
                            #[name = "chapter_popover"]
                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                gtk::ScrolledWindow {
                                    set_hscrollbar_policy: gtk::PolicyType::Never,
                                    set_min_content_width: 300,
                                    set_min_content_height: 400,
                                    set_max_content_height: 600,
                                    #[name = "chapter_grid"]
                                    gtk::FlowBox {
                                        set_valign: gtk::Align::Start,
                                        set_max_children_per_line: 5,
                                        set_min_children_per_line: 5,
                                        set_selection_mode: gtk::SelectionMode::None,
                                        set_column_spacing: 6,
                                        set_row_spacing: 6,
                                        set_margin_all: 12,
                                    },
                                },
                            },
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_spacing: 4,
                                #[name = "chapter_label"]
                                gtk::Label { set_label: &format!("Chapter {}", model.current_chapter) },
                                gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                            },
                        },
                    },

                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_hexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        #[local_ref]
                        section_list -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 30,
                        },
                    },
                  },

                  #[name = "dim_scrim"]
                  add_overlay = &gtk::Box {
                      add_css_class: "dim-scrim",
                      set_visible: false,
                      set_can_target: false,
                  },

                  #[name = "overlay_container"]
                  add_overlay = &gtk::Box {
                      set_halign: gtk::Align::End,
                      set_valign: gtk::Align::End,
                      set_margin_all: 25,
                      set_vexpand: false,

                      #[name = "menu_card"]
                      gtk::Box {
                          set_orientation: gtk::Orientation::Vertical,
                          add_css_class: "page-menu-card",
                          set_spacing: 0,
                          set_valign: gtk::Align::End,

                          #[name = "menu_button"]
                          gtk::Button {
                              add_css_class: "circular",
                              add_css_class: "osd",
                              add_css_class: "studypage-menu-trigger-btn",
                              set_has_frame: false,
                              set_width_request: 64,
                              set_height_request: 64,
                              set_halign: gtk::Align::Center,
                              set_valign: gtk::Align::Start,
                              gtk::Image { set_icon_name: Some("page-menu-symbolic"), set_pixel_size: 24 }
                          },

                           // BOTTOM ELEMENT: THE REVEALER
                          #[name = "options_revealer"]
                            gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::SlideDown,
                                set_transition_duration: 250,
                                set_visible: false,
                                // This ensures it grows DOWN from the button without pre-allocating space
                                set_valign: gtk::Align::Start,
                                set_vexpand: false,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,
                                    set_width_request: 350,
                                    // Strict margins: Top is 0 to touch the button

                                    // SECTION: FONT SIZE
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_hexpand: false,
                                        set_spacing: 12,
                                        add_css_class: "osd",
                                        add_css_class: "studypage-menu-section-container",
                                        set_width_request: 350,

                                        gtk::Label{
                                            set_label: "Theme and Font",
                                            add_css_class: "title-1",
                                            set_margin_start: 20,
                                            set_margin_end: 20,
                                            set_margin_top: 20,
                                            set_xalign: 0.0,
                                        },

                                        gtk::Label{
                                            set_label: "Font Size",
                                            add_css_class: "title-5",
                                            set_margin_start: 20,
                                            set_margin_end: 20,
                                            set_xalign: 0.0,
                                        },

                                        gtk::Box{
                                            set_margin_start: 20,
                                            set_margin_end: 20,
                                            set_orientation: gtk::Orientation::Horizontal,

                                            gtk::Image {
                                                set_icon_name: Some("font-letter-symbolic"),
                                            },


                                            gtk::Scale::with_range(gtk::Orientation::Horizontal, 12.0, 32.0, 1.0) {
                                                set_hexpand: true,
                                                add_css_class: "accent",
                                                #[watch]
                                                set_value: model.config.read().unwrap().font_size(),
                                                connect_value_changed[sender] => move |scale| {
                                                    sender.input(
                                                        StudyInput::ToggleDisplay(
                                                            VerseInputMessage::ChangeFontSize(
                                                                scale.value()
                                                            )
                                                        )
                                                    )
                                                }
                                            },

                                            gtk::Image {
                                                set_icon_name: Some("font-letter-symbolic"),
                                                set_pixel_size: 30,
                                            },
                                        },

                                        gtk::Box{
                                            set_margin_start: 20,
                                            set_margin_bottom: 20,
                                            set_margin_end: 20,
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_hexpand: true,

                                            gtk::Label{
                                                set_label: "Fonts",
                                                add_css_class: "title-5",
                                                set_xalign: 0.0,
                                            },

                                             gtk::ScrolledWindow {
                                                set_hscrollbar_policy: gtk::PolicyType::Automatic,
                                                set_vscrollbar_policy: gtk::PolicyType::Never,
                                                set_hexpand: true,
                                                set_height_request: 40,
                                                add_css_class: "font-scroll-container",

                                                #[name="menu_fonts_container"]
                                                gtk::Box {
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_spacing: 10,
                                                    set_margin_all: 5,
                                                    set_hexpand: true,
                                                }
                                            },

                                           gtk::Button {
                                                set_margin_top: 5,
                                                adw::ButtonContent {
                                                    set_icon_name: "emblem-system-symbolic",
                                                    set_label: "Customize",
                                                },

                                                connect_clicked[sender] => move |_|{
                                                    sender.input(StudyInput::OpenCustomizethemePopup)
                                                }
                                            }
                                        }
                                    },

                                    // SECTION: TOGGLES
                                    gtk::Box{
                                        add_css_class: "osd",
                                        add_css_class: "studypage-menu-section-container",
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 12,
                                        set_width_request: 350,

                                        gtk::Label{
                                            set_label: "Book Options",
                                            add_css_class: "title-1",
                                            set_margin_start: 20,
                                            set_margin_end: 20,
                                            set_margin_top: 20,
                                            set_xalign: 0.0,
                                        },


                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_spacing: 10,
                                            set_homogeneous: false,
                                            set_margin_end: 20,
                                            set_margin_start: 20,

                                            gtk::Label{
                                                set_label: "Text",
                                                add_css_class: "title-5",
                                                set_xalign: 0.0,
                                            },

                                             gtk::FlowBox {
                                                set_orientation: gtk::Orientation::Horizontal,
                                                set_hexpand: true,
                                                set_max_children_per_line: 1,
                                                set_min_children_per_line: 1,

                                                 gtk::CheckButton {
                                                    set_label: Some("Words of Christ in Red"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().christ_words_red(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = VerseInputMessage::PutChristWordsInRed(btn.is_active());
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },

                                                gtk::Box{
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_spacing: 30,

                                                    gtk::Label{
                                                        set_label: "Added words"
                                                    },

                                                    gtk::Separator{
                                                        set_orientation: gtk::Orientation::Horizontal,
                                                        add_css_class: "spacer",
                                                        set_hexpand: true
                                                    },

                                                    gtk::DropDown {
                                                        set_halign: gtk::Align::End,
                                                        set_model: Some(&gtk::StringList::new(
                                                            &AddedWordStyle::all().iter().map(
                                                                |style| style.to_string()
                                                            ).collect::<Vec<_>>()
                                                        .iter().map(
                                                            |string| string.as_str()
                                                        ).collect::<Vec<_>>())),
                                                        connect_selected_item_notify[sender] => move |dd| {
                                                            //sender.input(StudyPageInput::UpdateModule(dd.selected()));
                                                        }
                                                    },
                                                }

                                             },

                                              gtk::Label{
                                                set_label: "Lexicons",
                                                add_css_class: "title-5",
                                                set_xalign: 0.0,
                                            },


                                            gtk::FlowBox {
                                                set_orientation: gtk::Orientation::Horizontal,
                                                set_hexpand: true,
                                                set_max_children_per_line: 2,
                                                set_min_children_per_line: 2,


                                                gtk::CheckButton {
                                                    set_label: Some("Strongs"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().show_strongs(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = if btn.is_active() { VerseInputMessage::EnableStrongs }
                                                                else { VerseInputMessage::DisableStrongs };
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },

                                                 gtk::CheckButton {
                                                    set_label: Some("Lemma"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().show_lemma(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = if btn.is_active() { VerseInputMessage::EnableLemma }
                                                                else { VerseInputMessage::DisableLemma };
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },
                                                gtk::CheckButton {
                                                    set_label: Some("Morph"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().show_morphs(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = if btn.is_active() { VerseInputMessage::EnableMorphs }
                                                                else { VerseInputMessage::DisableMorphs };
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },

                                            },
                                        },
                                         gtk::Box {
                                                set_spacing: 8,
                                                set_homogeneous: true,
                                                set_margin_end: 20,
                                                set_margin_start: 20,
                                                set_margin_bottom: 20,

                                                 gtk::CheckButton {
                                                    set_label: Some("Show verse Notes"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().show_notes(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = if btn.is_active() { VerseInputMessage::EnableNotes }
                                                                else { VerseInputMessage::DisableNotes };
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },
                                        }
                                    }
                              }
                          }
                      }
                  }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let engine = init;

        // 1. Setup Sections Factory
        let section_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let sections = FactoryVecDeque::builder()
            .launch(section_list.clone())
            .forward(sender.output_sender(), move |message| match message {
                SectionOutput::Lookup(query) => StudyPageOutput::LookupSelectedStrong(query),
            });

        // 2. Load Saved Session / Resolve Active Module
        let saved_state = BiblePageSettings::load();
        let modules = engine.get_bible_modules();

        let active_module = if let Some(saved_name) = saved_state.last_module {
            modules.iter().find(|m| m.name == saved_name).cloned()
        } else {
            modules.first().cloned()
        };

        // 3. Resolve Book & Chapter
        // We initialize these as options to safely handle the "No Module" case
        let mut active_book = None;
        let mut active_chapter = 1;
        let mut categorized = Vec::new();

        if let Some(ref m) = active_module {
            categorized = engine.get_categorized_books(&m.name);

            active_book = if let Some(saved_b) = saved_state.last_book {
                categorized.iter().find(|b| b.name == saved_b).cloned()
            } else {
                categorized.first().cloned()
            };

            active_chapter = saved_state.last_chapter.unwrap_or(1);
        }

        // 4. Safety Check: If we still have nothing, handle gracefully before unwrap
        if active_module.is_none() {
            eprintln!("No modules found. Please install a SWORD module.");
            // Note: If your struct REQUIRES a module, you might need a dummy fallback
            // or to change your struct fields to Option<SwordModule>
        }

        // 5. Initialize Model
        // Replace your model initialization with this safer version
        let model = if let (Some(m), Some(b)) = (active_module, active_book) {
            BiblePage {
                engine,
                module: m,
                current_book_index: 0,
                current_book: b,
                current_chapter: active_chapter,
                sections,
                config: Arc::new(RwLock::new(PageDisplayConfig::new())),
                customize_theme_popup: None,
                annotations: AnnotationSettings::load_all(),
            }
        } else {
            // Return a 'Safe' or 'Empty' state model here
            // instead of crashing with .expect()
            panic!("XBible cannot start: No valid Bible modules found or module corrupted.");
        };

        let widgets = view_output!();

        // 6. Populate UI Components
        Self::populate_version_grid(&widgets, &modules, sender.clone());
        model.populate_book_grid(&widgets, &categorized, sender.clone());
        model.populate_chapter_grid(&widgets, sender.clone());

        // 7. Setup Overlay Animations
        let motion = gtk::EventControllerMotion::new();
        let options_revealer = widgets.options_revealer.clone();
        let dim_scrim = widgets.dim_scrim.clone();
        let menu_button = widgets.menu_button.clone();

        motion.connect_enter(clone!(
            #[weak]
            options_revealer,
            #[weak]
            dim_scrim,
            #[weak]
            menu_button,
            move |_, _, _| {
                options_revealer.set_visible(true);
                options_revealer.set_reveal_child(true);
                dim_scrim.set_visible(true);
                menu_button.set_opacity(0.0);
                menu_button.set_can_target(false);
            }
        ));

        motion.connect_leave(clone!(
            #[weak]
            options_revealer,
            #[weak]
            dim_scrim,
            #[weak]
            menu_button,
            move |_| {
                options_revealer.set_reveal_child(false);
                dim_scrim.set_visible(false);
                menu_button.set_opacity(1.0);
                menu_button.set_can_target(true);
            }
        ));

        widgets
            .options_revealer
            .connect_child_revealed_notify(|rev| {
                if !rev.reveals_child() && !rev.is_child_revealed() {
                    rev.set_visible(false);
                }
            });

        widgets.overlay_container.add_controller(motion);
        model.populate_fonts_container(&widgets.menu_fonts_container, sender.clone());

        // Load initial reference
        let initial_ref = format!("{} {}", model.current_book.name, model.current_chapter);
        sender.input(StudyInput::LoadReference(initial_ref));

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            StudyInput::LoadReference(refe) => {
                self.load_reference(&refe);
                BiblePageSettings::save(BiblePageState {
                    last_module: Some(self.module.name.clone()),
                    last_book: Some(self.current_book.name.clone()),
                    last_chapter: Some(self.current_chapter),
                });
            }
            StudyInput::SetModule(module) => {
                self.module = module;
                widgets.version_label.set_label(&self.module.name);
                let books = self.engine.get_categorized_books(&self.module.name);
                self.populate_book_grid(widgets, &books, sender.clone());
                if let Some(first_book) = books.first() {
                    sender.input(StudyInput::SetBook(first_book.index));
                }
            }
            StudyInput::SetBook(index) => {
                self.current_book_index = index;
                self.current_chapter = 1;
                let books = self.engine.get_categorized_books(&self.module.name);
                if let Some(book) = books.iter().find(|b| b.index == index) {
                    self.current_book = book.clone();
                    widgets.book_label.set_label(&book.name);
                }
                widgets.chapter_label.set_label("Chapter 1");
                self.populate_chapter_grid(widgets, sender.clone());
                let book_name = self.engine.get_book_name(&self.module.name, index);
                sender.input(StudyInput::LoadReference(format!("{} 1", book_name)));
            }
            StudyInput::SetChapter(chapter) => {
                self.current_chapter = chapter;
                widgets
                    .chapter_label
                    .set_label(&format!("Chapter {}", chapter));
                let book_name = self
                    .engine
                    .get_book_name(&self.module.name, self.current_book_index);
                sender.input(StudyInput::LoadReference(format!(
                    "{} {}",
                    book_name, chapter
                )));
            }
            StudyInput::OpenCustomizethemePopup => {
                let controller = CustomizeThemePopup::builder()
                    .launch((self.config.clone(), self.engine.clone()))
                    .forward(sender.input_sender(), |msg| match msg {
                        CustomizeThemeOutput::Close => StudyInput::CloseCustomizethemePopup,
                        CustomizeThemeOutput::SaveConfig(config) => StudyInput::SetConfig(config),
                    });
                controller.widget().present();
                self.customize_theme_popup = Some(controller);
            }
            StudyInput::CloseCustomizethemePopup => {
                if let Some(c) = self.customize_theme_popup.take() {
                    c.widget().close();
                }
            }
            StudyInput::ToggleDisplay(msg) => {
                self.config.write().unwrap().apply_message(&msg);
                for i in 0..self.sections.len() {
                    self.sections
                        .send(i, SectionInput::ToggleDisplay(msg.clone()));
                }
                let class = self.make_css_preview_clss(self.config.read().unwrap().theme());
                let _ = sender.output(StudyPageOutput::ChangeTheme);
                widgets
                    .page_overlay
                    .set_css_classes(&["page-overlay", &class]);
            }
            StudyInput::SetConfig(config) => {
                sender.input(StudyInput::ToggleDisplay(
                    VerseInputMessage::UpdateDisplayConf(config),
                ));
            }
        }
    }
}

impl BiblePage {
    fn populate_version_grid(
        widgets: &BiblePageWidgets,
        modules: &[SwordModule],
        sender: ComponentSender<Self>,
    ) {
        while let Some(child) = widgets.bible_grid.first_child() {
            widgets.bible_grid.remove(&child);
        }

        let mut grouped: std::collections::BTreeMap<String, Vec<SwordModule>> =
            std::collections::BTreeMap::new();
        for module in modules {
            grouped
                .entry(module.language.clone())
                .or_default()
                .push(module.clone());
        }

        for (lang, lang_modules) in grouped {
            // --- Language Header ---
            let header_label = gtk::Label::builder()
                .halign(gtk::Align::Start)
                .margin_top(20) // More space above for "High-End" feel
                .margin_bottom(12) // Space below the header
                .margin_start(16) // Inline margin
                .build();

            header_label.set_markup(&format!(
                "<span size='small' weight='heavy' alpha='60%' letter_spacing='1200'>{}</span>",
                lang.to_uppercase()
            ));

            widgets.bible_grid.append(&header_label);

            // --- The Grid (Using WrapBox with constant width logic) ---
            let wrap_box = adw::WrapBox::builder()
            .orientation(gtk::Orientation::Horizontal)
                .line_spacing(5) // Space between columns
                .child_spacing(5) // Space between rows
                .margin_start(10)
                .margin_end(10)
                .margin_bottom(20)
                .build();

            for module in lang_modules {
                let m = module.clone();
                // Use the updated tile with constant width
                let tile = Self::create_bible_tile(&module.name, &module.language);
                let s = sender.clone();
                let pop = widgets.version_popover.clone();

                tile.connect_clicked(move |_| {
                    s.input(StudyInput::SetModule(m.clone()));
                    pop.popdown();
                });

                wrap_box.append(&tile);
            }

            widgets.bible_grid.append(&wrap_box);

            // Subtle Separator
            let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
            sep.set_opacity(0.1);
            sep.set_margin_start(16);
            sep.set_margin_end(16);
            widgets.bible_grid.append(&sep);
        }
    }

    fn populate_book_grid(
        &self,
        widgets: &BiblePageWidgets,
        books: &[CategorizedBook],
        sender: ComponentSender<Self>,
    ) {
        while let Some(child) = widgets.ot_grid.first_child() {
            widgets.ot_grid.remove(&child);
        }
        while let Some(child) = widgets.nt_grid.first_child() {
            widgets.nt_grid.remove(&child);
        }
        let mut has_ot = false;
        let mut has_nt = false;
        for book in books {
            let btn = Self::create_book_tile(&book.name);
            let idx = book.index;
            let s = sender.clone();
            let pop = widgets.book_popover.clone();
            btn.connect_clicked(move |_| {
                s.input(StudyInput::SetBook(idx));
                pop.popdown();
            });
            match book.testament {
                Testament::Old => {
                    widgets.ot_grid.append(&btn);
                    has_ot = true;
                }
                Testament::New => {
                    widgets.nt_grid.append(&btn);
                    has_nt = true;
                }
            }
        }
        widgets.ot_container.set_visible(has_ot);
        widgets.nt_container.set_visible(has_nt);
    }

    fn populate_chapter_grid(&self, widgets: &BiblePageWidgets, sender: ComponentSender<Self>) {
        while let Some(child) = widgets.chapter_grid.first_child() {
            widgets.chapter_grid.remove(&child);
        }
        let count = self
            .engine
            .get_chapter_count(&self.module.name, self.current_book_index);
        for i in 1..=count {
            let btn = gtk::Button::builder()
                .label(&i.to_string())
                .css_classes(vec!["card", "chapter-tile"])
                .build();
            let s = sender.clone();
            let pop = widgets.chapter_popover.clone();
            btn.connect_clicked(move |_| {
                s.input(StudyInput::SetChapter(i as i32));
                pop.popdown();
            });
            widgets.chapter_grid.append(&btn);
        }
    }

    fn create_bible_tile(name: &str, _lang: &str) -> gtk::Button {
        let button = gtk::Button::builder()
            // Constant Dimensions
            .width_request(120)
            .height_request(48)
            .css_classes(vec!["card", "flat"])
            .build();

        // Centered, bold text for the module name
        let label = gtk::Label::builder().build();

        label.set_markup(&format!(
            "<span weight='bold' size='medium' font_features='tnum'>{}</span>",
            name
        ));

        button.set_child(Some(&label));

        // Inline margin to prevent buttons from touching
        button.set_margin_all(2);

        button
    }

    fn create_book_tile(name: &str) -> gtk::Button {
        gtk::Button::builder()
            .label(name)
            .css_classes(vec!["card", "book-tile"])
            .width_request(85)
            .height_request(40)
            .build()
    }
}
