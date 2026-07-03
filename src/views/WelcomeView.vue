<template>
  <div style="display: flex; align-items: center; justify-content: center; height: calc(100vh - 88px)">
    <n-card style="max-width: 480px; text-align: center">
      <n-h2>{{ t('app.name') }}</n-h2>
      <n-text depth="3">{{ t('app.tagline') }}</n-text>
      <div style="margin-top: 24px">
        <n-button type="primary" size="large" @click="showAdd = true">{{ t('connections.add') }}</n-button>
      </div>
    </n-card>
  </div>
  <RegistryEditDialog v-model:show="showAdd" />
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import RegistryEditDialog from '@/components/RegistryEditDialog.vue';

const { t } = useI18n();
const showAdd = ref(false);
</script>
