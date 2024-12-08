use cxlib::{
    default_impl::{
        sign::QrCodeSign,
        signner::{DefaultLocationInfoGetter, DefaultQrCodeSignner, LocationInfoGetterTrait},
    },
    error::SignError,
    sign::{SignResult, SignnerTrait},
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
    type ExtData<'e> = (&'e str, Option<Vec<Location>>);

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &QrCodeSign,
        _: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, SignError> {
        let sessions = self.app_handle.state::<CurrentSignState>().sessions.clone();
        let sessions_lock = sessions.lock().unwrap();
        drop(sessions_lock);
        log::info!("获取预设位置。");
        let app_handle_ = self.app_handle.clone();
        let location_info_getter = Arc::clone(&self.location_info_getter);
        let locations = Arc::new(Mutex::new(None));
        let enc_thread_handle = {
            let uid_set = self
                .app_handle
                .state::<crate::state::CurrentSignUidSetState>()
                .0
                .clone();
            log::info!("初始化二维码签到处理程序。");
            let app_handle = self.app_handle.clone();
            let sign = sign.clone();
            let locations = Arc::clone(&locations);
            std::thread::spawn(move || {
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
                        enc = get_enc();
                    }
                    let sign = sign.clone();
                    let uid_set__ = uid_set.lock().unwrap();
                    let mut handles = Vec::new();
                    let sessions_lock = sessions.lock().unwrap();
                    let sessions_ = sessions_lock.clone();
                    drop(sessions_lock);
                    // 这种写法会有死锁。应该是获取锁的顺序不确定。
                    // let locations = [
                    //     location.lock().unwrap().clone(),
                    //     location_fallback.lock().unwrap().clone(),
                    // ];
                    let locations = locations.lock().unwrap().clone();
                    for session in sessions_.iter().filter(|a| uid_set__.contains(a.get_uid())) {
                        let mut sign = sign.clone();
                        let session = session.clone();
                        let app = app.clone();
                        let locations = locations.clone();
                        let enc = enc.clone();
                        let h = std::thread::spawn(move || {
                            match <Self as SignnerTrait<QrCodeSign>>::sign_single(
                                &mut sign,
                                &session,
                                (&enc, locations),
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
            })
        };
        let preset_location = sign.as_location_sign().get_preset_location();
        // 如果存在预设位置。
        preset_location.map(|_| {
            let sign = sign.clone();
            log::info!("初始化位置信息处理程序。");
            let locations = Arc::clone(&locations);
            let location_info_thread_handle = std::thread::spawn(move || {
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
                    *locations.lock().unwrap() =
                        Some(t1.get_locations(sign.as_location_sign(), &location_str));
                    log::info!("received: `sign:qrcode:location`, end.");
                });
            });
            location_info_thread_handle.join().unwrap();
        });
        enc_thread_handle.join().unwrap();
        type QrCodeUtils<'a> = DefaultQrCodeSignner<'a, DefaultLocationInfoGetter<'a>>;
        fn get_enc() -> String {
            let enc = {
                #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
                let enc = QrCodeUtils::capture_screen_for_enc(false, false).unwrap_or_default();
                #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
                let enc = Default::default();
                enc
            };
            enc
        }
        Ok(Default::default())
    }

    fn sign_single(
        sign: &QrCodeSign,
        session: &Session,
        data: (&str, Option<Vec<Location>>),
    ) -> Result<SignResult, SignError> {
        DefaultQrCodeSignner::<T1>::sign_single(sign, session, data)
    }
}
