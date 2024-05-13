#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn capture_screen_for_enc() -> Option<String> {
    use cxsign::utils::is_enc_qrcode_url;

    let screens = xcap::Monitor::all().unwrap_or_else(|e| panic!("{e:?}"));
    for screen in screens {
        // 先截取整个屏幕。
        let pic = screen.capture_image().unwrap_or_else(|e| panic!("{e:?}"));
        log::info!("已截屏。");
        // 如果成功识别到二维码。
        let results = cxsign::utils::scan_qrcode(
            xcap::image::DynamicImage::from(pic),
            &mut std::collections::HashMap::new(),
        );
        let results = if let Ok(results) = results {
            results
        } else {
            continue;
        };
        // 在结果中寻找。
        for r in &results {
            let url = r.getText();
            // 如果符合要求的二维码。
            if !is_enc_qrcode_url(url) {
                log::warn!("{url:?}不是有效的签到二维码！");
                continue;
            }
            log::info!("存在签到二维码。");
            // 如果不是精确截取的二维码，则不需要提示。
            return cxsign::utils::find_qrcode_sign_enc_in_url(url);
        }
    }
    None
}
