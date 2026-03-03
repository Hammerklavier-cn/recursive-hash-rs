use adw::{Application, ApplicationWindow, prelude::AdwApplicationWindowExt};
use gtk::{
    Button,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    prelude::{BoxExt, ButtonExt, GtkWindowExt},
};

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
    tool_bar_view.add_top_bar(&header);

    // Create a button with label and margins
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(|button| {
        // Set the label to "Hello World!" after the button has been clicked on
        button.set_label("Hello World!");
    });
    tool_bar_view.set_content(Some(&button));

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .build();

    window.set_content(Some(&tool_bar_view));

    // Present window
    window.present();
}
