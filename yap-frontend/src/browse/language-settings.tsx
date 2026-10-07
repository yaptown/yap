import { t, UI_LANGUAGE_KEY } from "@/lib/i18n";

export function LanguageSettings() {
  return (
    <label className="flex items-center gap-2 px-2 py-1.5 text-sm">
      {t("settings-language")}
      <select
        className="min-w-0 flex-1 rounded bg-background p-1"
        defaultValue={localStorage.getItem(UI_LANGUAGE_KEY) ?? ""}
        onChange={(event) => {
          if (event.target.value)
            localStorage.setItem(UI_LANGUAGE_KEY, event.target.value);
          else localStorage.removeItem(UI_LANGUAGE_KEY);
          window.location.reload();
        }}
      >
        <option value="">{t("settings-language-automatic")}</option>
        <option value="en">{t("ui-language-english")}</option>
        <option value="fr">{t("ui-language-french")}</option>
        <option value="es">{t("ui-language-spanish")}</option>
        <option value="zh-Hans">{t("ui-language-chinese")}</option>
      </select>
    </label>
  );
}
