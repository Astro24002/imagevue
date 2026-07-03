export default {
  app: { name: 'ImageVue', tagline: 'OCI registry viewer' },
  nav: { connections: 'Connections', settings: 'Settings', about: 'About' },
  connections: { list: 'Registry Connections', add: 'Add New', edit: 'Edit Connection', name: 'Display Name', kind: 'Type', endpoint: 'Endpoint', insecure: 'Allow insecure HTTP', test: 'Test Connection', save: 'Save', cancel: 'Cancel', delete: 'Delete', notLoggedIn: 'Not logged in', browse: 'Browse', lastSeen: 'Last seen' },
  registry: { search: 'Search repositories', limit: 'Limit', filter: 'Filter', name: 'Name', pulls: 'Pulls', stars: 'Stars', updated: 'Updated' },
  repository: { tags: 'Tags', search: 'Search tags', size: 'Size', os: 'OS/Arch' },
  tag: { detail: 'Tag Detail', digest: 'Digest', manifest: 'Manifest', config: 'Config', layers: 'Layers', pull: 'Pull', pullChart: 'Pull Chart' },
  pull: { resolving: 'Resolving manifest', fetching: 'Fetching layers', assembling: 'Assembling tar', writing: 'Writing file', completed: 'Completed', failed: 'Failed', cancelled: 'Cancelled' },
  settings: { title: 'Settings', defaultDownloadDir: 'Default download directory', maxConcurrent: 'Max concurrent layers', blobCache: 'Blob cache size (MB)', theme: 'Theme', language: 'Language' },
};
