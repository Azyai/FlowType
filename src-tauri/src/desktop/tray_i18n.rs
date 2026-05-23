use crate::settings::{LocalePreference, OutputStyle};

#[derive(Debug, Clone, Copy)]
pub struct TrayLabels {
    pub open_settings: &'static str,
    pub pause_voice: &'static str,
    pub resume_voice: &'static str,
    pub show_mascot: &'static str,
    pub mode_raw: &'static str,
    pub mode_clean: &'static str,
    pub mode_formal: &'static str,
    pub check_microphone: &'static str,
    pub view_history: &'static str,
    pub quit: &'static str,
}

const EN_US: TrayLabels = TrayLabels {
    open_settings: "Open Settings",
    pause_voice: "Pause Voice Input",
    resume_voice: "Resume Voice Input",
    show_mascot: "Show Floating Mascot",
    mode_raw: "Mode: Raw",
    mode_clean: "Mode: Clean",
    mode_formal: "Mode: Formal",
    check_microphone: "Check Microphone",
    view_history: "View History",
    quit: "Quit FlowType",
};

const ZH_CN: TrayLabels = TrayLabels {
    open_settings: "打开设置",
    pause_voice: "暂停语音输入",
    resume_voice: "恢复语音输入",
    show_mascot: "显示桌宠悬浮窗",
    mode_raw: "模式：原始转写",
    mode_clean: "模式：干净文本",
    mode_formal: "模式：正式表达",
    check_microphone: "检查麦克风",
    view_history: "查看历史记录",
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

pub fn mode_label(labels: TrayLabels, output_style: &OutputStyle) -> &'static str {
    match output_style {
        OutputStyle::Raw => labels.mode_raw,
        OutputStyle::Clean => labels.mode_clean,
        OutputStyle::Formal => labels.mode_formal,
    }
}
