<template>
  <n-space align="center" style="margin-bottom: 16px">
    <n-button text @click="$router.back()">←</n-button>
    <n-text strong>{{ repoPath }}</n-text>
  </n-space>
  <n-data-table :columns="columns" :data="tags" :loading="loading" :pagination="{ pageSize: 50 }" />
</template>

<script setup lang="ts">
import { onMounted, ref, h, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { NButton, NText, NDataTable, NSpace, NTag } from 'naive-ui';
const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const repoPath = computed(() => Array.isArray(route.params.repoPath) ? (route.params.repoPath as string[]).join('/') : (route.params.repoPath as string));
const tags = ref<any[]>([]);
const loading = ref(false);
const columns = [
  { title: 'Tag', key: 'name', render: (row: any) => h('a', { href: '#', onClick: (e: MouseEvent) => { e.preventDefault(); router.push(`/r/${route.params.id}/repo/${repoPath.value}/tag/${row.name}`); } }, row.name) },
  { title: t('repository.size'), key: 'size', render: (row: any) => formatBytes(row.size) },
  { title: t('repository.os'), key: 'os', render: (row: any) => row.os ? `${row.os}/${row.architecture ?? ''}` : '-' },
  { title: 'Kind', key: 'artifactKind', render: (row: any) => h(NTag, { size: 'small', type: row.artifactKind === 'helmChart' ? 'info' : 'default' }, () => row.artifactKind) },
];
function formatBytes(n: number): string { if (!n) return '-'; const u = ['B','KB','MB','GB']; let i = 0; let v = n; while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; } return `${v.toFixed(1)} ${u[i]}`; }
onMounted(async () => { loading.value = true; try { tags.value = await invoke('list_tags', { connectionId: route.params.id, repository: repoPath.value }); } finally { loading.value = false; } });
</script>
