<template>
  <AppShell>
    <h2>ImageVue</h2>
    <p>{{ t('app.tagline') }}</p>
    <n-text depth="3">Version 0.1.0</n-text>
  </AppShell>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import AppShell from '@/components/AppShell.vue';
import { NText } from 'naive-ui';
const { t } = useI18n();
</script>
