<template>
  <n-space align="center" style="margin-bottom: 16px">
    <n-button text @click="$router.back()">←</n-button>
    <n-text strong>{{ repoPath }}:{{ tag }}</n-text>
  </n-space>
  <n-grid :cols="2" :x-gap="16">
    <n-gi>
      <n-card :title="t('tag.manifest')">
        <vue-json-pretty :data="manifest?.rawJson ? JSON.parse(manifest.rawJson) : {}" />
      </n-card>
    </n-gi>
    <n-gi>
      <n-card :title="t('tag.config')">
        <n-skeleton v-if="!config" text :repeat="6" />
        <n-space vertical v-else>
          <div><n-text depth="3">OS/Arch:</n-text> {{ config.os }}/{{ config.architecture }}</div>
          <div v-if="config.cmd"><n-text depth="3">Cmd:</n-text> {{ config.cmd?.join(' ') }}</div>
          <div v-if="config.entrypoint"><n-text depth="3">Entrypoint:</n-text> {{ config.entrypoint?.join(' ') }}</div>
          <div v-if="config.env?.length"><n-text depth="3">Env:</n-text> {{ config.env?.join(', ') }}</div>
          <div v-if="Object.keys(config.labels ?? {}).length"><n-text depth="3">Labels:</n-text> {{ Object.entries(config.labels).map(([k,v]) => `${k}=${v}`).join(', ') }}</div>
        </n-space>
      </n-card>
    </n-gi>
  </n-grid>
  <n-card :title="t('tag.layers')" style="margin-top: 16px">
    <n-data-table :columns="layerColumns" :data="manifest?.layerDescriptors ?? []" :pagination="false" />
  </n-card>
  <n-space style="margin-top: 16px">
    <n-button type="primary" @click="onPull" :disabled="!manifest">
      {{ manifest?.artifactKind === 'helmChart' ? t('tag.pullChart') : t('tag.pull') }}
    </n-button>
  </n-space>
  <PullProgressDrawer />
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';
import PullProgressDrawer from '@/components/PullProgressDrawer.vue';
import { NCard, NButton, NText, NSpace, NDataTable, NSkeleton, NGrid, NGi, useMessage } from 'naive-ui';
const { t } = useI18n();
const route = useRoute();
const message = useMessage();
const repoPath = computed(() => Array.isArray(route.params.repoPath) ? (route.params.repoPath as string[]).join('/') : (route.params.repoPath as string));
const tag = computed(() => route.params.tag as string);
const manifest = ref<any>(null);
const config = ref<any>(null);
const layerColumns = [
  { title: 'Digest', key: 'digest', render: (row: any) => row.digest.substring(0, 16) },
  { title: 'Size', key: 'size', render: (row: any) => formatBytes(row.size) },
  { title: 'MediaType', key: 'mediaType' },
];
function formatBytes(n: number): string { if (!n) return '-'; const u = ['B','KB','MB','GB']; let i = 0; let v = n; while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; } return `${v.toFixed(1)} ${u[i]}`; }
onMounted(async () => {
  manifest.value = await invoke('get_manifest', { connectionId: route.params.id, repository: repoPath.value, reference: tag.value });
  if (manifest.value.configDescriptor && manifest.value.artifactKind !== 'helmChart') {
    config.value = await invoke('get_image_config', { connectionId: route.params.id, repository: repoPath.value, digest: manifest.value.configDescriptor.digest });
  }
});
async function onPull() {
  await invoke('start_pull', { input: { connectionId: route.params.id, repository: repoPath.value, tag: tag.value, outputDir: '~/Downloads', isChart: manifest.value.artifactKind === 'helmChart' } });
  message.success('Pull started');
}
</script>
