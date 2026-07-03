import { defineStore } from 'pinia';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface PullProgressEvent {
  jobId: string;
  phase: string;
  bytesDownloaded: number;
  bytesTotal: number;
  currentLayer: string | null;
  layerIndex: number;
  layerCount: number;
  speedBps: number;
}

export const usePullStore = defineStore('pull', {
  state: () => ({ jobs: [] as PullProgressEvent[], unlisten: null as UnlistenFn | null }),
  actions: {
    async listen() {
      if (this.unlisten) return;
      this.unlisten = await listen<PullProgressEvent>('pull://progress', (e) => {
        const idx = this.jobs.findIndex((j) => j.jobId === e.payload.jobId);
        if (idx >= 0) this.jobs[idx] = e.payload; else this.jobs.push(e.payload);
      });
    },
  },
});
