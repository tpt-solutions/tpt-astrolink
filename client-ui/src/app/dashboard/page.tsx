// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

"use client";

import { useEffect, useState } from "react";
import { useTelemetry } from "../../lib/useTelemetry";
import { SkyView } from "../../components/SkyView";
import { ToONotifications } from "../../components/ToONotifications";
import { MultiTimezoneClocks } from "../../components/MultiTimezoneClocks";
import type { FocuserState, MountState, TooAlert, WeatherSample } from "../../lib/types";

const WS_URL = process.env.NEXT_PUBLIC_WS_URL ?? "ws://localhost:8080/ws";
const NODE_ID = process.env.NEXT_PUBLIC_NODE_ID ?? "demo-node";

export default function Dashboard() {
  const { connected, send, subscribe } = useTelemetry(WS_URL);
  const [mount, setMount] = useState<MountState>();
  const [focuser, setFocuser] = useState<FocuserState>();
  const [weather, setWeather] = useState<WeatherSample>();
  const [alerts, setAlerts] = useState<TooAlert[]>([]);
  const [ra, setRa] = useState("12.5");
  const [dec, setDec] = useState("-30.0");

  useEffect(() => {
    send("cmd.subscribe", {}, NODE_ID);
    return subscribe((env) => {
      switch (env.type) {
        case "telemetry.mount":
          setMount(env.payload as MountState);
          break;
        case "telemetry.focuser":
          setFocuser(env.payload as FocuserState);
          break;
        case "telemetry.weather":
          setWeather(env.payload as WeatherSample);
          break;
        case "alert.too":
          setAlerts((a) => [env.payload as TooAlert, ...a].slice(0, 10));
          break;
      }
    });
  }, [send, subscribe]);

  return (
    <main style={{ padding: 24, maxWidth: 1100, margin: "0 auto" }}>
      <header className="tpt-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: 8 }}>
        <h1 style={{ margin: 0 }}>TPT AstroLink — Control</h1>
        <div>
          <span>{connected ? "🟢 connected" : "🔴 offline"}</span>
          <MultiTimezoneClocks />
        </div>
      </header>

      <section className="tpt-grid">
        <div className="tpt-card">
          <h2>Mount</h2>
          <div className="tpt-row">
            <label>RA° <input className="tpt-input" value={ra} onChange={(e) => setRa(e.target.value)} /></label>
            <label>Dec° <input className="tpt-input" value={dec} onChange={(e) => setDec(e.target.value)} /></label>
          </div>
          <div className="tpt-row">
            <button className="tpt-btn" onClick={() => send("cmd.slew", { ra: +ra, dec: +dec, epoch: "J2000" }, NODE_ID)}>Slew</button>
            <button className="tpt-btn" onClick={() => send("cmd.slewStop", {}, NODE_ID)}>Stop</button>
          </div>
          {mount && <pre>{JSON.stringify(mount, null, 2)}</pre>}
        </div>

        <div className="tpt-card">
          <h2>Focuser</h2>
          <div className="tpt-row">
            <button className="tpt-btn" onClick={() => send("cmd.focus", { position: 10000 }, NODE_ID)}>Move to 10000</button>
            <button className="tpt-btn" onClick={() => send("cmd.focusRelative", { delta: 500 }, NODE_ID)}>+500</button>
          </div>
          {focuser && <pre>{JSON.stringify(focuser, null, 2)}</pre>}
        </div>

        <div className="tpt-card">
          <h2>Weather</h2>
          <div className="tpt-row">
            <button className="tpt-btn" onClick={() => send("cmd.weather.refresh", {}, NODE_ID)}>Refresh</button>
          </div>
          {weather && <pre>{JSON.stringify(weather, null, 2)}</pre>}
        </div>

        <div className="tpt-card">
          <h2>Imaging</h2>
          <div className="tpt-row">
            <button className="tpt-btn" onClick={() => send("cmd.imaging.start", { exposure: 30, gain: 100, bin: 1 }, NODE_ID)}>Start sequence</button>
            <button className="tpt-btn" onClick={() => send("cmd.imaging.stop", {}, NODE_ID)}>Stop</button>
          </div>
        </div>
      </section>

      <section style={{ marginTop: 16 }}>
        <h2>Sky</h2>
        <SkyView mount={mount} />
      </section>

      <ToONotifications alerts={alerts} />
    </main>
  );
}
