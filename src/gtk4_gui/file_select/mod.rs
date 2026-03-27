use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib;

pub mod path_line_bin;
pub mod path_object;

glib::wrapper! {
    pub struct FileSelectArea(ObjectSubclass<imp::FileSelectArea>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl FileSelectArea {
    pub fn new(title: impl AsRef<str>, parent: glib::WeakRef<adw::ApplicationWindow>) -> Self {
        let obj = glib::Object::new::<Self>();
        // obj.set_vexpand(true);

        let imp = obj.imp();

        imp.title_label.set_label(title.as_ref());
        // Set the parent after constructed() - callbacks captured Rc, so they will see this update
        *imp.parent.borrow_mut() = parent;

        obj
    }
}

mod imp {
    use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::OnceLock};

    use adw::{
        prelude::BinExt,
        subclass::{
            bin::BinImpl,
            prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt},
        },
    };
    use gtk::{
        ListItem,
        gio::{
            self,
            prelude::{FileExt, ListModelExtManual},
        },
        glib::{
            self, WeakRef,
            object::{Cast, CastNone, ObjectExt},
        },
        prelude::{BoxExt, ButtonExt, ListItemExt, SelectionModelExt, WidgetExt},
        subclass::widget::WidgetImpl,
    };

    use super::{path_line_bin::PathLineBin, path_object::PathObject};

    #[derive(Default)]
    pub struct FileSelectArea {
        // Using Rc<RefCell<...>> allows callbacks to capture Rc and see updates made after constructed()
        pub(super) parent: Rc<RefCell<WeakRef<adw::ApplicationWindow>>>,

        pub(super) title_label: gtk::Label,

        pub(super) path_list_view: gtk::ListView,
        pub(super) path_list_store: OnceLock<gtk::gio::ListStore>,

        pub(super) add_file_button: gtk::Button,
        pub(super) add_folder_button: gtk::Button,
        pub(super) remove_path_button: gtk::Button,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileSelectArea {
        const NAME: &'static str = "FileSelectArea";
        type Type = super::FileSelectArea;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for FileSelectArea {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Vertical layout
            let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
            obj.set_child(Some(&layout));

            // Title, which looks like
            // ------------`title`------------
            let title_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);

            // Left separator line
            let left_separator = gtk::Separator::new(gtk::Orientation::Horizontal);
            left_separator.set_hexpand(true);
            left_separator.set_valign(gtk::Align::Center);

            // Right separator line
            let right_separator = gtk::Separator::new(gtk::Orientation::Horizontal);
            right_separator.set_hexpand(true);
            right_separator.set_valign(gtk::Align::Center);

            self.title_label.set_hexpand(false);

            title_layout.append(&left_separator);
            title_layout.append(&self.title_label);
            title_layout.append(&right_separator);

            layout.append(&title_layout);

            // List all selected folders and paths in this widget
            //
            // 1. Initialize `self.path_list_store`.
            //  This is a GObject list of `PathObject`, which is also
            //  a GObject.
            let list_store = self
                .path_list_store
                .get_or_init(|| gtk::gio::ListStore::new::<PathObject>());

            // 2. Assign selection model for the ListStore. In this case, we allow
            //  multiple selections of PathObjects.
            let selection_model = gtk::MultiSelection::new(Some(list_store.clone()));

            // 2. Create a `gtk::SignalListItemFactory` for displaying `PathObject`
            //  in the list view. The factory defines how `PathObject`s in
            //  the `list_store` are displayed in the `GtkListView`
            let factory = gtk::SignalListItemFactory::new();

            // Bind the selection model and factory to the listview
            self.path_list_view.set_model(Some(&selection_model));
            self.path_list_view.set_factory(Some(&factory));

            // 3.1 Bind closure for setting up UI of each line
            factory.connect_setup(move |_, object| {
                let path_line_bin = PathLineBin::new(None, "");
                // let path_line_bin = gtk::Label::new(None);
                object
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&path_line_bin));
            });
            // 3.2 Bind closure for displaying data to UI
            factory.connect_bind(move |_, object| {
                let list_item = object.downcast_ref::<ListItem>().unwrap();

                let path_line_bin = list_item.child().and_downcast::<PathLineBin>().unwrap();
                // let path_line_bin = list_item.child().and_downcast::<gtk::Label>().unwrap();
                let path_object = list_item.item().and_downcast::<PathObject>().unwrap();

                path_line_bin.update_from_path_object(&path_object);
                // path_line_bin.set_label(&path_object.path().display().to_string());
            });

            // Wrap ListView in ScrolledWindow for scrollbar support
            // The list view has a flexible height between 150 and 350 pixels depending
            // on the number of items in the list. However it seems that the area will
            // not expand beyong 150 pixels automatically. Check the gtk4 documentation
            // later.
            let scrolled_window = gtk::ScrolledWindow::builder()
                .min_content_width(300)
                .min_content_height(150)
                .max_content_height(350)
                .child(&self.path_list_view)
                .build();

            layout.append(&scrolled_window);
            // layout.append(&self.path_list_view);

            // List three buttons for adding files, folders, and removing paths
            let button_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            self.add_file_button.set_label("Add File");
            self.add_folder_button.set_label("Add Folder");
            self.remove_path_button.set_label("Remove Path");
            self.add_file_button.set_hexpand(true);
            self.add_folder_button.set_hexpand(true);
            self.remove_path_button.set_hexpand(true);
            button_layout.append(&self.add_file_button);
            button_layout.append(&self.add_folder_button);
            button_layout.append(&self.remove_path_button);
            layout.append(&button_layout);

            // callbacks
            self.add_file_button.connect_clicked(glib::clone!(
                #[weak]
                list_store,
                #[strong(rename_to = parent_window)]
                self.parent,
                move |_| {
                    let parent_window = parent_window.borrow().upgrade();
                    if parent_window.is_some() {
                        log::debug!("parent_window is Some");
                    } else {
                        log::debug!("parent_window is None");
                    }
                    let dialog = gtk::FileDialog::builder().title("Choose files...").build();
                    dialog.open_multiple(
                        parent_window.as_ref(),
                        None::<&gio::Cancellable>,
                        move |files_result| match files_result {
                            Ok(files_list_model) => {
                                for file in files_list_model.iter() {
                                    let file: gio::File = file.expect("`file` is not a gio::File.");
                                    let path: PathBuf = file.path().unwrap();
                                    let path_obj = PathObject::new(&path, false);
                                    list_store.append(&path_obj);
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to select files: {}", e);
                            }
                        },
                    );
                }
            ));

            // callback for add_folder_button
            self.add_folder_button.connect_clicked(glib::clone!(
                #[weak]
                list_store,
                #[strong(rename_to = parent_window)]
                self.parent,
                move |_| {
                    let parent_window = parent_window.borrow().upgrade();
                    let dialog = gtk::FileDialog::builder().title("Choose folder...").build();
                    dialog.select_multiple_folders(
                        parent_window.as_ref(),
                        None::<&gio::Cancellable>,
                        move |folders_result| match folders_result {
                            Ok(folders_list_model) => {
                                for folder in folders_list_model.iter() {
                                    let folder: gio::File =
                                        folder.expect("`folder` is not a gio::File.");
                                    let path: PathBuf = folder.path().unwrap();
                                    let path_obj = PathObject::new(&path, true);
                                    list_store.append(&path_obj);
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to select folders: {}", e);
                            }
                        },
                    );
                }
            ));

            // callback for remove_path_button
            self.remove_path_button.connect_clicked({
                let list_store = list_store.downgrade();
                let selection_model = selection_model.downgrade();
                move |_| {
                    let list_store = list_store
                        .upgrade()
                        .expect("list_store should be available");
                    let selection_model = selection_model
                        .upgrade()
                        .expect("selection_model should be available");

                    log::debug!("Remove button clicked");
                    let selected_bitset = selection_model.selection();
                    let list_store_len = list_store.iter::<PathObject>().len();
                    log::debug!(
                        "The lenth for list_store is {list_store_len}. ListItems will \
                        be checked in reverse order."
                    );
                    for i in (0..list_store_len).rev() {
                        log::trace!("Checking index {i}...");
                        if selected_bitset.contains(i as u32) {
                            log::debug!("Removing index {i}...");
                            list_store.remove(i as u32);
                        }
                    }
                }
            });
        }
    }

    impl WidgetImpl for FileSelectArea {}

    impl BinImpl for FileSelectArea {}
}
