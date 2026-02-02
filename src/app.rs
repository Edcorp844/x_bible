use adw::prelude::*;
use relm4::prelude::*;
use std::sync::Arc;

use crate::features::{
    bible::components::page::model::{StudyInput, StudyPage},
    core::{sword_engine::SwordEngine, sword_module::SwordModule},
};

pub struct AppModel {
    engine: Arc<SwordEngine>,
    study_page: Controller<StudyPage>,
    modules: gtk::StringList,
    bible_data: Vec<SwordModule>,
    has_modules: bool,
}

#[derive(Debug)]
pub enum AppInput {
    Search(String),
    ModuleChanged(String),
    RefreshModules, // Useful for when a user finishes an install
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_width: 1000,
            set_default_height: 800,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                // 1. Header is only visible if we have something to read
                adw::HeaderBar {
                    #[watch]
                    set_visible: model.has_modules,

                    #[wrap(Some)]
                    set_title_widget = &gtk::Box {
                        set_spacing: 12,
                        set_halign: gtk::Align::Center,

                        gtk::Entry {
                            set_placeholder_text: Some("Enter Reference (e.g. John 3:16)"),
                            set_max_width_chars: 30,
                            connect_activate[sender] => move |entry| {
                                sender.input(AppInput::Search(entry.text().to_string()));
                            }
                        },

                        gtk::DropDown {
                            set_model: Some(&model.modules),
                            #[watch]
                            set_selected: 0,

                            connect_selected_item_notify[sender] => move |dd| {
                                if let Some(item) = dd.selected_item() {
                                    if let Some(obj) = item.downcast_ref::<gtk::StringObject>() {
                                        sender.input(AppInput::ModuleChanged(obj.string().to_string()));
                                    }
                                }
                            }
                        }
                    }
                },

                // 2. Main Content Stack
                gtk::Stack {
                    set_vexpand: true,
                    // Transitions smoothly between screens
                    set_transition_type: gtk::StackTransitionType::Crossfade,

                    #[watch]
                    set_visible_child_name: if model.has_modules { "bible_view" } else { "install_view" },

                    // VIEW: Installation Screen
                    add_named[Some("install_view")] = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 24,

                        adw::HeaderBar{},

                        adw::StatusPage {
                            set_title: "Welcome to XBible",
                            set_description: Some("No Bible modules found in your AppData folder. Please install one to begin."),
                            set_icon_name: Some("library-symbolic"),
                        },

                        gtk::Button {
                            set_label: "Download Bible Modules",
                            set_halign: gtk::Align::Center,
                            add_css_class: "suggested-action",
                            add_css_class: "pill",
                            set_margin_bottom: 100,
                            // connect_clicked => sender.input(AppInput::OpenInstaller),
                        }
                    },

                    // VIEW: Bible Reader
                    add_named[Some("bible_view")] = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,

                        #[local_ref]
                        study_page_widget -> gtk::Box {
                            set_vexpand: true,
                            set_hexpand: true,
                        },
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize the engine and check for content
        let engine = SwordEngine::new();
        let bible_data = engine.get_modules(); // Now returns Vec<SwordModule>
        let has_modules = !bible_data.is_empty();

        // Populate the DropDown list
        let modules_ui = gtk::StringList::new(&[]);
        for module in &bible_data {
            modules_ui.append(&module.name);
        }

        // Get the first Bible or a dummy string
        let first_bible = bible_data
            .first()
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "".to_string());

        let engine_arc = Arc::new(engine);

        // Start the StudyPage (it will be hidden if has_modules is false)
        let study_page = StudyPage::builder()
            .launch((engine_arc.mgr, first_bible))
            .detach();

        let model = AppModel {
            engine: engine_arc,
            study_page,
            modules: modules_ui,
            bible_data,
            has_modules,
        };

        let study_page_widget = model.study_page.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::Search(reference) => {
                if self.has_modules {
                    self.study_page.emit(StudyInput::LoadReference(reference));
                }
            }
            AppInput::ModuleChanged(module_name) => {
                self.study_page.emit(StudyInput::SetModule(module_name));
            }
            AppInput::RefreshModules => {
                // Logic to re-scan the AppData folder after a download
                self.bible_data = self.engine.get_modules();
                self.has_modules = !self.bible_data.is_empty();

                // Rebuild the UI list
                self.modules = gtk::StringList::new(&[]);
                for m in &self.bible_data {
                    self.modules.append(&m.name);
                }
            }
        }
    }
}
