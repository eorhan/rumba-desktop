# Rumba Desktop

Minimal, lightweight desktop client for **Rumba** — a voice chat and messaging
app at <https://rumba.hesap.net.tr>.

It is a thin [Tauri](https://tauri.app) shell that opens the Rumba web app in a
native window using the operating system's built-in WebView (WebView2 on
Windows), so it stays tiny and light on memory. There is no bundled browser and
no business logic here — the application code lives on the server.

## What it does

- Opens `https://rumba.hesap.net.tr` in a frameless native window
- Single instance (re-launching focuses the existing window)
- Opens external links in the system browser; keeps the Google sign-in flow in-app
- No access to native APIs is granted to the web content

## Build

Requires [Rust](https://rustup.rs) and [Node.js](https://nodejs.org).

```bash
npm install
npm run tauri build   # produces an NSIS installer under src-tauri/target/release/bundle
```

On Windows this yields `Rumba_<version>_x64-setup.exe`.

## Code signing

See [docs/code-signing-policy.md](docs/code-signing-policy.md).

## License

MIT — see [LICENSE](LICENSE).
