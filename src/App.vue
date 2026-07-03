<template>
  <n-config-provider :theme="null">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <router-view />
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { usePullStore } from '@/stores/pull';
const pull = usePullStore();
onMounted(() => { pull.listen(); });
</script>
