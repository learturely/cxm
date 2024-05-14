use cxsign::{
    store::{tables::LocationTable, DataBase, DataBaseTableTrait},
    Location, LocationInfoGetterTrait, LocationSign, SignTrait,
};
use std::sync::{Arc, Mutex};
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
        let table = LocationTable::from_ref(&binding);
        let location_str = location_str.trim();
        location_str
            .parse()
            .ok()
            .or_else(|| table.get_location_by_alias(location_str))
            .or_else(|| {
                location_str
                    .parse()
                    .map(|location_id| {
                        if table.has_location(location_id) {
                            let (_, location) = table.get_location(location_id);
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
        let table = LocationTable::from_ref(&binding);
        table
            .get_location_list_by_course(sign.as_inner().course.get_id())
            .pop()
            .or_else(|| table.get_location_list_by_course(-1).pop())
    }
}
