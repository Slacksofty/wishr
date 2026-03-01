# wishr — project context for Claude

## Overview
Full-stack Rust web app: personal wishlist/purchase tracker.
Stack: **Leptos 0.7 + Axum 0.7 + SQLite (sqlx 0.8)**, built with **cargo-leptos 0.3.5**.

## Key commands
```bash
cargo leptos watch          # dev server with hot reload → http://localhost:3000
cargo leptos build          # debug build
cargo leptos build --release # release build
```

## Source layout
```
src/
  main.rs                   # Axum server entry: mounts DB pool, leptos_axum handler
  lib.rs                    # hydrate() entry for WASM client
  app.rs                    # App component + Router with all routes
  models.rs                 # Shared data structs (ItemList, WishItem, PurchaseRecord, …)
  server/
    mod.rs                  # pub mod items; pub mod db;
    db.rs                   # DB pool initialisation (sqlx::SqlitePool + migrations)
    items.rs                # All #[server] functions
  components/
    mod.rs
    lists_home.rs           # / — list-of-lists grid + create form
    wish_list.rs            # /list/:list_id — active items, rename, move, pending budget
    wish_form.rs            # /list/:list_id/add  and  /list/:list_id/edit/:id
    list_archive.rs         # /list/:list_id/archive — per-list purchased items + stats
    archive.rs              # /archive — global purchased items + stats
migrations/
  001_init.sql              # wish_items + purchase_records tables
  002_add_lists.sql         # item_lists table + list_id column on wish_items
style/
  main.scss                 # All styles (compiled by cargo-leptos via grass)
```

## Models (src/models.rs)
```rust
ItemList        { id, name, created_at }
ItemListSummary { id, name, created_at, active_count, estimated_budget }  // from JOIN query
WishItem        { id, list_id, name, description?, estimated_cost?, want_level,
                  need_level, where_to_buy?, category?, notes?, status, created_at }
PurchaseRecord  { id, item_id, actual_cost?, purchased_at, notes? }
ArchivedItem    { item: WishItem, record: PurchaseRecord }                 // composed, not a DB row
WishStats       { total_spent, pending_budget, active_count, purchased_count }
```
- status values: `'active'` | `'purchased'`
- want_level / need_level: 1–5; items sorted by `want × need DESC`
- `#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]` on DB-mapped structs

## Server functions (src/server/items.rs)
All use `extract::<Extension<SqlitePool>>()` to get the DB pool from Axum.

**Lists**
- `get_item_lists_with_stats() -> Vec<ItemListSummary>`
- `get_item_list(id) -> ItemList`
- `get_item_list_detail(id) -> ItemListSummary`
- `create_item_list(name) -> i64`  (returns new id)
- `rename_item_list(id, name)`

**Wish items**
- `get_wish_items(list_id) -> Vec<WishItem>`   (active only, sorted by want×need)
- `get_wish_item(id) -> WishItem`
- `create_wish_item(list_id, name, …) -> i64`
- `update_wish_item(id, name, …)`
- `transfer_wish_item(item_id, target_list_id)`
- `mark_as_purchased(id, actual_cost?, notes)`  (sets status + inserts purchase_record)
- `delete_wish_item(id)`  (cascades to purchase_records)

**Archive / stats**
- `get_archive() -> Vec<ArchivedItem>`            (global)
- `get_archive_for_list(list_id) -> Vec<ArchivedItem>`
- `get_stats() -> WishStats`                       (global)
- `get_list_stats(list_id) -> WishStats`

## Routes (app.rs)
```
/                                     ListsHome
/list/:list_id                        WishList
/list/:list_id/archive                ListArchive
/list/:list_id/add                    WishForm  (create)
/list/:list_id/edit/:id               WishForm  (edit)
/archive                              Archive
```

## Feature flags (Cargo.toml)
```toml
hydrate = ["leptos/hydrate"]
ssr     = ["dep:axum", "dep:tokio", ..., "leptos/ssr", "leptos_meta/ssr", "leptos_router/ssr"]
```

## Leptos 0.7 API notes
- `use leptos::prelude::*` for main imports
- `use leptos::web_sys` to access web_sys types (not bare `web_sys::`)
- `A` component from `leptos_router` does NOT accept `class` prop — use plain `<a>` tags for styled links
- Callback props need `+ Send` bounds: `impl Fn() + Clone + Send + 'static`
- `tokio::join!` not available in WASM — use separate `Resource::new()` per async call
- `leptos_meta` and `leptos_router` do NOT have `hydrate` feature in 0.7 — only `leptos/hydrate`
- `log` crate must be added as an explicit optional dep (not inherited from simple_logger)

## Known gotchas
- **sqlx 16-tuple limit**: `FromRow` is not implemented for tuples > 16 fields.
  Archive queries use a named inner struct `ArchivedRow` with `#[derive(sqlx::FromRow)]`
  and SQL column aliases (`w_id`, `p_id`, etc.) to work around this.
- **SQLite ALTER TABLE**: `ADD COLUMN … NOT NULL DEFAULT x REFERENCES …` is rejected.
  Drop the `REFERENCES` clause in migrations; enforce FK logic in application code.
- **DB file**: `./wishr.db` locally, `/app/data/wishr.db` in Docker/dev container.
  Migrations run automatically at startup via `sqlx::migrate!()`.

## GitHub
- Repo: `github.com/Slacksofty/wishr` (SSH, gh CLI authenticated as Slacksofty)

## Docker / self-hosting
- `Dockerfile` — multi-stage: `rust:1.83-slim` builder → `debian:bookworm-slim` runtime
- `docker-compose.yml` — port 3000, named volume `wishr-data:/app/data`, `restart: unless-stopped`
- Env vars: `LEPTOS_SITE_ADDR=0.0.0.0:3000`, `DATABASE_URL=sqlite:/app/data/wishr.db`

## Dev Container (.devcontainer/)
- `Dockerfile`: `rust:1.85-slim` + apt deps + wasm32 target + cargo-leptos prebuilt binary
  - Use prebuilt binary from GitHub releases, NOT `cargo install` (Rust version constraints)
  - Non-root user `vscode` (uid/gid 1000) — Claude Code blocks `--dangerously-skip-permissions` as root
- `devcontainer.json`: `remoteUser: vscode`, forwards port 3000, `postCreateCommand: cargo fetch`
- **When rebuilding**: "Dev Containers: Rebuild Container" — never "Reopen" (reuses stale container)
