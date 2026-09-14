# Windows x64 builds / Windows 64 位构建

The Windows package targets **Windows 10/11 x64**. It is an NSIS `-setup.exe`
installer with Simplified Chinese and English UI, installing for the current user.
If Microsoft WebView2 Runtime is missing, setup downloads and installs it. This
keeps the application download small; first installation can require internet.
It does not bundle the full browser runtime, ROMs, saves or extracted game assets.
The application needs no Rust or Node.js installation on the user's PC.

Windows 包面向 **Windows 10/11 64 位**。运行 `-setup.exe` 按中英文向导安装，默认
只安装到当前用户。缺少 WebView2 时安装程序会联网补装；应用不要求用户安装 Rust
或 Node.js。发布包不含完整浏览器运行时、ROM、存档或提取的游戏资源。

Windows 7/8.1 and 32-bit Windows are not supported by this release. Renaming the
EXE, lowering its subsystem version or embedding the current WebView2 bootstrapper
does not establish old-system compatibility. Microsoft ended WebView2 support
for Windows 7/8.1 at version 109. This project does not ship a frozen legacy engine.

本包不支持 Windows 7/8.1 或 32 位系统。不能通过改文件名、降低 EXE 头部版本号，
或打入当前运行时引导程序就宣称兼容旧系统。

## Build on Windows

Install the normal [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/),
including Rust/MSVC, Visual Studio C++ build tools and Node.js. From the repository:

```sh
npm ci
npm run desktop:build -- --target x86_64-pc-windows-msvc --bundles nsis
```

The platform override `apps/desktop/tauri.windows.conf.json` selects NSIS and its
installation options. CI uses the same platform override.

## Cross-compile locally on macOS

Follow Tauri's [Windows cross-compilation guide](https://v2.tauri.app/distribute/windows-installer/#build-windows-apps-on-linux-and-macos):

```sh
brew install llvm lld nsis
export PATH="$(brew --prefix llvm)/bin:$(brew --prefix lld)/bin:$PATH"
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
npm ci
npm run desktop:build -- --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis
```

`cargo-xwin` downloads Microsoft's Windows SDK and CRT into its build cache.
Optionally set `XWIN_CACHE_DIR` and `CARGO_TARGET_DIR` to local-disk directories;
network filesystems can lack the locking or syncing operations build tools need.
The installer is emitted under
`$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release/bundle/nsis/` (default target
location: `target/`). Building MSI installers requires a Windows host.

The local September 2026 build uses Rust 1.98.1 and cargo-xwin 0.23.1. Temporary
compiler/SDK caches are development dependencies and are never release resources.

## Verification and limits / 验证范围

A successful cross-build and PE inspection verify compilation, architecture and
packaging. They **do not** verify the Windows GUI or installation behavior. Each
release's `build-info.json` records its actual validation and signing status.
The initial local Windows package is unsigned and has not been run on a Windows
machine. Do not label it Windows-tested based on macOS tests or compilation alone.

交叉编译成功和 PE 检查能确认编译、架构与打包结果，**不等于 Windows 上实际运行
通过**。首次本机 Windows 包未做 Windows 实机验证，也未进行 Authenticode 签名。

Before declaring Windows validation complete, use a disposable save and check:

- Install, launch and uninstall on Windows 10 and Windows 11 x64, including a
  machine without WebView2. Verify the Chinese/English installation UI.
- Open both advertised ROM profiles and a copied 128 KiB battery save through
  native file dialogs. Verify rejection of an unsupported ROM.
- Render sprites/maps; edit moves/items and origins; undo/redo; export to a new
  path and reopen. Check a Chinese directory path and backup behavior.
- Switch Pokémon while retaining the inspector tab, scroll fixed controls and
  use Chinese IME in searchable fields.

References: [Tauri installer documentation](https://v2.tauri.app/distribute/windows-installer/),
[Microsoft WebView2 supported platforms](https://learn.microsoft.com/en-us/microsoft-edge/webview2/),
[Microsoft's legacy support announcement](https://blogs.windows.com/msedgedev/2022/12/09/microsoft-edge-and-webview2-ending-support-for-windows-7-and-windows-8-8-1/).
