import createI18n from "../lib/createI18n";
import { DEFAULT_APP_LANGUAGE, translation } from "./translation";

const STORAGE_KEY = "hyperhost_lang";

const _i18n = createI18n(translation);

// Khởi tạo locale từ localStorage hoặc mặc định
const savedLocale = (() => {
  try {
    return localStorage.getItem(STORAGE_KEY) || DEFAULT_APP_LANGUAGE;
  } catch {
    return DEFAULT_APP_LANGUAGE;
  }
})();
_i18n.setLocale(savedLocale);

export const i18n = {
  t: _i18n.t,
};

export const getLocale = _i18n.getLocale;

/**
 * Đổi ngôn ngữ và lưu vào localStorage.
 * Trả về locale mới để component có thể trigger re-render.
 */
export const setLocale = (locale?: string): string => {
  const next = locale || DEFAULT_APP_LANGUAGE;
  _i18n.setLocale(next);
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Bỏ qua lỗi localStorage (private browsing, etc.)
  }
  return next;
};

export type SupportedLocale = "vi" | "en";

export const SUPPORTED_LOCALES: { value: SupportedLocale; label: string }[] = [
  { value: "vi", label: "🇻🇳 Tiếng Việt" },
  { value: "en", label: "🇬🇧 English" },
];
