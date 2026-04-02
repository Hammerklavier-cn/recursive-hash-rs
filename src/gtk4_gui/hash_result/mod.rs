use std::{collections::HashMap, path::PathBuf};

use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio::prelude::ListModelExtManual, glib};

use crate::gtk4_gui::hash_result::hash_result_object::HashResultObj;

mod hash_result_object;

glib::wrapper! {
    pub struct HashResultArea(ObjectSubclass<imp::HashResultArea>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl HashResultArea {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn remove_all(&self) {
        self.imp().model.remove_all();
    }

    pub fn add_result(&self, path: std::path::PathBuf, hash_val: String) {
        let obj = hash_result_object::HashResultObj::new(path, hash_val);
        self.imp().model.append(&obj);
    }

    pub fn update_result(&self, path: std::path::PathBuf, hash_val: String) {
        let imp = self.imp();
        let model = &imp.model;

        for res in model.iter::<HashResultObj>() {
            let hash_res = res.unwrap();
            if hash_res.path() == path {
                log::debug!(
                    "Compare path: {} matches {}",
                    path.display(),
                    hash_res.path().display(),
                );
                hash_res.set_hash_val(hash_val);
                break;
            } else {
                log::trace!(
                    "{} doesn't match {}",
                    path.display(),
                    hash_res.path().display(),
                )
            }
        }
    }

    pub fn batch_update_results(&self, batch: &HashMap<PathBuf, String>) {
        let imp = self.imp();
        let model = &imp.model;

        for res in model.iter::<HashResultObj>() {
            let hash_res = res.unwrap();
            if let Some(hash_val) = batch.get(&hash_res.path()) {
                log::debug!("Updating hash result for {:?}", hash_res.path());
                hash_res.set_hash_val(hash_val.clone());
            }
        }
    }

    pub fn clear(&self) {
        self.imp().model.remove_all();
    }
}

mod imp {
    use adw::{
        prelude::BinExt,
        subclass::{
            bin::BinImpl,
            prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt},
        },
    };
    use gtk::{
        ColumnViewColumn, SignalListItemFactory, gio,
        glib::{
            self, Properties,
            object::{CastNone, ObjectExt},
        },
        prelude::{Cast, ListItemExt, WidgetExt},
        subclass::{prelude::DerivedObjectProperties, widget::WidgetImpl},
    };

    use crate::gtk4_gui::hash_result::hash_result_object::HashResultObj;

    #[derive(Properties)]
    #[properties(wrapper_type = super::HashResultArea)]
    pub struct HashResultArea {
        /// Data model storing HashResultObj items
        pub(super) model: gio::ListStore,
        /// There are `path` and `hash` columns in the view.
        #[property(get)]
        pub(super) column_view: gtk::ColumnView,
        /// ScrolledWindow wrapper for the ColumnView
        pub(super) scrolled_window: gtk::ScrolledWindow,
    }

    impl Default for HashResultArea {
        fn default() -> Self {
            let model = gio::ListStore::new::<HashResultObj>();
            let selection = gtk::NoSelection::new(Some(model.clone()));
            let column_view = gtk::ColumnView::new(Some(selection));
            let scrolled_window = gtk::ScrolledWindow::builder()
                .hexpand(true)
                .vexpand(true)
                .min_content_height(200)
                .min_content_width(300)
                .child(&column_view)
                .build();
            Self {
                model,
                column_view,
                scrolled_window,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HashResultArea {
        const NAME: &'static str = "HashResultArea";
        type Type = super::HashResultArea;
        type ParentType = adw::Bin;
    }

    #[glib::derived_properties]
    impl ObjectImpl for HashResultArea {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Create the path column
            let path_column = self.create_column(
                "Path",
                || {
                    let label = gtk::Label::new(None);
                    label.set_halign(gtk::Align::Start);
                    label.set_hexpand(true);
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    label
                },
                |hash_result_obj, child_label| {
                    let path = hash_result_obj.path();
                    child_label.set_text(&path.to_string_lossy());
                },
                || {
                    let label = gtk::Label::new(None);
                    label.set_halign(gtk::Align::Start);
                    label.set_hexpand(true);
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    label
                },
            );

            // Create the hash column
            let hash_column = self.create_column(
                "Hash",
                || {
                    let label = gtk::Label::new(None);
                    label.set_halign(gtk::Align::Start);
                    // label.set_hexpand(true);
                    label.add_css_class("monospace");
                    label
                },
                |hash_res_obj, child_label| {
                    hash_res_obj
                        .bind_property("hash_val", child_label, "label")
                        .sync_create()
                        .build();
                },
                || {
                    let label = gtk::Label::new(None);
                    label.set_halign(gtk::Align::Start);
                    // label.set_hexpand(true);
                    label.add_css_class("monospace");
                    label
                },
            );

            path_column.set_fixed_width(200);
            path_column.set_resizable(true);
            hash_column.set_expand(true);
            hash_column.set_resizable(true);

            // Add columns to the view
            self.column_view.append_column(&path_column);
            self.column_view.append_column(&hash_column);

            // Enable visual separators
            self.column_view.set_show_row_separators(true);
            self.column_view.set_show_column_separators(true);

            // Set the scrolled window as the child (which contains the column_view)
            obj.set_child(Some(&self.scrolled_window));
        }
    }

    impl HashResultArea {
        fn create_column<FSetup, FBind, FUnbind>(
            &self,
            title: &str,
            setup: FSetup,
            bind: FBind,
            unbind: FUnbind,
        ) -> ColumnViewColumn
        where
            FSetup: Fn() -> gtk::Label + 'static,
            FBind: Fn(&HashResultObj, &gtk::Label) + 'static,
            FUnbind: Fn() -> gtk::Label + 'static,
        {
            let factory = SignalListItemFactory::new();

            factory.connect_setup(move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let child = setup();
                list_item.set_child(Some(&child));
            });

            factory.connect_bind(move |_, list_item| {
                let list_item = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                let child_label = list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .expect("Needs to be Label");
                let res_obj = list_item
                    .item()
                    .and_downcast::<HashResultObj>()
                    .expect("Needs to be HashResultObj");

                bind(&res_obj, &child_label);
            });

            factory.connect_unbind(move |_, list_item| {
                let list_item = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                list_item.set_child(Some(&unbind()));
            });

            ColumnViewColumn::new(Some(title), Some(factory))
        }
    }

    impl WidgetImpl for HashResultArea {}

    impl BinImpl for HashResultArea {}
}
