<template>
  <n-layout has-sider style="height: 100vh">
    <n-layout-sider bordered :width="240" :collapsed-width="56" collapse-mode="width" show-trigger="bar">
      <div class="brand">{{ t('app.name') }}</div>
      <n-menu :options="menuOptions" :value="route.path" @update:value="onMenu" />
    </n-layout-sider>
    <n-layout>
      <n-layout-header bordered style="height: 56px; display: flex; align-items: center; padding: 0 16px; gap: 12px">
        <n-text strong>{{ pageTitle }}</n-text>
      </n-layout-header>
      <n-layout-content style="padding: 16px">
        <slot />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const pageTitle = computed(() => (route.meta?.title as string) ?? t('app.name'));
const menuOptions = computed(() => [
  { label: t('nav.connections'), key: '/connections' },
  { label: t('nav.settings'), key: '/settings' },
  { label: t('nav.about'), key: '/about' },
]);
function onMenu(key: string) { router.push(key); }
</script>

<style scoped>
.brand { padding: 16px; font-weight: 700; font-size: 16px; }
</style>
