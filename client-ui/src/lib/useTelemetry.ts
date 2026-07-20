// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type { CommandType, Envelope } from "./types";

type Listener = (env: Envelope) => void;

/**
 * useTelemetry opens a single WebSocket to the Cloud Core gateway and exposes
 * a `send` for commands plus a subscription for inbound envelopes.
 */
export function useTelemetry(url: string) {
  const wsRef = useRef<WebSocket | null>(null);
  const listeners = useRef<Set<Listener>>(new Set());
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket(url);
    wsRef.current = ws;
    ws.onopen = () => setConnected(true);
    ws.onclose = () => setConnected(false);
    ws.onmessage = (ev) => {
      try {
        const env = JSON.parse(ev.data) as Envelope;
        listeners.current.forEach((l) => l(env));
      } catch {
        /* ignore malformed */
      }
    };
    return () => ws.close();
  }, [url]);

  const subscribe = useCallback((fn: Listener) => {
    listeners.current.add(fn);
    return () => {
      listeners.current.delete(fn);
    };
  }, []);

  const send = useCallback((type: CommandType, payload: unknown, nodeId?: string) => {
    const env: Envelope = {
      type,
      id: crypto.randomUUID(),
      ts: new Date().toISOString(),
      nodeId,
      payload,
    };
    wsRef.current?.send(JSON.stringify(env));
  }, []);

  return { connected, send, subscribe };
}
