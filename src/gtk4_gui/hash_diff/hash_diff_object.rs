use std::path::Path;

use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;

#[derive(Default, Clone, Copy)]
pub enum CheckStatus {
    #[default]
    Unchecked,
    Match,
    Miss,
}
impl CheckStatus {
    pub fn icon_name(&self) -> String {
        let str: &str = self.into();
        String::from(str)
    }
}

impl From<&str> for CheckStatus {
    fn from(s: &str) -> Self {
        match s {
            "box-symbolic" => Self::Unchecked,
            "checkmark-symbolic" => Self::Match,
            "gtk-no-symbolic" => Self::Miss,
            _ => Self::Unchecked,
        }
    }
}

impl Into<&'static str> for &CheckStatus {
    fn into(self) -> &'static str {
        match self {
            CheckStatus::Unchecked => "box-symbolic",
            CheckStatus::Match => "checkmark-symbolic",
            CheckStatus::Miss => "gtk-no-symbolic",
        }
    }
}

glib::wrapper! {
    pub struct HashDiffObj(ObjectSubclass<imp::HashDiffObj>);
}

impl HashDiffObj {
    pub fn new(
        path: impl AsRef<Path>,
        got_hash_val: Option<impl AsRef<str>>,
        real_hash_val: Option<impl AsRef<str>>,
    ) -> Self {
        let obj = glib::Object::builder::<Self>().build();
        let imp = obj.imp();
        imp.path.set(path.as_ref().to_path_buf()).unwrap();
        if let Some(got) = got_hash_val {
            obj.set_got_hash_val(got.as_ref());
        }
        if let Some(real) = real_hash_val {
            obj.set_real_hash_val(real.as_ref());
        }
        obj
    }

    pub fn check_status(&self) -> CheckStatus {
        self.status().as_str().into()
    }

    pub fn update_status(&self) {
        let got = self.got_hash_val();
        let real = self.real_hash_val();
        let new_status = if got.is_empty() || real.is_empty() {
            CheckStatus::Unchecked
        } else if got == real {
            CheckStatus::Match
        } else {
            CheckStatus::Miss
        };
        self.set_status(new_status.icon_name());
    }
}

mod imp {
    use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::OnceLock};

    use gtk::{
        glib::{self, Properties},
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::HashDiffObj)]
    pub struct HashDiffObj {
        /// path to file
        #[property(get)]
        pub path: OnceLock<PathBuf>,

        #[property(get, set)]
        pub status: Rc<RefCell<String>>,

        /// hash value of the file read from the checklist
        #[property(get, set)]
        pub got_hash_val: Rc<RefCell<String>>,

        /// actual hash value of the file
        #[property(get, set)]
        pub real_hash_val: Rc<RefCell<String>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for HashDiffObj {
        const NAME: &'static str = "HashDiffObject";
        type Type = super::HashDiffObj;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for HashDiffObj {}
}
