<template>
  <n-modal v-model:show="show" preset="card" :title="t('connections.add')" style="width: 520px">
    <n-form :model="form" label-placement="left" label-width="100">
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
      <n-space justify="end" style="margin-top: 16px">
        <n-button @click="show = false">{{ t('connections.cancel') }}</n-button>
        <n-button type="primary" @click="onSave" :disabled="!form.name || !form.endpoint">{{ t('connections.save') }}</n-button>
      </n-space>
    </n-form>
  </n-modal>
</template>

<script setup lang="ts">
import { reactive, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { useConnectionsStore } from '@/stores/connections';
import { useMessage } from 'naive-ui';

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ 'update:show': [v: boolean] }>();

const { t } = useI18n();
const router = useRouter();
const message = useMessage();
const store = useConnectionsStore();

const show = computed({
  get: () => props.show,
  set: (v: boolean) => emit('update:show', v),
});

const kindOptions = [
  { label: 'Docker Hub', value: 'dockerHub' },
  { label: 'GHCR', value: 'ghcr' },
  { label: 'Quay', value: 'quay' },
  { label: 'GCR', value: 'gcr' },
  { label: 'Generic', value: 'generic' },
];

const form = reactive({
  name: '', kind: 'dockerHub' as 'dockerHub'|'ghcr'|'quay'|'gcr'|'generic',
  endpoint: 'https://registry-1.docker.io', insecure: false, username: '', password: '',
});

async function onSave() {
  const c = await store.create({ ...form });
  message.success('saved');
  show.value = false;
  emit('update:show', false);
  router.push(`/r/${c.id}`);
}
</script>
