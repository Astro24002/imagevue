<template>
  <AppShell>
    <n-space align="center" style="margin-bottom: 16px">
      <n-text strong>{{ connection?.name }}</n-text>
      <n-tag size="small">{{ connection?.kind }}</n-tag>
    </n-space>
    <n-space style="margin-bottom: 16px">
      <n-input v-model:value="query" :placeholder="t('registry.search')" clearable style="width: 360px" @keyup.enter="reload" />
      <n-button @click="reload">{{ t('registry.filter') }}</n-button>
    </n-space>
    <n-data-table :columns="columns" :data="repos" :loading="loading" :pagination="{ pageSize: 50 }" />
  </AppShell>
</template>

<script setup lang="ts">
import { onMounted, ref, h } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import AppShell from '@/components/AppShell.vue';
import { useConnectionsStore } from '@/stores/connections';
import { NInput, NButton, NSpace, NText, NTag, NDataTable } from 'naive-ui';
const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const store = useConnectionsStore();
const connection = ref<any>(null);
const query = ref('');
const repos = ref<{ name: string }[]>([]);
const loading = ref(false);
const columns = [{ title: t('registry.name'), key: 'name', render: (row: any) => h('a', { href: '#', onClick: (e: MouseEvent) => { e.preventDefault(); router.push(`/r/${route.params.id}/repo/${row.name}/tags`); } }, row.name) }];
onMounted(async () => { connection.value = store.items.find((c) => c.id === route.params.id) ?? await invoke('get_connection', { id: route.params.id }); await reload(); });
async function reload() { loading.value = true; try { repos.value = await invoke('list_repositories', { connectionId: route.params.id, query: query.value, limit: 200 }); } finally { loading.value = false; } }
</script>
