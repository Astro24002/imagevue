import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
import { useConnectionsStore } from '@/stores/connections';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: () => {
      const store = useConnectionsStore();
      if (store.items.length > 0 && store.activeId) {
        return `/r/${store.activeId}`;
      }
      if (store.items.length > 0 && !store.activeId) {
        store.setActive(store.items[0].id);
        return `/r/${store.items[0].id}`;
      }
      return '/welcome';
    },
  },
  { path: '/welcome', component: () => import('@/views/WelcomeView.vue') },
  { path: '/connections/:id/edit', component: () => import('@/views/ConnectionEditView.vue'), props: true },
  { path: '/r/:id', component: () => import('@/views/RegistryView.vue'), props: true },
  { path: '/r/:id/repo/:repoPath(.*)/tags', component: () => import('@/views/RepositoryView.vue'), props: true },
  { path: '/r/:id/repo/:repoPath(.*)/tag/:tag', component: () => import('@/views/TagDetailView.vue'), props: true },
  { path: '/settings', component: () => import('@/views/SettingsView.vue') },
  { path: '/about', component: () => import('@/views/AboutView.vue') },
];

export const router = createRouter({ history: createWebHistory(), routes });
