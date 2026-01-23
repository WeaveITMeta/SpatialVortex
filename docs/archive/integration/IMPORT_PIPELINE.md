# Spatial Flux Import Pipeline

Front-end: SvelteKit  |  Back-end: Rust (Axum)  |  Data: RBXML / JSON / CSV  →  Bevy 3D Nodes

## Goal

- **Allow a user to click Import, pick a local file (RBXML/JSON/CSV), upload to a Rust server, and instantly render as colored 3D points in Bevy**.
- Each point is labeled with what it represents: `position`, `color`, and any extra `attributes`.

---

## High-Level Architecture

```mermaid
flowchart LR
    U[User (Browser)] -- choose file --> SK[SvelteKit UI]
    SK -- multipart/form-data --> AX[Axum /api/import]
    AX -- parse & map --> BUS[Node Bus (channel)]
    BUS -- spawn events --> BV[Bevy Runtime]
    BV -- 3D points + labels --> Window
    AX -- SSE/WebSocket progress --> SK
```

- **SvelteKit**: presents an Import button and uploads the chosen file via `multipart/form-data`.
- **Axum**: receives the upload, detects/reads format (RBXML/JSON/CSV), maps to a unified node schema, streams parsed nodes into a cross-thread channel (Node Bus).
- **Bevy**: a runtime in the same process consumes Node Bus events and spawns/updates 3D point entities with labels/colors in real time.
- **Feedback**: Axum emits progress via SSE/WebSockets so the UI can show parse/spawn progress.

---

## Unified Node Schema

All formats get normalized to this structure before Bevy spawn:

```json
{
  "id": "uuid-or-stable-id",
  "position": { "x": 0.0, "y": 0.0, "z": 0.0 },
  "color":    { "r": 0.1, "g": 0.6, "b": 0.9, "a": 1.0 },
  "label": "human-readable name",
  "attributes": { "any": "metadata", "key": 123 }
}
```

- Positions are in source units; optional normalization/scaling can be applied (see Mapping Rules).
- Color is RGBA in 0..1 floats. If not present, derive from attributes or defaults.
- `attributes` is a flat map for extra fields.

---

## File Format Mapping Rules

### 1) JSON

- Accept either:
  - Array of nodes: `[{ x, y, z, r, g, b, a, label, ...attrs }]`, or
  - Object with a field `nodes` containing the array.
- Mapping:
  - `x,y,z` → `position`
  - `r,g,b,a` (0..1 or 0..255) → `color`
  - `label` (optional) → `label`, else generate from id/index
  - All other fields copied to `attributes`.

### 2) CSV

- Expect a header row. Supported columns (case-insensitive):
  - Required: `x,y,z`
  - Optional: `r,g,b[,a]`, `label`
  - Arbitrary `attr_*` columns go into `attributes` (e.g., `attr_mass`, `attr_type`).
- Color values can be 0..1 or 0..255 (auto-detected by max component > 1 → divide by 255).

### 3) RBXML (XML)

- Read nodes from elements, e.g.:
  - `<Node id="..." label="..."><Position x="..." y="..." z="..."/><Color r="..." g="..." b="..." a="..."/> ...</Node>`
- Mapping:
  - `<Position/>` → `position`
  - `<Color/>` → `color`
  - Node attributes/elements → `attributes`
- Robustness: ignore unknown elements; collect as attributes if scalar.

### Mapping Options

- **Normalization**: optional transform (center + scale to unit cube/sphere).
- **Color derivation**: if color missing, map from `label` or `type` via a palette.
- **ID policy**: use provided IDs if unique; otherwise assign UUIDv4.

---

## API Design (Axum)

- `POST /api/import` (multipart)
  - Form fields:
    - `file`: the uploaded file
    - `format`: optional explicit format: `json|csv|rbxml` (auto-detect if missing)
    - `normalize`: optional `none|unit_sphere|unit_cube` (default `none`)
  - Headers:
    - `Authorization: Bearer <JWT>` (required)
  - Response: `{ job_id: string }` (start parsing asynchronously and streaming to Bevy)
  - Errors:
    - `401` if token missing/invalid; `403` if role lacks entitlement; `413` if size exceeds role limits.

- `GET /api/import/:job_id/status` (JSON)
  - Response: `{ state: "pending|parsing|rendering|done|error", parsed: number, spawned: number }`

- `GET /api/import/:job_id/events` (SSE)
  - Event stream: `progress`, `error`, `done`
  - Auth: same JWT; enforce job ownership or `admin` role.

Notes:
- For small files, the server can optionally respond only after Bevy spawn completes.
- Large files should stream progress; Bevy spawns entities incrementally to keep UI responsive.

---

## Bevy Integration

- **Runtime topology**:
  - Start Bevy app and Axum server in the same process.
  - Share a bounded channel (e.g., `crossbeam_channel` or `tokio::sync::mpsc`) as a Node Bus.
  - Axum parses and pushes `NodeEvent::Spawn(Node)` batches onto the bus.
  - A Bevy system drains the bus each frame and spawns/updates entities.

- **Entity layout**:
  - `PbrBundle` (sphere or instanced point) with `Transform::from_translation([x,y,z])`
  - `StandardMaterial` from `color`
  - `Name` = `label`
  - Optional: `Attributes` component holding a compact map for UI inspection

