// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

import { describe, expect, it, vi } from "vitest";
import { WebSocketServer, WebSocket } from "ws";
import { createCommandEnvelope } from "./useTelemetry";
import type { Envelope } from "./types";

// Integration test of the WebSocket command/telemetry contract: a ws server
// acts as the Cloud Core gateway. The client builds a command envelope with
// createCommandEnvelope, sends it, and the server echoes a telemetry envelope
// back which the client must parse into the expected shape.
describe("WebSocket contract", () => {
  it("client command -> server receives envelope, server telemetry -> client parses it", async () => {
    const server = new WebSocketServer({ port: 0 });
    const port: number = (server.address() as any).port;

    const receivedByServer: Envelope[] = [];
    let serverSocket: WebSocket | undefined;

    server.on("connection", (ws: WebSocket) => {
      serverSocket = ws;
      ws.on("message", (data: RawData) => {
        receivedByServer.push(JSON.parse(data.toString()));
        // Echo a telemetry envelope back, as the gateway would forward it.
        const telemetry: Envelope = {
          type: "telemetry.mount",
          id: "server-1",
          nodeId: "demo-node",
          ts: new Date().toISOString(),
          payload: { ra: 12.5, dec: -30, alt: 0, az: 0, tracking: true, status: "tracking" },
        };
        ws.send(JSON.stringify(telemetry));
      });
    });

    const client = new WebSocket(`ws://localhost:${port}`);
    const inbound: Envelope[] = [];
    const onMessage = vi.fn((env: Envelope) => inbound.push(env));

    await new Promise<void>((resolve, reject) => {
      client.on("open", () => resolve());
      client.on("error", reject);
    });

    const cmd = createCommandEnvelope("cmd.slew", { ra: 12.5, dec: -30, epoch: "J2000" }, "demo-node");
    client.send(JSON.stringify(cmd));

    // Wait for the server to receive the command and echo telemetry.
    await new Promise((r) => setTimeout(r, 150));

    expect(receivedByServer.length).toBe(1);
    expect(receivedByServer[0].type).toBe("cmd.slew");
    expect(receivedByServer[0].nodeId).toBe("demo-node");
    expect((receivedByServer[0].payload as any).ra).toBe(12.5);

    // Simulate the client-side onmessage handler used by useTelemetry.
    client.on("message", (data: RawData) => onMessage(JSON.parse(data.toString()) as Envelope));
    // Trigger one more echo.
    client.send(JSON.stringify(cmd));
    await new Promise((r) => setTimeout(r, 150));

    expect(onMessage).toHaveBeenCalled();
    const last = inbound[inbound.length - 1];
    expect(last.type).toBe("telemetry.mount");
    expect((last.payload as any).tracking).toBe(true);

    client.close();
    server.close();
  });
});

// RawData is the ws message payload type; declared locally to avoid importing
// the namespace just for a type annotation.
type RawData = Buffer | ArrayBuffer | Buffer[];
