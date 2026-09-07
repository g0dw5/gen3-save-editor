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
