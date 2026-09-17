# Release checklist / 发布检查

Run CI on the exact commit; run local ROM and UI tests for every advertised
profile; verify a disposable gameplay save in an emulator. Record platform and
architecture. Build installers through the manually dispatched artifact workflow.
It produces downloadable build artifacts and does not automatically publish a
GitHub release.

Before publishing, include both README languages, license and third-party
notices; state that users provide ROMs and 128 KiB battery saves. Verify that the
bundle contains neither a ROM, local saves, generated game assets, test fixtures,
source maps intended only for development, nor the opt-in HTTP bridge. Include
file checksums and label unsigned/unnotarized builds accurately. Signing and
notarization credentials belong in CI secrets, never the repository.

发布前验证同一提交上的测试、全部宣称支持的 ROM、模拟器回读及目标平台安装包。
发布说明同时提供中英文，列明版本 MD5、已验证能力和研究缺口。当前工作流只生成
构建产物，不自动创建或发布 GitHub Release。

## Changelog / 更新日志

`CHANGELOG.md` starts at the first public release, **0.1.5**. Before each release,
update its English and Chinese entries, promote the unreleased heading, and keep
its version consistent with Cargo, npm and Tauri. Record internal build changes
without inventing public release dates.

Tauri's shared `bundle.resources` configuration embeds `CHANGELOG.md` in every
platform's app resources, including Windows installers. The artifact workflow
also includes a directly readable copy alongside the installers. When assembling
an archive manually, include a top-level copy of `CHANGELOG.md` alongside the
README files and licenses; verify it matches the packaged commit. Do not remove
the shared resource mapping in platform overrides.

更新日志以首次公开发布的 **0.1.5** 为基线。后续发布前补齐中英文变更记录、移除
对应“尚未发布”标识，并保持 Cargo、npm、Tauri 版本一致。应用资源及 CI 附件会自动
包含日志；手工整理 ZIP 时也要在顶层放入同一提交的 `CHANGELOG.md`，便于用户直接阅读。
