<template>
  <n-layout style="height: 100vh">
    <n-layout-header bordered style="height: 56px; display: flex; align-items: center; padding: 0 16px; gap: 12px">
      <n-text strong style="font-size: 16px; margin-right: 8px">{{ t('app.name') }}</n-text>
      <RegistrySelector />
      <div style="flex: 1" />
      <n-button v-if="store.items.length === 0" size="small" type="primary" @click="showAdd = true">{{ t('connections.add') }}</n-button>
    </n-layout-header>
    <n-layout-content style="padding: 16px">
      <router-view />
    </n-layout-content>
  </n-layout>
  <RegistryEditDialog v-model:show="showAdd" />
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConnectionsStore } from '@/stores/connections';
import RegistrySelector from './RegistrySelector.vue';
import RegistryEditDialog from './RegistryEditDialog.vue';

const { t } = useI18n();
const store = useConnectionsStore();
const showAdd = ref(false);
</script>
