use adw::{Application, ApplicationWindow, prelude::AdwApplicationWindowExt};
use gtk::glib::VariantTy;
use gtk::prelude::WidgetExt;
use gtk::{
    gio::{
        self,
        glib::object::ObjectExt,
        prelude::{ActionMapExt, ApplicationExt, ApplicationExtManual},
    },
    prelude::{BoxExt, GtkWindowExt},
};

pub mod file_select;

static APP_ID: &str = "com.hammerklavier.recursive-hash";

pub fn run() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run_with_args(&[] as &[&str]);
}

fn build_ui(app: &Application) {
    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .build();

    // Create a AdwToolBarView, which is an overall container
    let tool_bar_view = adw::ToolbarView::builder()
        .top_bar_style(adw::ToolbarStyle::Flat)
        .build();
    window.set_content(Some(&tool_bar_view));

    // Create a AdwHeaderBar
    let title = adw::WindowTitle::builder()
        .title("Recursive Hash")
        .subtitle("Generate and check file hash recursively")
        .build();
    let header = adw::HeaderBar::builder().title_widget(&title).build();
    // Add the header bar to the toolbar view
    tool_bar_view.add_top_bar(&header);

    // Create a content box, which is a vertical box for all contents
    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    tool_bar_view.set_content(Some(&content_box));

    // // This is never triggered.
    // content_box.connect_width_request_notify(|widget| {
    //     log::info!("Connect width notify");
    //     let width = widget.width();
    //     if width > 200 {
    //         widget.set_orientation(gtk::Orientation::Horizontal);
    //     } else {
    //         widget.set_orientation(gtk::Orientation::Vertical);
    //     }
    // });

    let input_layout = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    content_box.append(&input_layout);

    // 1. Area for adding file/folder paths.
    let path_add_area =
        file_select::FileSelectArea::new("Select Files/Folders", window.downgrade());
    input_layout.append(&path_add_area);

    // 2. Area for excluding file/folder paths.
    let path_exclusion_area =
        file_select::FileSelectArea::new("Exclude Files/Folders", window.downgrade());
    input_layout.append(&path_exclusion_area);

    // 3. Area for specifying output file and hash algorithm.
    let output_layout = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    content_box.append(&output_layout);
    // Output file entry
    let output_file_entry = gtk::Entry::builder()
        .hexpand(true)
        .placeholder_text("placeholder_text")
        .build();
    output_layout.append(&output_file_entry);
    // Hash Algorithm SplitButton, used for selecting the hash algorithm and triggering
    // hash calculation
    let menu = gio::Menu::new();
    menu.append(Some("MD5"), Some("app.hash_algorithm::MD5"));
    menu.append(Some("SHA-1"), Some("app.hash_algorithm::SHA-1"));
    menu.append(Some("SHA-256"), Some("app.hash_algorithm::SHA-256"));
    menu.append(Some("SHA-384"), Some("app.hash_algorithm::SHA-384"));
    menu.append(Some("SHA-512"), Some("app.hash_algorithm::SHA-512"));
    let hasher_popovermenu = gtk::PopoverMenu::from_model(Some(&menu));
    let hasher_splitbutton = adw::SplitButton::builder()
        .name("hasher_splitbutton")
        .label("SHA-256")
        .popover(&hasher_popovermenu)
        .vexpand(false)
        .margin_start(6)
        .width_request(100)
        .build();

    // Create hash_algorithm action with string parameter
    let hash_algorithm_action = gio::SimpleAction::new("hash_algorithm", Some(VariantTy::STRING));
    hash_algorithm_action.connect_activate({
        let hasher_splitbutton_weak = hasher_splitbutton.downgrade();
        move |_action, parameter| {
            if let Some(algorithm) = parameter.and_then(|p| p.str()) {
                log::debug!("change hash_algorithm to {}", algorithm);
                if let Some(button) = hasher_splitbutton_weak.upgrade() {
                    button.set_label(algorithm);
                }
            }
        }
    });
    app.add_action(&hash_algorithm_action);

    output_layout.append(&hasher_splitbutton);

    // 4. Area for displaying all files (found or expected to be found) and hashes.
    //

    // Present window
    window.present();
}
