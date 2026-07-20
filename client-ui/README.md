# TPT Client UI (Next.js)

Part of TPT AstroLink — Project Cosmos. Licensed MIT OR Apache-2.0.

The Client UI is a TypeScript / React / Next.js dashboard with a Three.js 3D
sky view, real-time telemetry over WebSocket, mount/focuser/weather controls,
imaging triggers, and Target-of-Opportunity alert notifications.

## Env
- `NEXT_PUBLIC_WS_URL` — Cloud Core WebSocket URL (default `ws://localhost:8080/ws`)
- `NEXT_PUBLIC_NODE_ID` — default Edge Node id

## Develop
```
npm install
npm run dev
```

## Structure (`src/`)
- `app/` — pages (`/`, `/dashboard`)
- `components/` — `SkyView`, `ToONotifications`
- `lib/` — `useTelemetry` WebSocket hook, `types`
