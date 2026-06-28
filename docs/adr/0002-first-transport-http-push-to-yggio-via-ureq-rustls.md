# 0002 — First transport: HTTP push to yggio via ureq + rustls

- Status: accepted
- Date: 2026-06-28

## Context

ADR 0001 frames telltaled as reporting over an *arbitrary* transport, but the MVP
needs one concrete transport to carry its first real signal off-host. The target
sink is yggio (the work platform) on `https://public.yggio.net`. yggio accepts
device data over several protocols; the two relevant to a daemon are its generic
HTTP push endpoint and MQTT.

- **HTTP push** (`POST /http-push/generic?identifier=secret`) is stateless: a
  per-host `secret` embedded in the JSON body is the only credential, with no
  Authorization header and no broker session. Provisioning is one API/UI call to
  create the device.
- **MQTT** needs a persistent broker connection plus a `basicCredentialsSet` and a
  reserved topic provisioned ahead of time.

The overriding constraint is low host overhead. The MVP samples a single signal at
a slow cadence, where a short-lived POST is trivially cheap and holding a resident
MQTT/TLS session is pure standing cost. The endpoint is HTTPS, so a TLS-capable
client is required regardless.

## Decision

We will make HTTP push the MVP's first concrete transport: one `POST` to
`/http-push/generic?identifier=secret` per sample, body `{"secret": ..., <signal
fields>}`, `content-type: application/json`.

We will implement it with the **`ureq`** blocking HTTP client over the **rustls**
TLS backend, using **`webpki-roots`** (bundled Mozilla roots) for trust — not
`reqwest` (async) and not native-tls/OpenSSL.

## Consequences

- The daemon stays a synchronous `sample → POST → sleep` loop with no async
  runtime; `reqwest`/`tokio` are ruled out for the MVP as overhead with nothing to
  await.
- rustls + webpki-roots keeps the binary self-contained (no system OpenSSL
  linkage), which eases copying a built binary onto helium later.
- This introduces the MVP's first non-trivial dependency tree (ureq, rustls, a
  crypto provider, webpki-roots). It **will** trip `just deps`; their licenses
  (e.g. ISC, ring's OpenSSL-derived terms) must be added to `deny.toml`
  deliberately, with a reason.
- MQTT is not foreclosed — it remains the likely choice for high-frequency fleet
  telemetry and would be a later transport milestone under the same pluggable
  boundary.
- Each reporting host is a distinct yggio device identified by its own `secret`
  (provisioning model detailed in a later ADR).
