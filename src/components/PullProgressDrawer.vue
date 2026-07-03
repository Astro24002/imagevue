<template>
  <n-drawer v-model:show="open" :width="380" placement="right">
    <n-drawer-content :title="t('pull.fetching')" closable>
      <n-tabs v-if="pull.jobs.length > 1" type="line" :value="active" @update:value="(v) => active = v">
        <n-tab-pane v-for="(j, i) in pull.jobs" :key="j.jobId" :name="i" :tab="`Job ${i + 1}`">
          <PullJob :event="j" />
        </n-tab-pane>
      </n-tabs>
      <PullJob v-else-if="pull.jobs[0]" :event="pull.jobs[0]" />
      <n-empty v-else :description="t('pull.fetching')" />
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import { ref, watch, computed, defineComponent, h } from 'vue';
import { storeToRefs } from 'pinia';
import { usePullStore } from '@/stores/pull';
import { useI18n } from 'vue-i18n';
import { NDrawer, NDrawerContent, NTabs, NTabPane, NEmpty, NProgress, NText, NSpace } from 'naive-ui';
const { t } = useI18n();
const pull = usePullStore();
const { jobs } = storeToRefs(pull);
const open = ref(false);
const active = ref(0);
watch(jobs, (j) => { if (j.length > 0 && !open.value) open.value = true; }, { deep: true });

const PullJob = defineComponent({
  props: { event: { type: Object, required: true } },
  setup(props) {
    const pct = computed(() => props.event.bytesTotal ? Math.floor((props.event.bytesDownloaded / props.event.bytesTotal) * 100) : 0);
    return () => h(NSpace, { vertical: true }, () => [
      h(NText, null, () => `Phase: ${props.event.phase}`),
      h(NProgress, { type: 'line', percentage: pct.value, indicatorPlacement: 'inside' }),
      h(NText, null, () => `${formatBytes(props.event.bytesDownloaded)} / ${formatBytes(props.event.bytesTotal)}`),
      h(NText, null, () => `Layer ${props.event.layerIndex}/${props.event.layerCount}`),
    ]);
  },
});
function formatBytes(n: number): string { if (!n) return '0 B'; const u = ['B','KB','MB','GB']; let i = 0; let v = n; while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; } return `${v.toFixed(1)} ${u[i]}`; }
</script>
