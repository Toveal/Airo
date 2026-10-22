# Releasing Airo

## Build and check locally

Use Windows x64, Rust stable (MSVC), Visual Studio C++ Build Tools, WebView2 and Bun 1.4.2.

```powershell
bun install --frozen-lockfile
bun run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
bun run tauri build --ci --config src-tauri/tauri.local.conf.json -- --locked
```

The local config disables updater artifact signing so contributors can build an installer without the maintainer's private key. The installer is in `src-tauri/target/release/bundle/nsis/`. Do not distribute this test build as an updater release.

## Prepare a draft on GitHub

1. Keep the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` identical. Update `src-tauri/Cargo.lock` when changing the Rust package version.
2. Add `docs/releases/<version>.md`. Update the release workflow title when the app leaves the early-release stage.
3. Set repository Actions secrets `TAURI_SIGNING_PRIVATE_KEY` and, if needed, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Use the private key matching `plugins.updater.pubkey` in `tauri.conf.json`. Never commit or print the key. If the existing key is unavailable, generate a new pair with `bun run tauri signer generate` and update the public key before the first release. Existing installations cannot verify updates signed with a different key.
4. Push the reviewed changes. Run **Actions → Draft release → Run workflow** on the intended branch. The workflow checks versions, builds, tests and creates a draft containing the installer, its `.sig` and `latest.json` using [tauri-action](https://github.com/tauri-apps/tauri-action).
5. Complete the manual checks below against that draft's installer. Confirm the manifest contains `windows-x86_64`, the correct version, a signature and the installer URL for that tag. Do not rerun the workflow against an already published version; increment the version instead.
6. Review the notes and publish the draft as a regular release when ready. “Early release” describes maturity; it is not GitHub's pre-release flag. The configured `/releases/latest/` update endpoint follows regular releases.

Updater signing follows the [Tauri updater documentation](https://v2.tauri.app/plugin/updater/). It is separate from Windows Authenticode signing and does not remove SmartScreen warnings.

## Manual release checks

These are checks to perform, not claims of completed validation. Record the Windows build and result for each tested machine in the release notes.

- [ ] Install as a standard user on a clean Windows x64 machine. Check setup with WebView2 present and missing.
- [ ] Confirm startup options in setup agree with the in-app autostart setting.
- [ ] Launch, select colors, close to tray and reopen. Verify English/Russian and both themes.
- [ ] Verify selection colors on one display and on two displays, including different scaling values. Test both switching modes.
- [ ] Exit via the tray and confirm original system colors return.
- [ ] Force-close a test instance, relaunch it, then exit normally; check recovery. Use a test session rather than an important desktop.
- [ ] Disconnect/reconnect a display, sleep/resume and lock/unlock. Check profiles and restored switching.
- [ ] Reboot and verify autostart and start-minimized behavior.
- [ ] Uninstall while Airo is running; verify colors, tray icon and autostart are cleaned up. If this fails, require exit before uninstall and fix cleanup before broad distribution.
- [ ] For a later release, install the previous public version and test the actual in-app update through the public feed. Confirm settings, autostart and color restoration survive it.
- [ ] Confirm both README GIFs and download links render on GitHub.

## Repository presentation

Suggested description: `Automatically switch Windows selection colors to match the monitor under your cursor.`

Suggested topics: `windows`, `windows-11`, `customization`, `multi-monitor`, `tauri`, `rust`, `vue`.

Keep unverified compatibility and performance claims out of the description and release notes. Ask early users for their Windows build, monitor setup and whether they keep Airo enabled.
