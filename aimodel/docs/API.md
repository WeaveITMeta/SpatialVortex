# Vortex API — Developer Guide

OpenAI-compatible REST API with API-key auth and per-token usage billing.

## Quick Start (self-hosted)

```bash
# From the repo root
docker compose -f aimodel/docker-compose.yml up -d --build

# Or bare metal
cd aimodel
VORTEX_HOST=0.0.0.0 cargo run --release --bin vortex-api --features web
```

On first boot, if `VORTEX_ADMIN_KEY` is not set, an admin key is generated,
printed to the logs, and persisted in the key store file. Grab it:

```bash
docker compose -f aimodel/docker-compose.yml logs vortex-api | grep sk-vortex-admin
```

## Authentication

All `/v1/*` endpoints (except `/health`) require a Bearer key:

```
Authorization: Bearer sk-vortex-<key>
```

### Create a developer key (admin)

```bash
curl -X POST http://localhost:7000/admin/keys \
  -H "Authorization: Bearer $VORTEX_ADMIN_KEY" \
  -H "Content-Type: application/json" \
  -d '{"label":"my-app"}'
# => {"api_key":"sk-vortex-...","label":"my-app", ...}
```

### Revoke a key (admin)

```bash
curl -X DELETE http://localhost:7000/admin/keys/sk-vortex-... \
  -H "Authorization: Bearer $VORTEX_ADMIN_KEY"
```

## Chat Completions

Drop-in replacement for the OpenAI chat endpoint — point any OpenAI SDK at
`base_url = "http://your-host:7000/v1"`.

```bash
curl -X POST http://localhost:7000/v1/chat/completions \
  -H "Authorization: Bearer sk-vortex-..." \
  -H "Content-Type: application/json" \
  -d '{
    "model": "vortex-0.1",
    "messages": [{"role": "user", "content": "What is 48 / 2 + 12?"}]
  }'
```

```python
from openai import OpenAI
client = OpenAI(base_url="http://localhost:7000/v1", api_key="sk-vortex-...")
resp = client.chat.completions.create(
    model="vortex-0.1",
    messages=[{"role": "user", "content": "What is 48 / 2 + 12?"}],
)
print(resp.choices[0].message.content)
```

Responses include the standard `usage` block plus a `vortex` extension with
confidence, reasoning trace, and the request's cost in USD:

```json
{
  "choices": [...],
  "usage": {"prompt_tokens": 7, "completion_tokens": 12, "total_tokens": 19},
  "vortex": {"confidence": 0.91, "cost_usd": 0.0000055, "...": "..."}
}
```

## Pricing & Usage

Pricing is configured server-side in USD per 1M tokens:

| Env var | Default | Meaning |
|---------|---------|---------|
| `VORTEX_PRICE_INPUT_PER_M` | `0.10` | $/1M input (prompt) tokens |
| `VORTEX_PRICE_OUTPUT_PER_M` | `0.40` | $/1M output (completion) tokens |

Inference is CPU-only with no pretrained weights, so hosting cost is a small
VPS — the defaults are intentionally far below GPU-backed model pricing.
Tokens are counted by the engine's whitespace tokenizer (documented, simple,
deterministic).

Check usage and accumulated cost for your key:

```bash
curl http://localhost:7000/v1/usage -H "Authorization: Bearer sk-vortex-..."
```

```json
{
  "label": "my-app",
  "requests": 1042,
  "prompt_tokens": 31337,
  "completion_tokens": 88210,
  "cost_usd": 0.0384,
  "pricing": {"input_per_million_usd": 0.1, "output_per_million_usd": 0.4}
}
```

Admins can list all keys (redacted) with usage: `GET /admin/keys`.

## Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/v1/chat/completions` | key | OpenAI-compatible chat |
| GET | `/v1/models` | — | List models |
| GET | `/v1/usage` | key | Usage + cost for the calling key |
| GET | `/health` | — | Health/liveness check |
| POST | `/admin/keys` | admin | Create developer key |
| GET | `/admin/keys` | admin | List keys + usage |
| DELETE | `/admin/keys/{key}` | admin | Revoke a key |

## Operational Notes

- **Key store**: a single JSON file (`VORTEX_KEYS_PATH`, default
  `vortex_keys.json`). Keys are stored in plaintext — restrict file
  permissions to the service user, and mount it on a persistent volume in
  Docker. Suitable for self-hosted single-node deployments; swap in a
  database before multi-node scale-out.
- **TLS**: terminate HTTPS at a reverse proxy (Caddy/nginx/Traefik) in front
  of the service. Do not expose port 7000 directly to the internet without it.
- **Rate limiting**: not built in yet; enforce at the reverse proxy if needed.
- **Streaming**: `stream: true` is accepted but responses are returned whole.
