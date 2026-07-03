<template>
  <n-popover trigger="click" placement="bottom" :width="600" @update:show="onToggle">
    <template #trigger>
      <n-button size="small" ghost data-testid="registry-selector">
        <template #icon>
          <n-tag v-if="store.active" :type="store.active.lastConnectedAt ? 'success' : 'warning'" size="small" round>●</n-tag>
        </template>
        <span style="max-width: 180px" class="ellipsis">{{ store.active?.name ?? t('connections.notLoggedIn') }}</span>
        <template #suffix>
          <n-icon><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="16" height="16"><path d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6z"/></svg></n-icon>
        </template>
      </n-button>
    </template>
    <RegistryTable @select="onSelect" />
  </n-popover>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { useConnectionsStore } from '@/stores/connections';
import RegistryTable from './RegistryTable.vue';

const { t } = useI18n();
const router = useRouter();
const store = useConnectionsStore();

function onSelect(id: string) {
  store.setActive(id);
  router.push(`/r/${id}`);
}

function onToggle(show: boolean) {
  if (show) store.loadAll();
}
</script>
