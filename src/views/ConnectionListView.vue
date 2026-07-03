<template>
  <AppShell>
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px">
      <h2>{{ t('connections.list') }}</h2>
      <n-button type="primary" @click="$router.push('/connections/new')">{{ t('connections.add') }}</n-button>
    </div>
    <n-spin :show="store.loading">
      <n-grid :cols="3" :x-gap="16" :y-gap="16" responsive="screen">
        <n-gi v-for="c in store.items" :key="c.id" :span="1">
          <ConnectionCard :connection="c" @browse="$router.push(`/r/${c.id}`)" @edit="$router.push(`/connections/${c.id}/edit`)" @delete="onDelete(c.id)" @test="onTest(c.id)" />
        </n-gi>
      </n-grid>
      <n-empty v-if="!store.loading && store.items.length === 0" :description="t('connections.list')" />
    </n-spin>
  </AppShell>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConnectionsStore } from '@/stores/connections';
import AppShell from '@/components/AppShell.vue';
import ConnectionCard from '@/components/ConnectionCard.vue';
import { useMessage } from 'naive-ui';
const { t } = useI18n();
const store = useConnectionsStore();
const message = useMessage();
onMounted(() => store.loadAll());
async function onDelete(id: string) { await store.remove(id); message.success('deleted'); }
async function onTest(id: string) { try { await store.test(id); message.success('ok'); } catch (e: any) { message.error(e?.message ?? 'failed'); } }
</script>
