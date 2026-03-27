use std::path::Path;

use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib;

glib::wrapper! {
    pub struct PathObject(ObjectSubclass<imp::PathObject>);
}

impl PathObject {
    pub fn new(path: impl AsRef<Path>, is_directory: bool) -> Self {
        let obj = glib::Object::new::<Self>();
        let imp = obj.imp();
        imp.is_directory.set(is_directory).unwrap();
        imp.path.set(path.as_ref().into()).unwrap();

        obj
    }
}

mod imp {
    use std::{path::PathBuf, sync::OnceLock};

    use gtk::{
        glib::{self, Properties},
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::PathObject)]
    pub struct PathObject {
        /// whether or not the path refers to a directory
        #[property(get)]
        pub is_directory: OnceLock<bool>,

        /// file / folder path
        #[property(get)]
        pub path: OnceLock<PathBuf>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PathObject {
        const NAME: &'static str = "RecursiveHashPathObject";
        type Type = super::PathObject;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for PathObject {}
}
