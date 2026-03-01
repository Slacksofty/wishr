# wishr

Personal wishlist and purchase tracker. Centralise the things you want to buy, score them by want/need level, track estimated and actual costs, and keep a full purchase history — all organised into named lists.

Built entirely in Rust: server, client, and database.

## Features

- **Multiple lists** — create as many named lists as you want (Tech, Books, Home, etc.)
- **Want / Need scoring** — rate each item 1–5 on both axes; the list sorts by their product so the most justified items float to the top
- **Cost tracking** — estimated cost per item, pending budget shown live on the list page
- **Purchase logging** — mark items as bought with the actual price paid and optional notes
- **Item transfer** — move an item between lists at any time
- **Inline list renaming** — rename a list directly from its page without a modal
- **Per-list archive** — view purchased items and total spent scoped to one list
- **Global archive** — full cross-list purchase history with overall totals

## Stack

| Layer | Technology |
|---|---|
| Language | Rust (100%) |
| Frontend | [Leptos 0.7](https://leptos.dev) — compiled to WASM |
| Backend | [Axum 0.7](https://github.com/tokio-rs/axum) |
| Database | SQLite via [sqlx 0.8](https://github.com/launchbadge/sqlx) |
| Build tool | [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) |
| Styling | SCSS, dark theme |

## Getting started

### Prerequisites

- Rust toolchain (`rustup`)
- `wasm32-unknown-unknown` target
- `cargo-leptos` (install as a prebuilt binary — see note below)

```bash
rustup target add wasm32-unknown-unknown
```

> **Windows note:** install `cargo-leptos` from its [GitHub releases](https://github.com/leptos-rs/cargo-leptos/releases) as a prebuilt binary rather than building from source (`cargo install` fails because OpenSSL requires Perl modules that Git for Windows does not ship). Extract the `.tar.gz` using `C:\Windows\System32\tar.exe`, not Git's `tar`.

### Run in development

```bash
cargo leptos watch
```

Opens at `http://localhost:3000`. The SQLite database (`wishr.db`) is created automatically on first run.

### Release build

```bash
cargo leptos build --release
```

## Project structure

```
src/
  main.rs              # Axum server entry point (SSR feature)
  lib.rs               # WASM hydration entry point
  app.rs               # Router and top-level App component
  models.rs            # Shared data structures (WishItem, ItemList, …)
  server/
    db.rs              # Database pool initialisation + migrations
    items.rs           # All server functions (list CRUD, item CRUD, archive, stats)
  components/
    lists_home.rs      # Home page — grid of list cards
    wish_list.rs       # List page — items, rename, move, pending budget
    wish_form.rs       # Add / edit item form
    list_archive.rs    # Per-list purchase archive
    archive.rs         # Global purchase archive
migrations/
  001_init.sql         # wish_items, purchase_records tables
  002_add_lists.sql    # item_lists table, list_id column on wish_items
style/
  main.scss            # Dark theme styles
```

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:./wishr.db` | SQLite connection string |
