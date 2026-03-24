use adw::prelude::*;
use relm4::prelude::*;

pub(crate) struct AppMenu {}

impl AppMenu {
    pub(crate) fn register() {
        let app = relm4::main_application();

        // --- 1. About Action ---
        let about_action = gtk::gio::SimpleAction::new("about", None);
        about_action.connect_activate(move |_, _| {
            Self::show_about_window();
        });
        app.add_action(&about_action);

        // --- 2. New Window Action (Ctrl+N) ---
        let new_window_action = gtk::gio::SimpleAction::new("new_window", None);
        new_window_action.connect_activate(move |_, _| {
            // Logic to launch a new instance of your main Relm4 window
            //relm4::main_application().activate();
            println!("New window requested");
        });
        app.add_action(&new_window_action);
        app.set_accels_for_action("app.new_window", &["<Primary>n"]);

        // 3. Shortcuts Action (Ctrl + ?)
        let shortcuts_action = gtk::gio::SimpleAction::new("shortcuts", None);
        shortcuts_action.connect_activate(move |_, _| {
            Self::show_shortcuts_window();
        });
        app.add_action(&shortcuts_action);

        // Primary+question is the standard "Ctrl + ?" shortcut
        app.set_accels_for_action("app.shortcuts", &["<Primary>question"]);

        // --- 4. Quit Action (Ctrl+Q) ---
        let quit_action = gtk::gio::SimpleAction::new("quit", None);
        let app_clone = app.clone();
        quit_action.connect_activate(move |_, _| {
            // Now you can call quit directly on the cloned app
            app_clone.quit();
        });

        app.add_action(&quit_action);
        app.set_accels_for_action("app.quit", &["<Primary>q"]);
    }

    pub(crate) fn show_about_window() {
        if let Some(active_window) = relm4::main_application().active_window() {
            let about = adw::AboutDialog::builder()
                .application_name("xBible")
                .application_icon("com.example.xbible")
                .version("1.0.0")
                .developer_name("Edson Frost Twinamatsiko")
                .website("https://github.com/your-repo/xbible")
                .issue_url("https://github.com/your-repo/xbible/issues")
                .copyright("© 2026 Edson Frost Twinamatsiko")
                .license_type(gtk::License::Gpl30)
                .developers(vec!["Frost Edson"])
                .artists(vec!["Frost Edson"])
                .build();

            about.add_acknowledgement_section(
                Some("Data and Engines"),
                &["The SWORD Project (Crosswire Bible Society)"],
            );

            about.present(Some(&active_window));
        }
    }

    pub(crate) fn show_shortcuts_window() {
        if let Some(active_window) = relm4::main_application().active_window() {
            let shortcuts_window = adw::ShortcutsDialog::builder()
                .title("Keyboard Shortcuts")
                .width_request(600)
                .height_request(500)
                .build();

            // --- SECTION: Window ---
            let window_section = adw::ShortcutsSection::new(Some("Window"));

            let new_win = adw::ShortcutsItem::new("New Window", "<Primary>n");
            new_win.set_subtitle("Opens a new window");
            window_section.add(new_win);

            let quit = adw::ShortcutsItem::new("Quit", "<Primary>q");
            quit.set_subtitle("Close the application");
            window_section.add(quit);

            // --- SECTION: Application ---
            let application_section = adw::ShortcutsSection::new(Some("Application"));

            let prefs = adw::ShortcutsItem::new("Preferences", "<Primary>comma");
            prefs.set_subtitle("Configure application preferences");
            application_section.add(prefs);

            let shorts = adw::ShortcutsItem::new("Shortcuts", "<Primary>question");
            shorts.set_subtitle("Shows shortcuts window");
            application_section.add(shorts);

            let help = adw::ShortcutsItem::new("Help", "F1");
            help.set_subtitle("Opens up the help notes");
            application_section.add(help);

            // --- SECTION: Navigation ---
            let nav_section = adw::ShortcutsSection::new(Some("Navigation"));

            let search = adw::ShortcutsItem::new("Search Scriptures", "<Primary>f");
            search.set_subtitle("Find verses or keywords");
            nav_section.add(search);

            // Add sections to the dialog
            shortcuts_window.add(window_section);
            shortcuts_window.add(application_section);
            shortcuts_window.add(nav_section);

            shortcuts_window.present(Some(&active_window));
        }
    }
}
