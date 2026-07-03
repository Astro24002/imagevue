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
  state: () => ({ items: [] as RegistryConnection[], activeId: null as string | null, loading: false }),
  actions: {
    async loadAll() { this.loading = true; try { this.items = await invoke<RegistryConnection[]>('list_connections'); } finally { this.loading = false; } },
    async create(input: any) { const c = await invoke<RegistryConnection>('create_connection', { input }); this.items.unshift(c); return c; },
    async remove(id: string) { await invoke('delete_connection', { id }); this.items = this.items.filter((c) => c.id !== id); },
    async test(id: string) { await invoke('test_connection', { id }); await this.loadAll(); },
    setActive(id: string | null) { this.activeId = id; },
  },
});
