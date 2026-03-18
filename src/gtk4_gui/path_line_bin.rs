use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib;

use crate::gtk4_gui::path_object::PathObject;

glib::wrapper! {
    /// This GObject represents a line in the path viewer, displaying an icon and a label.
    /// The icon is displayed on the left, indicating the path is a directory or a file.
    /// The label is the absolute path of the file or directory.
    pub struct PathLineBin(ObjectSubclass<imp::PathLineBin>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PathLineBin {
    pub fn new(is_directory: Option<bool>, path: impl AsRef<std::path::Path>) -> Self {
        let obj = glib::Object::new::<Self>();
        let imp = obj.imp();

        match is_directory {
            Some(true) => {
                imp.icon.set_icon_name(Some("folder"));
            }
            Some(false) => {
                imp.icon.set_icon_name(Some("document"));
            }
            None => {
                imp.icon.set_icon_name(None);
            }
        }

        imp.label.set_text(path.as_ref().to_str().unwrap());

        obj
    }

    pub fn update_from_path_object(&self, path_object: &PathObject) {
        let imp = self.imp();

        match path_object.is_directory() {
            true => {
                imp.icon.set_icon_name(Some("folder"));
            }
            false => {
                imp.icon.set_icon_name(Some("document"));
            }
        }
        imp.label.set_text(path_object.path().to_str().unwrap());
    }
}

mod imp {
    use adw::prelude::BinExt;
    use adw::subclass::bin::BinImpl;
    use adw::subclass::prelude::ObjectImpl;
    use adw::subclass::prelude::ObjectImplExt;
    use adw::subclass::prelude::ObjectSubclass;
    use adw::subclass::prelude::ObjectSubclassExt;
    use gtk::glib;
    use gtk::prelude::BoxExt;
    use gtk::subclass::widget::WidgetImpl;

    #[derive(Default)]
    pub struct PathLineBin {
        pub(super) icon: gtk::Image,
        pub(super) label: gtk::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PathLineBin {
        const NAME: &'static str = "PathLineBin";
        type Type = super::PathLineBin;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for PathLineBin {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // create a horizontal box to hold the icon and label
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            hbox.append(&self.icon);
            hbox.append(&self.label);
            obj.set_child(Some(&hbox));
        }
    }

    impl WidgetImpl for PathLineBin {}

    impl BinImpl for PathLineBin {}
}
