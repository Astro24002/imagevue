<template>
  <n-card :title="connection.name" hoverable>
    <template #header-extra>
      <n-tag :type="connection.lastConnectedAt ? 'success' : 'default'" size="small" round>
        {{ connection.lastConnectedAt ? '●' : '○' }}
      </n-tag>
    </template>
    <n-space vertical size="small">
      <n-text depth="3">{{ connection.kind }} · {{ connection.endpoint }}</n-text>
      <n-text v-if="connection.lastConnectedAt" depth="3">{{ t('connections.lastSeen') }}: {{ formatRelative(connection.lastConnectedAt) }}</n-text>
    </n-space>
    <template #action>
      <n-space>
        <n-button size="small" type="primary" @click="$emit('browse')">{{ t('connections.browse') }}</n-button>
        <n-button size="small" @click="$emit('test')">{{ t('connections.test') }}</n-button>
        <n-button size="small" @click="$emit('edit')">{{ t('connections.edit') }}</n-button>
        <n-popconfirm @positive-click="$emit('delete')">
          <template #trigger><n-button size="small" type="error">{{ t('connections.delete') }}</n-button></template>
          {{ t('connections.delete') }}?
        </n-popconfirm>
      </n-space>
    </template>
  </n-card>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { NCard, NTag, NText, NSpace, NButton, NPopconfirm } from 'naive-ui';
import type { RegistryConnection } from '@/stores/connections';
const { t } = useI18n();
defineProps<{ connection: RegistryConnection }>();
defineEmits<{ browse: []; edit: []; delete: []; test: [] }>();
function formatRelative(iso: string): string { const d = new Date(iso); return d.toLocaleString(); }
</script>
