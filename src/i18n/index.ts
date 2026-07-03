import { createI18n } from 'vue-i18n';
import en from './en-US';
import zh from './zh-CN';

export const i18n = createI18n({ legacy: false, locale: 'en-US', fallbackLocale: 'en-US', messages: { 'en-US': en, 'zh-CN': zh } });
