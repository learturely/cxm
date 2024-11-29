use cxlib::{
    default_impl::{sign::QrCodeSign, signner::LocationInfoGetterTrait},
    error::Error,
    sign::{SignResult, SignTrait},
    signner::SignnerTrait,
    types::Location,
    user::Session,
};
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, Listener, Manager};

use crate::CurrentSignState;

pub struct TauriQrCodeSignner<
    T1: LocationInfoGetterTrait + Sync + Send + for<'a> From<&'a Arc<Mutex<T2>>>,
    T2,
> where
    Arc<Mutex<T2>>: Sync + Send,
{
    app_handle: AppHandle,
    location_info_getter: Arc<Mutex<T2>>,
    _p: PhantomData<T1>,
}
impl<T1: LocationInfoGetterTrait + Sync + Send + for<'a> From<&'a Arc<Mutex<T2>>>, T2>
    TauriQrCodeSignner<T1, T2>
where
    Arc<Mutex<T2>>: Sync + Send,
{
    pub fn new(location_info_getter: Arc<Mutex<T2>>, app_handle: AppHandle) -> Self {
        Self {
            location_info_getter,
            app_handle,
            _p: PhantomData,
        }
    }
}

impl<T1: LocationInfoGetterTrait + Sync + Send + for<'a> From<&'a Arc<Mutex<T2>>>, T2: 'static>
    SignnerTrait<QrCodeSign> for TauriQrCodeSignner<T1, T2>
