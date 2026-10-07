use bridgerton::bridge;

#[bridge]
pub fn set_ui_language(tag: String) {
    yap_i18n::set_ui_language(&tag);
}

#[bridge]
pub fn ui_language() -> String {
    yap_i18n::ui_language().tag().to_owned()
}
