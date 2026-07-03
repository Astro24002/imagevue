<template>
  <n-config-provider :theme="null">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <n-spin :show="loading">
            <AppShell v-if="!loading" />
          </n-spin>
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { usePullStore } from '@/stores/pull';
import { useConnectionsStore } from '@/stores/connections';

const loading = ref(true);
const pull = usePullStore();
const store = useConnectionsStore();
const router = useRouter();

onMounted(async () => {
  pull.listen();
  await store.loadAll();

  if (store.items.length > 0 && !store.activeId) {
    store.setActive(store.items[0].id);
  }

  loading.value = false;

  if (router.currentRoute.value.path === '/') {
    if (store.activeId) {
      router.replace(`/r/${store.activeId}`);
    } else {
      router.replace('/welcome');
    }
  }
});
</script>
