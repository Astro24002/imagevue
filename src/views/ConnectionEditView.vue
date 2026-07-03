<template>
  <AppShell>
    <h2>{{ isEdit ? t('connections.edit') : t('connections.add') }}</h2>
    <n-form :model="form" label-placement="left" label-width="120" style="max-width: 640px">
      <n-form-item :label="t('connections.name')">
        <n-input v-model:value="form.name" placeholder="My Registry" />
      </n-form-item>
      <n-form-item :label="t('connections.kind')">
        <n-select v-model:value="form.kind" :options="kindOptions" />
      </n-form-item>
      <n-form-item :label="t('connections.endpoint')">
        <n-input v-model:value="form.endpoint" placeholder="https://registry.example.com" />
      </n-form-item>
      <n-form-item v-if="form.kind === 'generic'" :label="t('connections.insecure')">
        <n-switch v-model:value="form.insecure" />
      </n-form-item>
      <n-form-item v-if="form.kind === 'generic'" label="Username">
        <n-input v-model:value="form.username" />
      </n-form-item>
      <n-form-item v-if="form.kind === 'generic'" label="Password">
        <n-input type="password" v-model:value="form.password" show-password-on="click" />
      </n-form-item>
      <n-form-item v-if="isOAuth(form.kind)" label="OAuth">
        <n-button @click="onOAuth">Login with browser</n-button>
      </n-form-item>
      <n-space>
        <n-button type="primary" @click="onSave" :disabled="!form.name || !form.endpoint">{{ t('connections.save') }}</n-button>
        <n-button @click="$router.push('/connections')">{{ t('connections.cancel') }}</n-button>
      </n-space>
    </n-form>
  </AppShell>
</template>

<script setup lang="ts">
import { onMounted, reactive, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import AppShell from '@/components/AppShell.vue';
import { useConnectionsStore } from '@/stores/connections';
import { useMessage } from 'naive-ui';
const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const message = useMessage();
const store = useConnectionsStore();

const form = reactive({ name: '', kind: 'dockerHub' as 'dockerHub'|'ghcr'|'quay'|'gcr'|'generic', endpoint: 'https://registry-1.docker.io', insecure: false, username: '', password: '' });
const isEdit = computed(() => !!route.params.id);
const kindOptions = [
  { label: 'Docker Hub', value: 'dockerHub' },
  { label: 'GHCR', value: 'ghcr' },
  { label: 'Quay', value: 'quay' },
  { label: 'GCR', value: 'gcr' },
  { label: 'Generic', value: 'generic' },
];
function isOAuth(k: string) { return k === 'ghcr' || k === 'quay' || k === 'gcr'; }

onMounted(async () => {
  if (isEdit.value) {
    const id = route.params.id as string;
    const c = store.items.find((x) => x.id === id) ?? await invoke('get_connection', { id });
    if (c) { form.name = c.name; form.kind = c.kind; form.endpoint = c.endpoint; form.insecure = c.insecure; }
  }
});

async function onSave() {
  await store.create({ ...form });
  message.success('saved');
  router.push('/connections');
}
async function onOAuth() {
  const session = await invoke<{ authUrl: string; state: string; codeVerifier: string }>('begin_oauth', { input: { kind: form.kind, connectionId: crypto.randomUUID() } });
  window.open(session.authUrl, '_blank');
  message.info('Complete login in browser, then return.');
}
</script>
