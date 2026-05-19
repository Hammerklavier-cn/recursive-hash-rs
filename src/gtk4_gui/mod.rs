use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use adw::{Application, ApplicationWindow, BreakpointCondition, prelude::AdwApplicationWindowExt};
use gtk::glib::{self, VariantTy};
use gtk::prelude::{EditableExt, ToValue, WidgetExt};
use gtk::{
    gio::{
        self,
        glib::object::ObjectExt,
        prelude::{ActionMapExt, ApplicationExt, ApplicationExtManual},
    },
    prelude::{BoxExt, GtkWindowExt},
};

use crate::finder::normalize_path;
use crate::gtk4_gui::hash_diff::HashDiffArea;
use crate::gtk4_gui::hash_result::HashResultArea;
use crate::hasher::{Hasher, Md5Hasher, Sha1Hasher, Sha256Hasher, Sha384Hasher, Sha512Hasher};

mod file_select;
mod hash_diff;
mod hash_result;

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
        .title("Recursive Hash")
        .default_height(600)
        .default_width(800)
        .height_request(704)
        .build();

    // Create a AdwToolBarView, which is an overall container
    let tool_bar_view = adw::ToolbarView::builder()
        .top_bar_style(adw::ToolbarStyle::Flat)
        .vexpand(true)
        .hexpand(true)
        .build();
    window.set_content(Some(&tool_bar_view));

    // Create the ViewStack first, as both ViewSwitcher and ViewSwitcherBar need it
    let view_stack = adw::ViewStack::new();

    // Create the AdwWindowTitle, placed at the leftmost of the header bar
    let title = adw::WindowTitle::builder()
        .title("Recursive Hash")
        .subtitle("Generate and check file hash recursively")
        .build();

    // Create a ViewSwitcher as the centered title widget in the header bar
    let view_switcher = adw::ViewSwitcher::builder()
        .stack(&view_stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    // Build the header bar: AdwWindowTitle on the left, ViewSwitcher centered
    let header = adw::HeaderBar::builder()
        .title_widget(&view_switcher)
        .build();
    header.pack_start(&title);

    // Add the header bar to the toolbar view
    tool_bar_view.add_top_bar(&header);

    // ViewSwitcherBar at the bottom for narrow/adaptive mode.
    // reveal=false lets AdwToolbarView automatically show/hide it based on width.
    let switcher_bar = adw::ViewSwitcherBar::builder()
        .stack(&view_stack)
        .reveal(false)
        .build();

    tool_bar_view.set_content(Some(&view_stack));
    tool_bar_view.add_bottom_bar(&switcher_bar);

    // Create a content box, which is a vertical box for all contents
    let check_content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    // tool_bar_view.set_content(Some(&content_box));
    view_stack.add_titled_with_icon(
        &check_content_box,
        Some("content 1"),
        "Check",
        "text-editor-symbolic",
    );

    let input_layout = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .spacing(6)
        .build();
    check_content_box.append(&input_layout);

    // Create a Breakpoint for adaptive view switching.
    // When the window is narrow (max-width: 500sp):
    //   - Reveal the ViewSwitcherBar at the bottom
    //   - Remove the ViewSwitcher from the header bar (replace with empty Bin)
    //   - Stack the input_layout vertically so FileSelectAreas don't overflow
    //   - Set window height-request to 812 and width-request to 373, as breakpoint
    //     breaks AdwWindow's default size limit.
    // When the window is wide again, all properties revert to their original values
    // automatically.
    {
        let breakpoint_condition = BreakpointCondition::parse("max-width: 710sp")
            .expect("Failed to parse breakpoint condition");
        let breakpoint = adw::Breakpoint::new(breakpoint_condition);
        breakpoint.add_setter(&switcher_bar, "reveal", Some(&true.to_value()));
        // None leads to a default title, which is unwanted. We want it blank.
        breakpoint.add_setter(&header, "title-widget", Some(&adw::Bin::new().to_value()));

        // breakpoint.connect_unapply(f)
        // Stack the two FileSelectAreas vertically instead of horizontally
        breakpoint.add_setter(
            &input_layout,
            "orientation",
            Some(&gtk::Orientation::Vertical.to_value()),
        );
        breakpoint.add_setter(&window, "height-request", Some(&900.to_value()));
        breakpoint.add_setter(&window, "width-request", Some(&500.to_value()));
        window.add_breakpoint(breakpoint);
    }

    // 1. Area for adding file/folder paths.
    let path_add_area =
        file_select::FileSelectArea::new("Select Files/Folders", window.downgrade());
    input_layout.append(&path_add_area);

    // 2. Area for excluding file/folder paths.
    let path_exclusion_area =
        file_select::FileSelectArea::new("Exclude Files/Folders", window.downgrade());
    input_layout.append(&path_exclusion_area);

    // 3. Area for specifying output file and hash algorithm.
    let options_layout = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    check_content_box.append(&options_layout);
    // Output file entry
    let output_file_entry = gtk::Entry::builder()
        .hexpand(true)
        .text("./checklist.sha256") // Default output file name for debugging.
        .editable(false) // Shouldn't be manually edited.
        .placeholder_text("placeholder_text")
        .build();
    options_layout.append(&output_file_entry);
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
        .width_request(100)
        .build();
    options_layout.append(&hasher_splitbutton);

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

    // 4. Area for displaying all files (found or expected to be found) and hashes.
    let hash_result_area = HashResultArea::new();
    check_content_box.append(&hash_result_area);
    hash_result_area.set_margin_top(12);
    hash_result_area.set_margin_bottom(12);
    hash_result_area.set_margin_start(12);
    hash_result_area.set_margin_end(12);

    // Connect paths_updated callback to execute after file selection completes
    path_add_area.connect_paths_updated(glib::clone!(
        #[weak]
        path_add_area,
        #[weak]
        path_exclusion_area,
        #[weak]
        hash_result_area,
        move || {
            log::debug!("Executing path_add_area path finding process after paths updated...");
            // first clear the result area before adding new results.
            hash_result_area.remove_all();

            let selected_paths = path_add_area.paths();
            let excluded_paths = path_exclusion_area.paths();
            let paths_to_hash =
                crate::finder::find_files(&selected_paths, &excluded_paths).unwrap(); // TODO: pop up a warning window.
            for path in paths_to_hash {
                hash_result_area.add_result(path, String::from(""));
            }
        }
    ));
    // also for path_exclusion_area
    path_exclusion_area.connect_paths_updated(glib::clone!(
        #[weak]
        path_add_area,
        #[weak]
        path_exclusion_area,
        #[weak]
        hash_result_area,
        move || {
            log::debug!(
                "Executing path_exclusion_area path finding process after paths updated..."
            );
            // first clear the result area before adding new results.
            hash_result_area.remove_all();

            let selected_paths = path_add_area.paths();
            let excluded_paths = path_exclusion_area.paths();
            let paths_to_hash =
                crate::finder::find_files(&selected_paths, &excluded_paths).unwrap(); // TODO: pop up a warning window.
            for path in paths_to_hash {
                hash_result_area.add_result(path, String::from(""));
            }
        }
    ));
    // Update hash results when `hasher_splitbutton` is clicked.
    // This process can take a long time, so we run it in a
    // separate thread while using a channel to communicate results.
    let (sender, receiver) = async_channel::bounded::<Option<(PathBuf, String)>>(1);
    hasher_splitbutton.connect_clicked(glib::clone!(
        #[weak]
        path_add_area,
        #[weak]
        path_exclusion_area,
        #[weak]
        output_file_entry,
        move |btn| {
            log::debug!("hasher_splitbutton clicked. Hash all files and update the results.");

            let hash_algorithm = btn.label().unwrap().to_string();

            let selected_paths = path_add_area.paths();
            let excluded_paths = path_exclusion_area.paths();
            let save_path = output_file_entry.text().to_string();

            let files_to_hash =
                crate::finder::find_files(&selected_paths, &excluded_paths).unwrap();

            if files_to_hash.len() != 0 {
                gio::spawn_blocking(glib::clone!(
                    #[strong]
                    sender,
                    move || {
                        match File::create(&save_path) {
                            Ok(mut save_file) => {
                                log::debug!("Hashing results will be saved in {save_path}");

                                // generate checksum
                                let mut i = 0;
                                for read_path in files_to_hash {
                                    i += 1;
                                    let mut read_file =
                                        File::open(&read_path).expect("failed to read {read_path}");
                                    let checksum = match hash_algorithm.to_lowercase().as_str() {
                                        "md5" | "md-5" => {
                                            log::debug!("Hashing {} with MD5", read_path.display());
                                            Md5Hasher.get_hash(&mut read_file)
                                        }
                                        "sha1" | "sha-1" => {
                                            log::debug!(
                                                "Hashing {} with SHA-1",
                                                read_path.display()
                                            );
                                            Sha1Hasher.get_hash(&mut read_file)
                                        }
                                        "sha256" | "sha-256" => {
                                            log::debug!(
                                                "Hashing {} with SHA-256",
                                                read_path.display()
                                            );
                                            Sha256Hasher.get_hash(&mut read_file)
                                        }
                                        "sha384" | "sha-384" => {
                                            log::debug!(
                                                "Hashing {} with SHA-384",
                                                read_path.display()
                                            );
                                            Sha384Hasher.get_hash(&mut read_file)
                                        }
                                        "sha512" | "sha-512" => {
                                            log::debug!(
                                                "Hashing {} with SHA-512",
                                                read_path.display()
                                            );
                                            Sha512Hasher.get_hash(&mut read_file)
                                        }
                                        e => {
                                            log::error!("SplitButton possesses wrong label: {}", e);
                                            std::process::exit(1);
                                        }
                                    };
                                    // Send the result to the main thread
                                    sender
                                        .send_blocking(Some((read_path.clone(), checksum.clone())))
                                        .unwrap();
                                    // write checksum to save file
                                    save_file
                                        .write_all(
                                            format!(
                                                "{} *{}\n",
                                                checksum,
                                                normalize_path(
                                                    read_path,
                                                    Path::new(&save_path).parent().unwrap()
                                                )
                                                .display()
                                            )
                                            .as_bytes(),
                                        )
                                        .expect("failed to write to {save_path}");
                                }
                                log::info!("Hashed {i} files in total");
                                sender.send_blocking(None).unwrap();
                            }
                            Err(_) => {
                                log::error!("Cannot save hashing results to {save_path}");
                            }
                        }
                    }
                ));
            } else {
                log::debug!("No files need hashing.");
            }
        }
    ));

    glib::spawn_future_local(glib::clone!(
        #[weak]
        hash_result_area,
        async move {
            let buf_max = 25;
            let mut buf = HashMap::<PathBuf, String>::new();
            let mut i = 0;
            while let Ok(opt) = receiver.recv().await {
                match opt {
                    Some((path, hash)) => {
                        i += 1;
                        log::debug!("Received hash result for {:?}", path);
                        buf.insert(path, hash);
                        if buf.len() > buf_max {
                            hash_result_area.batch_update_results(&buf);
                            buf.clear();
                        }
                    }
                    None => {
                        log::debug!("Received {} hash results in total.", i);
                        i = 0;
                        hash_result_area.batch_update_results(&buf);
                        buf.clear();
                    }
                }
            }
            if !buf.is_empty() {
                hash_result_area.batch_update_results(&buf);
            }
        }
    ));

    let verify_content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let hash_diff_view = HashDiffArea::new();
    verify_content_box.append(&hash_diff_view);

    view_stack.add_titled_with_icon(
        &verify_content_box,
        Some("content 2"),
        "Verify",
        "checkbox-checked-symbolic",
    );

    // Present window
    window.present();
}
