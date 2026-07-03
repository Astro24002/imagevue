<template>
  <h2>{{ t('settings.title') }}</h2>
  <n-form label-placement="left" label-width="200" style="max-width: 600px">
    <n-form-item :label="t('settings.defaultDownloadDir')"><n-input v-model:value="form.defaultDownloadDir" /></n-form-item>
    <n-form-item :label="t('settings.maxConcurrent')"><n-input-number v-model:value="form.maxConcurrentLayers" :min="1" :max="16" /></n-form-item>
    <n-form-item :label="t('settings.blobCache')"><n-input-number v-model:value="form.blobCacheSizeMb" :min="256" :max="8192" /></n-form-item>
    <n-form-item :label="t('settings.theme')"><n-select v-model:value="form.theme" :options="themeOptions" /></n-form-item>
    <n-form-item :label="t('settings.language')"><n-select v-model:value="form.language" :options="langOptions" /></n-form-item>
  </n-form>
</template>

<script setup lang="ts">
import { reactive } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
const form = reactive({ defaultDownloadDir: '~/Downloads', maxConcurrentLayers: 4, blobCacheSizeMb: 2048, theme: 'auto', language: 'en-US' });
const themeOptions = [{ label: 'Auto', value: 'auto' }, { label: 'Light', value: 'light' }, { label: 'Dark', value: 'dark' }];
const langOptions = [{ label: 'English', value: 'en-US' }, { label: '简体中文', value: 'zh-CN' }];
</script>