where
    Arc<Mutex<T2>>: Sync + Send,
{
    type ExtData<'e> = Location;

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut QrCodeSign,
        _: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let sessions = self.app_handle.state::<CurrentSignState>().sessions.clone();
        let uid_set = self
            .app_handle
            .state::<crate::state::CurrentSignUidSetState>()
            .0
            .clone();
        let sessions_lock = sessions.lock().unwrap();
        let preset_location =
            T1::from(&self.location_info_getter).get_preset_location(sign.as_location_sign_mut());
        drop(sessions_lock);
        log::info!("获取预设位置。");
        let app_handle_ = self.app_handle.clone();
        let location_info_getter = Arc::clone(&self.location_info_getter);
        let mut location = Arc::new(Mutex::new(Location::get_none_location()));
        log::info!("初始化位置信息处理程序。");
        let location_info_thread_handle = if let Some(preset_location) = preset_location {
            // global_locations.append(&mut course_locations);
            // let locations = global_locations;
            // let locations = Arc::new(Mutex::new(locations));
            location = Arc::new(Mutex::new(preset_location.clone()));
            let preset_location = Arc::new(Mutex::new(preset_location));
            let location = Arc::clone(&location);
            std::thread::spawn(move || {
                let app = app_handle_.clone();
                app_handle_.listen("sign:qrcode:location", move |p| {
                    log::info!("received: `sign:qrcode:location`.");
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app.unlisten(p.id());
                        return;
                    }
                    let crate::LocationSignnerInfo { location_str } = p.payload().parse().unwrap();
                    let t1 = T1::from(&Arc::clone(&location_info_getter));
                    if let Some(location_str) = location_str.as_ref() {
                        t1.map_location_str(location_str).map_or_else(
                            || {
                                let mut preset_location = preset_location.lock().unwrap();
                                if !location_str.is_empty() {
                                    preset_location.set_addr(location_str);
                                }
                                *location.lock().unwrap() = preset_location.clone();
                            },
                            |location_| {
                                let mut preset_location = preset_location.lock().unwrap().clone();
                                preset_location.set_addr(location_.get_addr());
                                *location.lock().unwrap() = location_;
                            },
                        )
                    };
                    log::info!("received: `sign:qrcode:location`, end.");
                });
            })
        } else {
            let location_info_getter_ = T1::from(&Arc::clone(&self.location_info_getter));
            if let Some(location_) =
                location_info_getter_.get_fallback_location(sign.as_location_sign_mut())
            {
                location = Arc::new(Mutex::new(location_.clone()));
                let location_fallback = location_.clone();
                let location = Arc::clone(&location);
                std::thread::spawn(move || {
                    let app = app_handle_.clone();
                    app_handle_.listen("sign:qrcode:location", move |p| {
                        log::info!("received: `sign:qrcode:location`.");
                        if p.payload() == "\"quit\"" {
                            log::info!("quit");
                            app.unlisten(p.id());
                            return;
                        }
                        let crate::LocationSignnerInfo { location_str } =
                            p.payload().parse().unwrap();
                        if let Some(location_str) = location_str.as_ref() {
                            let location_info_getter_ =
                                T1::from(&Arc::clone(&location_info_getter));
                            location_info_getter_
                                .map_location_str(location_str)
                                .map_or_else(
                                    || {
                                        *location.lock().unwrap() = location_fallback.clone();
                                    },
                                    |location_| {
                                        *location.lock().unwrap() = location_;
                                    },
                                )
                        }
                        log::info!("received: `sign:qrcode:location`, end.");
                    });
                })
            } else {
                let location = Arc::clone(&location);
                let location_info_getter = Arc::clone(&self.location_info_getter);
                std::thread::spawn(move || {
                    let app = app_handle_.clone();
                    app_handle_.listen("sign:qrcode:location", move |p| {
                        log::info!("received: `sign:qrcode:location`.");
                        if p.payload() == "\"quit\"" {
                            log::info!("quit");
                            app.unlisten(p.id());
                            return;
                        }
                        let crate::LocationSignnerInfo { location_str } =
                            p.payload().parse().unwrap();

                        if let Some(location_str) = location_str.as_ref() {
                            T1::from(&Arc::clone(&location_info_getter))
                                .map_location_str(location_str)
                                .map_or_else(
                                    || {},
                                    |location_| {
                                        *location.lock().unwrap() = location_;
                                    },
                                )
                        }
                        log::info!("received: `sign:qrcode:location`, end.");
                    });
                })
            }
        };
        let app_handle = self.app_handle.clone();
        let sign = sign.clone();
        log::info!("初始化二维码签到处理程序。");
        let enc_thread_handle = std::thread::spawn(move || {
            let uid_set_ = Arc::clone(&uid_set);
            let app = app_handle.clone();
            app_handle.listen("sign:qrcode:enc", move |p| {
                log::info!("received: `sign:qrcode:enc`.");
                if p.payload() == "\"quit\"" {
                    log::info!("quit");
                    app.unlisten(p.id());
                    return;
                }
                let mut enc = p.payload().trim_matches(|c| c == '"').to_string();
                if enc.is_empty() {
                    enc = get_enc().unwrap_or_default();
                }
                let mut sign = sign.clone();
                sign.set_enc(enc);
                let uid_set__ = uid_set_.lock().unwrap();
                let mut handles = Vec::new();
                let sessions_lock = sessions.lock().unwrap();
                let sessions_ = sessions_lock.clone();
                drop(sessions_lock);
                // 这种写法会有死锁。应该是获取锁的顺序不确定。
                // let locations = [
                //     location.lock().unwrap().clone(),
                //     location_fallback.lock().unwrap().clone(),
                // ];
                let location1 = location.lock().unwrap().clone();
                for session in sessions_.iter().filter(|a| uid_set__.contains(a.get_uid())) {
                    let mut sign = sign.clone();
                    let session = session.clone();
                    let app = app.clone();
                    let location = location1.clone();
                    let h = std::thread::spawn(move || {
                        match <Self as SignnerTrait<QrCodeSign>>::sign_single(
                            &mut sign, &session, location,
                        )
                        .unwrap_or_else(|e| SignResult::Fail { msg: e.to_string() })
                        {
                            SignResult::Susses => {
                                app.emit("sign:susses", session.get_uid()).unwrap();
                            }
                            SignResult::Fail { msg } => {
                                app.emit("sign:fail", [session.get_uid(), &msg]).unwrap()
                            }
                        };
                    });
                    handles.push(h);
                }
                for h in handles {
                    h.join().unwrap();
                }
                log::info!("received: `sign:qrcode:enc`, end.");
            });
        });
        location_info_thread_handle.join().unwrap();
        enc_thread_handle.join().unwrap();
        fn get_enc() -> Result<String, Error> {
            let enc = {
                #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
                let enc =
                    cxlib::qrcode_utils::capture_screen_for_enc(false, false).unwrap_or_default();
                #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
                let enc = Default::default();
                enc
            };
            Ok(enc)
        }
        Ok(Default::default())
    }

    fn sign_single(
        sign: &mut QrCodeSign,
        session: &Session,
        location: Location,
    ) -> Result<SignResult, Error> {
        let r = sign.pre_sign(session).map_err(Error::from)?;
        sign.set_location(location);
        unsafe { sign.sign_unchecked(session, r) }.map_err(Error::from)
    }
}
