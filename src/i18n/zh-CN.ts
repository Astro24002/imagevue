export default {
  app: { name: 'ImageVue', tagline: 'OCI 仓库查看器' },
  nav: { connections: '连接', settings: '设置', about: '关于' },
  connections: { list: 'Registry 连接', add: '新增', edit: '编辑连接', name: '显示名', kind: '类型', endpoint: '端点', insecure: '允许 HTTP（不推荐）', test: '测试连接', save: '保存', cancel: '取消', delete: '删除', notLoggedIn: '未登录', browse: '浏览', lastSeen: '最后连接' },
  registry: { search: '搜索仓库', limit: '数量', filter: '过滤', name: '名称', pulls: '拉取数', stars: 'Star', updated: '更新于' },
  repository: { tags: '标签', search: '搜索标签', size: '大小', os: '系统/架构' },
  tag: { detail: '标签详情', digest: '摘要', manifest: 'Manifest', config: 'Config', layers: '层', pull: '拉取', pullChart: '拉取 Chart' },
  pull: { resolving: '解析 manifest', fetching: '下载层', assembling: '组装 tar', writing: '写入文件', completed: '完成', failed: '失败', cancelled: '已取消' },
  settings: { title: '设置', defaultDownloadDir: '默认下载目录', maxConcurrent: '最大并发层数', blobCache: 'Blob 缓存（MB）', theme: '主题', language: '语言' },
};
