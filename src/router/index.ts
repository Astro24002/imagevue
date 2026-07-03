import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/connections' },
  { path: '/connections', component: () => import('@/views/ConnectionListView.vue') },
  { path: '/connections/new', component: () => import('@/views/ConnectionEditView.vue') },
  { path: '/connections/:id/edit', component: () => import('@/views/ConnectionEditView.vue'), props: true },
  { path: '/r/:id', component: () => import('@/views/RegistryView.vue'), props: true },
  { path: '/r/:id/repo/:repoPath(.*)/tags', component: () => import('@/views/RepositoryView.vue'), props: true },
  { path: '/r/:id/repo/:repoPath(.*)/tag/:tag', component: () => import('@/views/TagDetailView.vue'), props: true },
  { path: '/settings', component: () => import('@/views/SettingsView.vue') },
  { path: '/about', component: () => import('@/views/AboutView.vue') },
];

export const router = createRouter({ history: createWebHistory(), routes });
