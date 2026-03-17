use adw::{Application, ApplicationWindow, prelude::AdwApplicationWindowExt};
use gtk::{
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    glib::object::ObjectExt,
    prelude::{BoxExt, GtkWindowExt},
};

pub mod file_select;
pub mod path_line_bin;
pub mod path_object;

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

    // 1. Area for adding file/folder paths.
    let path_add_area =
        file_select::FileSelectArea::new("Select Files/Folders", window.downgrade());
    content_box.append(&path_add_area);

    // 2. Area for excluding file/folder paths.
    let path_exclusion_area =
        file_select::FileSelectArea::new("Exclude Files/Folders", window.downgrade());
    content_box.append(&path_exclusion_area);

    window.set_content(Some(&tool_bar_view));

    // Present window
    window.present();
}
