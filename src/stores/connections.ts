import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export interface RegistryConnection {
  id: string;
  name: string;
  kind: 'dockerHub' | 'ghcr' | 'quay' | 'gcr' | 'generic';
  endpoint: string;
  insecure: boolean;
  credentialRef: string | null;
  createdAt: string;
  lastConnectedAt: string | null;
}

export const useConnectionsStore = defineStore('connections', {
  state: () => ({
    items: [] as RegistryConnection[],
    activeId: null as string | null,
    loading: false,
  }),
  getters: {
    active: (state) => state.items.find((c) => c.id === state.activeId) ?? null,
    hasActive: (state) => state.items.some((c) => c.id === state.activeId),
  },
  actions: {
    async loadAll() {
      this.loading = true;
      try {
        this.items = await invoke<RegistryConnection[]>('list_connections');
        if (this.activeId && !this.items.some((c) => c.id === this.activeId)) {
          this.activeId = null;
        }
      } finally {
        this.loading = false;
      }
    },
    async create(input: any) {
      const c = await invoke<RegistryConnection>('create_connection', { input });
      this.items.unshift(c);
      this.activeId = c.id;
      return c;
    },
    async remove(id: string) {
      await invoke('delete_connection', { id });
      this.items = this.items.filter((c) => c.id !== id);
      if (this.activeId === id) {
        this.activeId = this.items[0]?.id ?? null;
      }
    },
    async test(id: string) {
      await invoke('test_connection', { id });
      await this.loadAll();
    },
    setActive(id: string | null) {
      this.activeId = id;
    },
  },
});
