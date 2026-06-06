# AGENTS.md - secure_voting

## Project

Rust TUI voting app using **ratatui 0.28** + **crossterm 0.28**. Edition **2024** (requires rustc 1.85+). No tests, no CI, no linter config (uses stock `cargo fmt` / `cargo clippy`).

## Commands

| Action | Command |
|--------|---------|
| Build | `cargo build` |
| Run | `cargo run` |
| Check | `cargo check` |
| Lint | `cargo clippy -- -D warnings` |
| Format | `cargo fmt --check` (check), `cargo fmt` (apply) |

## Architecture

```
main.rs          → terminal init, keyboard thread, event loop
app.rs           → scene router (draw/handle/on_enter/on_exit)
event.rs         → Event enum { Input, Progress }
scene/login.rs   → LoginScene (Enter → Dashboard)
scene/dashboard.rs → DashboardScene (progress gauge, background thread)
```

**Scene lifecycle**: `on_enter(tx)` → `handle(key) → Action` → `on_exit()`. Scenes switch via `Action::SwitchScene(Scene::...)`.

**Key routing**: `q` to quit is handled in `app.rs`, not per-scene. Scene handlers return `Action::None` for unhandled keys.

**Cross-thread**: Dashboard spawns a thread that sends `Event::Progress(f64)` over mpsc. The event loop in `App::run` maps this onto `DashboardScene.progress`.

**`Q` to quit** is hardcoded in `App::handle_key_event` before the scene gets the key. Adding new global keys must go there.

## Adding a new scene

1. Create `src/scene/my_scene.rs`
2. Register `pub mod my_scene;` in `scene/mod.rs`
3. Add variant to `Scene` enum and implement its match arms in `draw`/`handle`/`on_enter`/`on_exit`
4. Wire the switch in another scene's `handle` via `Action::SwitchScene(Scene::MyScene(...))`
