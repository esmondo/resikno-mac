# Technical Decisions & Known Issues

## Scanner Logic Flaws

> Discovered: 2026-03-28

### Missing scan targets
- `~/.npm/_cacache` — not in any scanner path; was 5.1 GB on first live scan
- `~/Library/Caches/electron/` and `~/Library/Caches/electron-builder/`
- `~/Library/Caches/ms-playwright-go/` — Playwright browser binaries

### System paths require root
`src/platform/macos.rs` lists `/Library/Caches`, `/var/log`, `/private/var/folders` but a user-space CLI cannot clean these. They should be surfaced as read-only info or gated behind an explicit `--sudo` flag.

### Safety classification gap
The generic `~/Library/Caches` scan treats all entries equally. `SiriTTS/BNNSModels` (968 MB of ML models) should be classified as `LARGE_REDOWNLOAD` rather than `SAFE`.

### Code bugs

`src/platform/macos.rs:103` — `config_dir()` returns `.resikno-mak` (missing `c`). Should be `.resikno-mac`.
Confirmed live: `restore --latest` shows manifest path as `~/.resikno-mak/restore/…` (2026-04-01).

`src/cli/commands.rs` — `restore <date>` positional arg with value `"latest"` returns "restore point not found" instead of delegating to the `--latest` path. Minor UX inconsistency discovered 2026-04-01.

---

## Technical Decisions

### TD-001 — npm cache requires ownership fix before clean (2026-03-28)
- **Decision:** Use `sudo chown -R $(whoami) ~/.npm && npm cache clean --force`
- **Why:** Older npm versions created root-owned cache files; both `rm -rf` and `npm cache clean` fail without fixing ownership first
- **Outcome:** Freed 5.1 GB

### TD-002 — Delegate Homebrew cleanup to `brew cleanup` (2026-03-28)
- **Decision:** Use `brew cleanup` instead of `rm -rf ~/Library/Caches/Homebrew`
- **Why:** `brew cleanup` also removes stale Cellar versions and symlinks — freed 292 MB vs expected 51 MB
- **Outcome:** More thorough than direct deletion

### TD-003 — Unconditionally safe cache categories for macOS (2026-03-28)
The following are safe to delete with no user data at risk (all auto-regenerated):
- npm, cargo, pnpm, bun, TypeScript caches
- Browser caches (Chrome, Firefox, Safari, Edge)
- Electron / electron-builder framework caches
- Playwright browser binaries
- Homebrew download cache

Deferred (requires user judgement):
- `~/Library/Caches/Comet` — Linear app, 940 MB, will re-sync
- `~/Library/Caches/SiriTTS/BNNSModels` — 968 MB ML models, large re-download
- `~/Library/Developer/CoreSimulator/Devices` — 132 MB iOS simulators

---

## Live Test Run — 2026-04-01

Full CLI smoke test against `cargo build` dev binary. All commands launched without panics or crashes.

| Command | Result |
|---|---|
| `resikno` (no args) | ✅ Banner + REPL shell |
| `resikno scan --no-interactive` | ✅ 677 items / 4.3 GB found |
| `resikno clean` (dry-run) | ✅ Items listed with safety colors |
| `resikno analyze --large 100` | ✅ 2 files found |
| `resikno analyze --duplicates` | ✅ Stub message |
| `resikno analyze --old 90` | ✅ Stub message |
| `resikno restore --list` | ✅ 8 restore points shown |
| `resikno restore --latest` | ✅ Stub + manifest path (typo present) |
| `resikno config` | ✅ Stub ("Opening config...") |
| `resikno update` | ❌ Fails — `cargo install` pulls `clap_lex v1.1.0` which requires `edition2024`; local toolchain is Rust 1.79.0. Error is caught and surfaced cleanly. |

### TD-004 — `update` command requires Rust ≥ 1.85 on the host (2026-04-01)
- **Decision (pending):** Either pin `clap_lex` to `< 1.1.0` in `Cargo.toml`, or document that `update` requires an up-to-date toolchain
- **Why:** `cargo install --git …` resolves the latest deps at install time; `clap_lex 1.1.0` adopted edition2024 which Rust 1.79 cannot compile
- **Impact:** `update` always fails silently on machines that haven't run `rustup update`
