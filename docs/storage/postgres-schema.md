# Postgres Schema — TPT Cloud Core

**Owner:** TPT Cloud Core (Go) · **Spec ref:** Phase 1, Storage.

PostgreSQL 16. Times stored as `timestamptz` (UTC). Coordinates in degrees
(J2000 unless noted). Enums as `text` with CHECK constraints for portability.

## Tables

### users

```sql
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT NOT NULL UNIQUE,
    display_name  TEXT NOT NULL,
    role          TEXT NOT NULL DEFAULT 'observer'
                    CHECK (role IN ('observer','operator','admin','researcher')),
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### nodes (Edge Agents)

```sql
CREATE TABLE nodes (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    hardware      TEXT NOT NULL CHECK (hardware IN ('raspberry-pi-5','intel-nuc','other')),
    location      TEXT,
    latitude      DOUBLE PRECISION CHECK (latitude BETWEEN -90 AND 90),
    longitude     DOUBLE PRECISION CHECK (longitude BETWEEN -180 AND 180),
    altitude_m    DOUBLE PRECISION,
    timezone      TEXT,                       -- IANA tz, e.g. 'Pacific/Auckland'
    status        TEXT NOT NULL DEFAULT 'offline'
                    CHECK (status IN ('offline','online','error','maintenance')),
    firmware_ver  TEXT,
    last_seen     TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (owner_id, name)
);
```

### targets

```sql
CREATE TABLE targets (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          TEXT NOT NULL,
    ra            DOUBLE PRECISION NOT NULL CHECK (ra BETWEEN 0 AND 360),
    dec           DOUBLE PRECISION NOT NULL CHECK (dec BETWEEN -90 AND 90),
    epoch         TEXT NOT NULL DEFAULT 'J2000',
    priority      INT NOT NULL DEFAULT 0,
    constraints   JSONB,                      -- { "minAlt": 30, "maxMoon": 50, ... }
    created_by    UUID REFERENCES users(id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### observations

```sql
CREATE TABLE observations (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id       UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    target_id     UUID REFERENCES targets(id),
    operator_id   UUID REFERENCES users(id),
    started_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at      TIMESTAMPTZ,
    status        TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','capturing','solved','stitched','failed')),
    exposure_s    DOUBLE PRECISION,
    gain          INT,
    bin           INT,
    frame_count   INT NOT NULL DEFAULT 0
);
```

### metadata (astrometry / plate-solve results)

```sql
CREATE TABLE metadata (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    observation_id  UUID NOT NULL REFERENCES observations(id) ON DELETE CASCADE,
    object_key      TEXT NOT NULL,            -- S3 key of the FITS object
    ra_center       DOUBLE PRECISION CHECK (ra_center BETWEEN 0 AND 360),
    dec_center      DOUBLE PRECISION CHECK (dec_center BETWEEN -90 AND 90),
    fov_w_deg       DOUBLE PRECISION,
    fov_h_deg       DOUBLE PRECISION,
    orientation_deg DOUBLE PRECISION,
    solved_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (observation_id, object_key)
);

CREATE INDEX idx_metadata_obs ON metadata(observation_id);
CREATE INDEX idx_observations_node ON observations(node_id);
CREATE INDEX idx_observations_target ON observations(target_id);
CREATE INDEX idx_nodes_owner ON nodes(owner_id);
```

## Notes

- `metadata.object_key` links to S3 (see `docs/storage/s3-layout.md`).
- Transient / Target-of-Opportunity alerts reference an `observation_id` +
  a separate `alerts` table (TBD in Phase 2/Edge AI).
- Migrations managed via Goose / golang-migrate (TBD Phase 7).
