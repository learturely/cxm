use std::sync::{Arc, Mutex};

use cxsign::{
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
    fn map_location_str(&self, location_str: &str) -> Option<Location> {
        let binding = self.0.lock().unwrap();
        let location_str = location_str.trim();
        location_str
            .parse()
            .ok()
            .or_else(|| LocationTable::get_location_by_alias(&binding, location_str))
            .or_else(|| {
                location_str
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
        LocationTable::get_location_list_by_course(&binding, sign.as_inner().course.get_id())
            .pop()
            .or_else(|| LocationTable::get_location_list_by_course(&binding, -1).pop())
    }
}
