import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Pigeon',
  description: 'Self-hosted webhook delivery service',
  base: '/pigeon/',

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/pigeon/pigeon.svg' }],
  ],

  themeConfig: {
    logo: '/pigeon.svg',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'API', link: '/api/overview' },
      { text: 'GitHub', link: 'https://github.com/The127/pigeon' },
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Architecture', link: '/guide/architecture' },
          { text: 'Configuration', link: '/guide/configuration' },
          { text: 'Deployment', link: '/guide/deployment' },
        ],
      },
      {
        text: 'Features',
        items: [
          { text: 'Webhook Delivery', link: '/features/delivery' },
          { text: 'Signing Secrets', link: '/features/signing-secrets' },
          { text: 'Multitenancy', link: '/features/multitenancy' },
        ],
      },
      {
        text: 'API Reference',
        items: [
          { text: 'Overview', link: '/api/overview' },
          { text: 'Applications', link: '/api/applications' },
          { text: 'Event Types', link: '/api/event-types' },
          { text: 'Endpoints', link: '/api/endpoints' },
          { text: 'Messages', link: '/api/messages' },
          { text: 'Dead Letters', link: '/api/dead-letters' },
        ],
      },
      {
        text: 'Development',
        items: [
          { text: 'Local Setup', link: '/development/local-setup' },
          { text: 'Contributing', link: '/development/contributing' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/The127/pigeon' },
    ],

    footer: {
      message: 'Released under the AGPL-3.0 License.',
    },

    search: {
      provider: 'local',
    },
  },
})
