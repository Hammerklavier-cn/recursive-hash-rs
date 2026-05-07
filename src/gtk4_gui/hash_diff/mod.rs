use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio::prelude::ListModelExtManual, glib};

use crate::gtk4_gui::hash_diff::hash_diff_object::HashDiffObj;

pub mod hash_diff_object;

glib::wrapper! {
    pub struct HashDiffArea(ObjectSubclass<imp::HashDiffArea>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl HashDiffArea {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn remove_all(&self) {
        self.imp().model.remove_all();
    }

    pub fn append(
        &self,
        p: impl AsRef<Path>,
        got_hash_val: Option<impl AsRef<str>>,
        real_hash_val: Option<impl AsRef<str>>,
    ) {
        let imp = self.imp();
        let model = imp.model.clone();

        let item = HashDiffObj::new(p.as_ref(), got_hash_val, real_hash_val);
        model.append(&item);
    }

    pub fn insert_at(
        &self,
        pos: u32,
        p: impl AsRef<Path>,
        got_hash_val: Option<impl AsRef<str>>,
        real_hash_val: Option<impl AsRef<str>>,
    ) {
        let imp = self.imp();
        let model = imp.model.clone();

        let item = HashDiffObj::new(p.as_ref(), got_hash_val, real_hash_val);
        model.insert(pos, &item);
    }

    pub fn remove(&self, p: impl AsRef<Path>) {
        let imp = self.imp();
        let model = imp.model.clone();

        for (i, res) in model.iter::<HashDiffObj>().enumerate() {
            let hash_diff_obj = res.expect("Needs to be HashDiffObj");
            if hash_diff_obj.path() == p.as_ref() {
                model.remove(i as u32);
                return;
            }
        }
    }

    pub fn update_got_hash_val(&self, p: impl AsRef<Path>, hash: String) {
        let imp = self.imp();
        let model = imp.model.clone();

        for res in model.iter::<HashDiffObj>() {
            let hash_diff_obj = res.expect("Needs to be HashDiffObj");
            if hash_diff_obj.path() == p.as_ref() {
                hash_diff_obj.set_got_hash_val(hash.clone());
                // update status
                hash_diff_obj.update_status();
                return;
            }
        }
    }

    pub fn update_real_hash_val(&self, p: impl AsRef<Path>, hash: String) {
        let imp = self.imp();
        let model = imp.model.clone();

        for res in model.iter::<HashDiffObj>() {
            let hash_diff_obj = res.expect("Needs to be HashDiffObj");
            if hash_diff_obj.path() == p.as_ref() {
                hash_diff_obj.set_real_hash_val(hash.clone());
                // update status
                hash_diff_obj.update_status();
                return;
            }
        }
    }

    pub fn batch_update_got_hash(&self, batch: &HashMap<PathBuf, String>) {
        let imp = self.imp();
        let model = &imp.model;

        for res in model.iter::<HashDiffObj>() {
            let hash_diff_obj = res.unwrap();
            if let Some(hash) = batch.get(&hash_diff_obj.path()) {
                hash_diff_obj.set_got_hash_val(hash.as_str());
                // update status
                hash_diff_obj.update_status();
            }
        }
    }

    pub fn batch_update_real_hash(&self, batch: &HashMap<PathBuf, String>) {
        let imp = self.imp();
        let model = &imp.model;

        for res in model.iter::<HashDiffObj>() {
            let hash_diff_obj = res.unwrap();
            if let Some(hash) = batch.get(&hash_diff_obj.path()) {
                hash_diff_obj.set_real_hash_val(hash.as_str());
                // update status
                hash_diff_obj.update_status();
            }
        }
    }
}

mod imp {
    use adw::prelude::BinExt;
    use adw::subclass::bin::BinImpl;
    use adw::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt};
    use gtk::glib::object::{Cast, CastNone, ObjectExt};
    use gtk::prelude::{ListItemExt, WidgetExt};
    use gtk::{ColumnViewColumn, SignalListItemFactory};
    use gtk::{
        gio,
        glib::{self},
        subclass::{prelude::ObjectImplExt, widget::WidgetImpl},
    };

    use crate::gtk4_gui::hash_diff::hash_diff_object::HashDiffObj;

    pub struct HashDiffArea {
        /// Data model storing HashDiffObj items
        pub(super) model: gio::ListStore,
        pub(super) column_view: gtk::ColumnView,
        /// ScrolledWindow wrapper for the ColumnView
        pub(super) scrolled_window: gtk::ScrolledWindow,
    }

    impl Default for HashDiffArea {
        fn default() -> Self {
            let model = gio::ListStore::new::<HashDiffObj>();
            let selection = gtk::NoSelection::new(Some(model.clone()));
            let column_view = gtk::ColumnView::new(Some(selection));
            let scrolled_window = gtk::ScrolledWindow::builder()
                .hexpand(true)
                .vexpand(true)
                .min_content_height(300)
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
    impl ObjectSubclass for HashDiffArea {
        const NAME: &'static str = "HashDiffArea";
        type Type = super::HashDiffArea;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for HashDiffArea {
        fn constructed(&self) {
            self.parent_constructed();

            // Set up the child widgets
            let obj = self.obj();
            obj.set_child(Some(&self.scrolled_window));
            self.scrolled_window.set_child(Some(&self.column_view));

            let create_column = |name: &str,
                                 setup: Box<dyn Fn() -> gtk::Widget + 'static>,
                                 bind: Box<
                dyn Fn(&HashDiffObj, &gtk::Widget) -> Option<glib::Binding> + 'static,
            >|
             -> ColumnViewColumn {
                let factory = SignalListItemFactory::new();

                factory.connect_setup(move |_, list_item| {
                    let list_item = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Needs to be ListItem");
                    let child_label = setup();
                    list_item.set_child(Some(&child_label));
                });

                factory.connect_bind(move |_, list_item| {
                    let list_item = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Needs to be ListItem");
                    let child_label = list_item.child().expect("Needs to have a Widget child");
                    let diff_obj = list_item
                        .item()
                        .and_downcast::<HashDiffObj>()
                        .expect("Needs to be HashDiffObj");

                    if let Some(binding) = bind(&diff_obj, &child_label) {
                        unsafe {
                            list_item.set_data("binding", binding);
                        }
                    }
                });

                factory.connect_unbind(move |_, list_item| {
                    let list_item = list_item
                        .downcast_ref::<gtk::ListItem>()
                        .expect("Needs to be ListItem");

                    if let Some(binding) = unsafe { list_item.data::<glib::Binding>("binding") } {
                        unsafe { binding.read() }.unbind();
                    }
                });

                ColumnViewColumn::new(Some(name), Some(factory))
            };

            // Create the path column
            self.column_view.append_column(&create_column(
                "Path",
                Box::new(|| {
                    let label = gtk::Label::new(None);
                    label.set_halign(gtk::Align::Start);
                    label.set_hexpand(true);
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    label.upcast::<gtk::Widget>()
                }),
                Box::new(|diff_obj, child_widget| {
                    let path = diff_obj.path();
                    let label = child_widget
                        .downcast_ref::<gtk::Label>()
                        .expect("Needs to be Label");
                    label.set_text(&path.to_string_lossy());
                    None
                }),
            ));

            // Create the column for the check status
            self.column_view.append_column(&create_column(
                "Checked",
                Box::new(|| {
                    let icon = gtk::Image::new();
                    icon.upcast::<gtk::Widget>()
                }),
                Box::new(|diff_obj, child_widget| {
                    let child_icon = child_widget
                        .downcast_ref::<gtk::Image>()
                        .expect("Needs to be Image");
                    Some(
                        diff_obj
                            .bind_property("status", child_icon, "icon-name")
                            .sync_create()
                            .build(),
                    )
                }),
            ));

            // Create the column for the recorded hash value
            self.column_view.append_column(&create_column(
                "Recorded Hash",
                Box::new(|| {
                    let label = gtk::Label::new(None);
                    label.upcast::<gtk::Widget>()
                }),
                Box::new(|diff_obj, child_widget| {
                    let label = child_widget
                        .downcast_ref::<gtk::Label>()
                        .expect("Needs to be Label");
                    Some(
                        diff_obj
                            .bind_property("got_hash_val", label, "text")
                            .sync_create()
                            .build(),
                    )
                }),
            ));

            self.column_view.append_column(&create_column(
                "Calculated Hash",
                Box::new(|| {
                    let label = gtk::Label::new(None);
                    label.upcast::<gtk::Widget>()
                }),
                Box::new(|diff_obj, child_widget| {
                    let label = child_widget
                        .downcast_ref::<gtk::Label>()
                        .expect("Needs to be Label");
                    Some(
                        diff_obj
                            .bind_property("real_hash_val", label, "text")
                            .sync_create()
                            .build(),
                    )
                }),
            ));

            // Create the column for the actual calculated hash value
        }
    }

    impl WidgetImpl for HashDiffArea {}

    impl BinImpl for HashDiffArea {}
}