- **Labels**:
  - Use `bevy_text` (2D/3D) or an egui overlay to show hovered/selected node attributes.

- **Performance hints**:
  - Prefer instanced rendering (GPU) for large point clouds
  - Batch spawns in chunks (e.g., 1k per frame) to avoid hitches
  - Use a quadtree/octree structure for culling if needed later

---

## SvelteKit Front-End Flow

- **UI**:
  - `Import` button → `<input type="file" accept=".json,.csv,.xml,.rbxml">`
  - On selection, POST to `/api/import` with `FormData`
  - Subscribe to `/api/import/:job_id/events` (SSE) for progress
  - Display progress bar and a node count while Bevy spawns in the native window

- **Pseudocode**:

```ts
// +page.svelte
<input type="file" on:change={(e) => upload(e.target.files[0])} accept=".json,.csv,.xml,.rbxml" />

async function upload(file: File) {
  const data = new FormData();
  data.append('file', file);
  // optional: data.append('format', 'csv'); data.append('normalize', 'unit_sphere');
  const res = await fetch('/api/import', { method: 'POST', body: data });
  const { job_id } = await res.json();

  const sse = new EventSource(`/api/import/${job_id}/events`);
  sse.onmessage = (ev) => { /* update progress */ };
}
```

- **Dev setup**: SvelteKit 
  - Port 3000 (default). Configure CORS in Axum for local dev, or reverse-proxy through SvelteKit.

---

## Error Handling & Validation

- **Size limits**: enforce max upload size (e.g., 50 MB) to prevent OOM
- **Format guardrails**:
  - Reject if required fields/columns missing (JSON/CSV)
  - RBXML: validate required elements
- **Type checks**: coerce numbers; clamp colors to [0,1]
- **Logging**: structured logs with counts and first few errors for user feedback

---

## Security & Privacy

- **On-device only** in dev. In prod, host Axum and Bevy on a trusted machine.
- **No egress** from import handlers; never re-emit uploaded data externally.
- **Temporary files**: write to a sandbox dir; delete on completion.
- **CORS/CSRF**: tighten when front-end and back-end are split across origins.

---

## Authentication & RBAC (SSO/CRA)

Goal: Only authorized users may import; behavior (allowed formats, size limits, redaction) is governed by roles/claims.

- **SSO (OIDC)**
  - Front-end (SvelteKit) signs in with an IdP (e.g., Auth0/Azure AD/Cognito) using OIDC.
  - Attach `Authorization: Bearer <JWT>` to `POST /api/import` and status/events endpoints.
  - Token lifetime/refresh handled by SvelteKit; backend is stateless.

- **JWT verification in Axum**
  - Validate signature and claims using IdP JWKS (cache keys; rotate by `kid`).
  - Crates: `openidconnect` or `jsonwebtoken` + `reqwest` for JWKS fetch.
  - Required claims: `sub`, `exp`, `iat`, optional `aud`, `iss` match.
  - Role/permission claims: `roles`, `permissions`, or provider-specific namespace.

- **RBAC model**
  - Suggested roles:
    - `viewer` (read-only status/events)
    - `importer.basic` (CSV/JSON, size ≤ N MB)
    - `importer.rbxml` (RBXML enabled)
    - `admin` (all formats, higher limits)
  - Alternative CRA (claims/roles/attributes): combine roles with attributes like `tenant`, `department`, `data_class`.

- **Route guards (Axum)**
  - Middleware extracts/validates JWT and places `AuthContext { sub, roles, tenant, perms }` in request extensions.
  - Handlers enforce:
    - `POST /api/import`: requires `importer.basic` or `importer.rbxml` depending on `format`.
    - `GET /api/import/:job_id/*`: owner-only or `admin`, matched by `sub`/`tenant` on the job.

- **Entitlements at parse time**
  - Per role, configure:
    - Allowed formats: CSV/JSON/RBXML
    - Max file size / node count
    - Redaction rules (drop/obfuscate sensitive attributes)
  - Examples:
    - `importer.basic`: CSV/JSON only; redact columns matching `attr_*secret*`.
    - `importer.rbxml`: may ingest RBXML; preserve attributes, but mask `pii` keys.

- **Multi-tenant scoping**
  - Include `tenant` claim in job metadata and entity tags.
  - SSE events and status endpoints MUST check that the caller's `tenant` equals job `tenant`.

- **Auditing**
  - Log auth decisions (role, tenant, format, size), first errors, and job ownership.
  - Avoid logging raw data; log counts and hashes.

---

## Implementation Plan (incremental)

1. **Axum skeleton**: `POST /api/import` with multipart; echo back `{ job_id }`.
2. **Bevy plugin**: Node Bus + system that spawns spheres from `Node` batches.
3. **Parser adapters**: JSON → Node; CSV → Node; RBXML → Node.
4. **Progress**: emit SSE events while parsing/spawning.
5. **Normalization**: add optional center/scale.
6. **Labels & UI**: hover/inspect attributes, search by label.
7. **Performance**: instanced drawing, chunked spawns, culling if needed.

---

## Notes

- The project currently includes a Bevy viewer binary at `src/bin/vortex_view.rs`. We can:
  - Add a new server binary `src/bin/import_server.rs` that runs Axum + Bevy together, or
  - Create a dedicated `viewer/` crate for clearer separation.
- If you prefer Actix-Web (already present) over Axum, the same API shape applies; only handlers change.
