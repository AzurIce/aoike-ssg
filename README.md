# Aoike

An experimental static site generator that decouples content building from frontend rendering.

## Usage

Aoike is designed to be flexible. It separates the content processing logic from the presentation layer.

`aoike` crate provides the core data structures and the build logic to convert your Markdown/Typst vault into a JSON-based API.

`aoike-leptos` is the reference implementation of a frontend app using Leptos, which consumes the generated JSON data.

## Design Philosophy

The core philosophy is **"Content as Data, Frontend as App"**.

Instead of generating static HTML files for every page or compiling content into the WASM binary (which bloats the size), Aoike treats your content vault as a database.

The process consists of two decoupled phases:

1. **The Build Phase**:
   This can happen via the `aoike` CLI or a `build.rs` script. We:
    - Scan your vault directory (posts, notes, etc.).
    - Use `git` to retrieve creation and update timestamps.
    - Parse Markdown (via `pulldown-cmark`) or Typst to HTML.
    - **Asset Handling**: Automatically detect relative links to local images/files, rewrite them to absolute URLs, and copy the assets to the output directory.
    - Export a `vault.json` (manifest/tree structure) and individual JSON files for each article.

2. **The Runtime Phase**:
   The frontend is a standalone Single Page Application (SPA) (e.g., built with Leptos).
    - On load, it fetches `vault.json` to build the posts and notes list.
    - When a user navigates to a page, it fetches the corresponding JSON file for that article on demand.
    - This ensures the initial load is fast and the app remains lightweight, regardless of how much content you have.

With this architecture:
- **Framework Agnostic**: You can build the frontend with Leptos, Dioxus, React, Vue, or anything that can fetch JSON.
- **Incremental Loading**: Users only download the content they view.
- **Rich Content**: Support for Markdown and Typst out of the box.
- **Asset Management**: Local assets in your vault just work, mirroring the folder structure in the output.
