<template>
  <div style="min-width: 500px">
    <div class="flex items-center justify-between" style="padding: 8px 12px; gap: 8px">
      <n-button size="small" type="primary" @click="showDialog = true">{{ t('connections.add') }}</n-button>
      <n-input v-model:value="filter" :placeholder="t('registry.search')" size="small" clearable style="width: 200px" />
    </div>
    <n-data-table
      :columns="columns"
      :data="filtered"
      :row-props="rowProps"
      size="small"
      :max-height="360"
      :bordered="false"
    />
  </div>
  <RegistryEditDialog v-model:show="showDialog" />
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { useConnectionsStore, type RegistryConnection } from '@/stores/connections';
import { NButton, NPopconfirm, NSpace, NTag } from 'naive-ui';
import RegistryEditDialog from './RegistryEditDialog.vue';

const emit = defineEmits<{ select: [id: string] }>();
const { t } = useI18n();
const router = useRouter();
const store = useConnectionsStore();
const filter = ref('');
const showDialog = ref(false);

const filtered = computed(() => {
  if (!filter.value) return store.items;
  const q = filter.value.toLowerCase();
  return store.items.filter((c) => c.name.toLowerCase().includes(q) || c.endpoint.toLowerCase().includes(q));
});

const columns = [
  {
    title: '',
    key: 'status',
    width: 40,
    render: (row: RegistryConnection) => {
      const tag = h(NTag, { size: 'tiny', type: row.lastConnectedAt ? 'success' : 'default', round: true }, { default: () => '●' });
      return tag;
    },
  },
  {
    title: t('connections.name'),
    key: 'name',
    render: (row: RegistryConnection) => {
      const children: any[] = [row.name];
      if (store.activeId === row.id) {
        children.push(h(NTag, { size: 'tiny', type: 'primary', style: 'margin-left: 6px' }, { default: () => 'active' }));
      }
      return h('span', null, children);
    },
  },
  {
    title: t('connections.kind'),
    key: 'kind',
    width: 100,
  },
  {
    title: t('connections.endpoint'),
    key: 'endpoint',
    ellipsis: true,
  },
  {
    title: '',
    key: 'actions',
    width: 100,
    render: (row: RegistryConnection) => h(NSpace, { size: 'small' }, {
      default: () => [
        h(NButton, {
          size: 'tiny', quaternary: true,
          onClick: (e: MouseEvent) => { e.stopPropagation(); onEdit(row.id); },
        }, { default: () => t('connections.edit') }),
        h(NPopconfirm, {
          onPositiveClick: (e: MouseEvent) => { e?.stopPropagation(); onDelete(row.id); },
        }, {
          trigger: () => h(NButton, { size: 'tiny', quaternary: true, type: 'error' }, { default: () => t('connections.delete') }),
          default: () => `${t('connections.delete')}?`,
        }),
      ],
    }),
  },
];

function rowProps(row: RegistryConnection) {
  return {
    style: 'cursor: pointer',
    onClick: () => emit('select', row.id),
  };
}

async function onDelete(id: string) {
  await store.remove(id);
}

function onEdit(id: string) {
  router.push(`/connections/${id}/edit`);
}
</script>
