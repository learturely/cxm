use std::sync::{Arc, Mutex};

use cxlib::{
    default_impl::{
        sign::LocationSign,
        signner::LocationInfoGetterTrait,
        store::{DataBase, LocationTable},
    },
    sign::SignTrait,
    types::Location,
};
pub struct TauriLocationInfoGetter(Arc<Mutex<DataBase>>);
impl TauriLocationInfoGetter {
    pub fn new(db: &Arc<Mutex<DataBase>>) -> Self {
        Self(Arc::clone(db))
    }
}
impl From<Arc<Mutex<DataBase>>> for TauriLocationInfoGetter {
    fn from(db: Arc<Mutex<DataBase>>) -> Self {
        Self(db)
    }
}
impl From<&Arc<Mutex<DataBase>>> for TauriLocationInfoGetter {
    fn from(db: &Arc<Mutex<DataBase>>) -> Self {
        Self::new(db)
    }
}
impl LocationInfoGetterTrait for TauriLocationInfoGetter {
    fn get_location_by_location_str(&self, trimmed_location_str: &str) -> Option<Location> {
        let binding = self.0.lock().unwrap();
        LocationTable::get_location_by_alias(&binding, trimmed_location_str).or_else(|| {
            trimmed_location_str
                .parse()
                .map(|location_id| {
                    if LocationTable::has_location(&binding, location_id) {
                        let (_, location) = LocationTable::get_location(&binding, location_id);
                        Some(location)
                    } else {
                        None
                    }
                })
                .ok()
                .flatten()
        })
    }
    fn get_fallback_location(&self, sign: &LocationSign) -> Option<Location> {
        let binding = self.0.lock().unwrap();
        LocationTable::get_location_list_by_course(&binding, sign.as_inner().course.get_id()).pop()
    }
}
