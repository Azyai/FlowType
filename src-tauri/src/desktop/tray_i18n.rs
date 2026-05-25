use crate::settings::LocalePreference;

#[derive(Debug, Clone, Copy)]
pub struct TrayLabels {
    pub open_settings: &'static str,
    pub show_mascot: &'static str,
    pub hide_mascot: &'static str,
    pub quit: &'static str,
}

const EN_US: TrayLabels = TrayLabels {
    open_settings: "Open Settings",
    show_mascot: "Show Floating Mascot",
    hide_mascot: "Hide Floating Mascot",
    quit: "Quit FlowType",
};

const ZH_CN: TrayLabels = TrayLabels {
    open_settings: "打开设置",
    show_mascot: "显示桌宠",
    hide_mascot: "隐藏桌宠",
    quit: "退出 FlowType",
};

pub fn tray_labels(preference: &LocalePreference) -> TrayLabels {
    match preference {
        LocalePreference::ZhCn => ZH_CN,
        LocalePreference::EnUs => EN_US,
        LocalePreference::Auto => {
            let locale = sys_locale::get_locale().unwrap_or_default();
            if locale.to_lowercase().starts_with("zh") {
                ZH_CN
            } else {
                EN_US
            }
        }
    }
}
