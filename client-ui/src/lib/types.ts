// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Types mirroring docs/protocols/websocket-contract.md.

export type Envelope<T = unknown> = {
  type: string;
  id?: string;
  ts?: string;
  nodeId?: string;
  payload: T;
};

export type MountState = {
  ra: number;
  dec: number;
  alt: number;
  az: number;
  tracking: boolean;
  status: "idle" | "slewing" | "tracking" | "error";
};

export type FocuserState = {
  position: number;
  temperature: number;
};

export type WeatherSample = {
  temp: number;
  humidity: number;
  pressure: number;
  windSpeed: number;
  dewPoint: number;
  cloudCover: number;
};

export type TooAlert = {
  objectId: string;
  ra: number;
  dec: number;
  magDelta: number;
  confidence: number;
  imageKey: string;
};

export type CommandType =
  | "cmd.slew"
  | "cmd.slewStop"
  | "cmd.focus"
  | "cmd.focusRelative"
  | "cmd.weather.refresh"
  | "cmd.imaging.start"
  | "cmd.imaging.stop"
  | "cmd.subscribe";
