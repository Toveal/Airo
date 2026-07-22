import { createI18n } from 'vue-i18n';
import en from './en.json';
import ru from './ru.json';

/**
 * The language comes from Rust: `system` is resolved there, because the backend
 * draws the tray menu and it has to match the language of the window.
 */

export const FALLBACK_LOCALE = 'en';

/** The order here defines the order of entries in the language picker. */
export const LOCALES = [
  { code: 'en', label: 'English' },
  { code: 'ru', label: 'Русский' },
];

export const i18n = createI18n({
  // Composition API: strings are read through useI18n(), no global $t needed.
  legacy: false,
  locale: FALLBACK_LOCALE,
  fallbackLocale: FALLBACK_LOCALE,
  messages: { en, ru },
});

export function setLocale(locale) {
  const next = LOCALES.some((l) => l.code === locale) ? locale : FALLBACK_LOCALE;
  i18n.global.locale.value = next;
  // Affects line breaking, quotation marks and font selection in WebView2.
  document.documentElement.lang = next;
}
