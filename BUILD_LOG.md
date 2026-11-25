# Zavro Client — Build Log

## Built

- [ ] Release v1.0.0 — Fedora 45 `.rpm` + `.AppImage` (GitHub Actions)
  - Workflow: `.github/workflows/release.yml` (push to main + v* tags + manual dispatch; release step pins tag v1.0.0)
  - Xray-core version shipped: _to be filled in from the CI run_
  - Release URL: _to be filled in after the main push triggers CI_

## Verified (2026-11-25, coordinator integration pass)

- [x] `npm run build` — `tsc --noEmit && vite build`, 1594 modules, built in ~3.3s, exit 0
- [x] `npx tsc --noEmit` — exit 0
- [x] `cargo test` (with `ZAVRO_XRAY_BIN=.../binaries/xray-x86_64-unknown-linux-gnu`) — **15 passed, 0 failed**, including `generated_config_passes_real_xray_test` (real `xray -test` on generated VLESS/Reality+Vision, Trojan, VLESS/WS/TLS configs incl. TUN inbound)
- [x] `cargo clippy` — 0 errors, 0 warnings; `cargo fmt --check` — clean
- [x] Real Xray binary: `Xray 26.11.9 (Xray, Penetrates Everything.) 676eaf6 (go1.27.3 linux/amd64)` — downloaded from XTLS/Xray-core latest release; `1.8.8` appears nowhere as a shipped version
- [x] `.github/workflows/release.yml` parses as valid YAML (python yaml.safe_load)
- [x] `src-tauri/icons/icon.svg` — Z-hexagon mark, blue gradient on dark rounded square (valid XML)
- [x] `actionlint` not available in this VM; workflow validated by YAML parse only

## Integration fixes (coordinator, 2026-11-25)

- Frontend `connect` sent `{ nodeId }` but the Rust command parameter is `node_id` (Tauri deserialises by exact name) — fixed `src/api.ts` to send `{ node_id: nodeId }`. All other command arg keys already matched.
- Added `src-tauri/capabilities/default.json` (`core:default` for window `main`) — without it Tauri v2 would deny all `invoke()` calls at runtime. Verified it is picked up: `src-tauri/gen/schemas/capabilities.json` now lists the default capability.
- Fixed CI geo-file download: `geoip.dat`/`geosite.dat` are NOT separate assets in Xray-core v26.11.9 — they ship inside `Xray-linux-64.zip`. The workflow now tries the separate URLs as a non-fatal fallback and verifies the files from the unzipped archive, failing loudly if absent either way.
- CI now also stages `src-tauri/scripts/zavro-tun.sh` into `src-tauri/resources/zavro-tun.sh` so the installed app finds the TUN helper at `<exe>/resources/zavro-tun.sh` (see `src-tauri/src/tun.rs`).
- README corrections: geo files come from inside the zip; TUN and system-proxy are independent toggles (TUN takes effect on next connect), not mutually exclusive.
- `.gitignore` += `src-tauri/gen/` (generated Tauri schemas).

## Known limitations

- The Fedora `.rpm`/`.AppImage` are built **only** by GitHub Actions
  (Fedora 45 container). This VM is Ubuntu 24.04 — no Fedora artifact has
  been produced or tested locally, and none is claimed here.
- `cargo tauri build` — _see status below; local Ubuntu build is best-effort only_
- TUN needs polkit + kernel `tun` module; speed graph is interface-sampled
  from `/proc/net/dev`; ping is TCP-connect to node host:port, not Xray
  URL-test; seed nodes ship with no measurements until Check Ping, and
  their seed links carry placeholder UUID/password (`00000000-…`,
  `REPLACE-ME`) — replace them with real user links before connecting.
  (Full list in README.md.)
- The app has never been launched end-to-end (no real node credentials
  exist; no reachable `*.zavro.net` servers from this VM). Live
  connect / gsettings / TUN paths are implemented but unproven until a
  Fedora 45 run with real credentials.
