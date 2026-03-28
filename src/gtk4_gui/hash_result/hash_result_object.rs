use std::path::PathBuf;

use gtk::glib;
use gtk::glib::property::PropertySet;
use gtk::subclass::prelude::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct HashResultObj(ObjectSubclass<imp::HashResultObj>);
}

impl HashResultObj {
    pub fn new(path: PathBuf, hash_val: String) -> Self {
        let obj = glib::Object::builder::<Self>().build();
        let imp = obj.imp();
        imp.path.set(path).unwrap();
        imp.hash_val.set(hash_val);
        obj
    }
}

mod imp {
    use std::{cell::RefCell, path::PathBuf, sync::OnceLock};

    use gtk::{
        glib::{self, Properties},
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::HashResultObj)]
    pub struct HashResultObj {
        /// path to file
        #[property(get)]
        pub path: OnceLock<PathBuf>,

        /// hash value of the file
        #[property(get, set)]
        pub hash_val: RefCell<String>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for HashResultObj {
        const NAME: &'static str = "HashResultObject";
        type Type = super::HashResultObj;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for HashResultObj {}
}
