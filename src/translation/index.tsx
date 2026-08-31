import React, { createContext, useContext, useState, useEffect, ReactNode, useMemo } from "react";
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

interface I18nContextType {
  locale: SupportedLocale;
  setLocale: (l: SupportedLocale) => void;
  t: typeof _i18n.t;
  locales: typeof SUPPORTED_LOCALES;
}

const I18nContext = createContext<I18nContextType>({
  locale: savedLocale as SupportedLocale,
  setLocale: () => {},
  t: _i18n.t,
  locales: SUPPORTED_LOCALES,
});

export const I18nProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [locale, setLocaleState] = useState<SupportedLocale>(() => {
    try {
      return (localStorage.getItem(STORAGE_KEY) as SupportedLocale) || (DEFAULT_APP_LANGUAGE as SupportedLocale);
    } catch {
      return DEFAULT_APP_LANGUAGE as SupportedLocale;
    }
  });

  const handleSetLocale = (l: SupportedLocale) => {
    setLocale(l);
    setLocaleState(l);
  };

  useEffect(() => {
    _i18n.setLocale(locale);
  }, [locale]);

  const value = useMemo(
    () => ({
      locale,
      setLocale: handleSetLocale,
      t: _i18n.t,
      locales: SUPPORTED_LOCALES,
    }),
    [locale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
};

export const useI18n = () => useContext(I18nContext);
export const useTranslation = useI18n;
