---
layout: home

hero:
  name: Pigeon
  text: Webhook Delivery Service
  tagline: Send events, Pigeon delivers them. Self-hosted, multi-tenant, with signing secret rotation and automatic retries.
  image:
    src: /pigeon/pigeon.svg
    alt: Pigeon
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: API Reference
      link: /api/overview
    - theme: alt
      text: GitHub
      link: https://github.com/The127/pigeon

features:
  - title: Fan-out Delivery
    details: Send a message once, Pigeon delivers it to every subscribed endpoint with HMAC-SHA256 signatures.
  - title: Automatic Retries
    details: Exponential backoff, dead lettering after exhaustion, replay from the UI. No messages lost.
  - title: Signing Secret Rotation
    details: Pigeon-generated secrets with zero-downtime rotation. Dual-secret window so consumers verify against either.
  - title: Multi-tenant
    details: OIDC-based organization isolation. Every query is scoped by org_id at the SQL level.
  - title: Transactional Outbox
    details: Domain events committed in the same transaction as the data change. No dual-write problems.
  - title: Full UI
    details: Vue 3 dashboard with stats, delivery charts, dead letter management, audit log, and theme customization.
---
