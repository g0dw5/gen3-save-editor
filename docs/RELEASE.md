# Release checklist / 发布检查

Run CI on the exact commit; run local ROM and UI tests for every advertised
profile; verify a disposable gameplay save in an emulator. Record platform and
architecture. Build installers through the artifact workflow. Branch dispatches produce build
artifacts; version tags produce a GitHub Release draft after validation.
Public publication remains a separate, reviewed action.

Before publishing, include both README languages, license and third-party
notices; state that users provide ROMs and 128 KiB battery saves. Verify that the
bundle contains neither a ROM, local saves, generated game assets, test fixtures,
source maps intended only for development, nor the opt-in HTTP bridge. Include
file checksums and label unsigned/unnotarized builds accurately. Signing and
notarization credentials belong in CI secrets, never the repository.

发布前验证同一提交上的测试、全部宣称支持的 ROM、模拟器回读及目标平台安装包。
发布说明同时提供中英文，列明版本 MD5、已验证能力和研究缺口。分支手动构建只生成
产物；版本标签构建通过后创建 Release 草稿，不自动公开。

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


## Branches and immutable tags / 分支与不可变标签

The default branch is `main`. Merge completed feature branches into it, verify
the push, and then delete those feature branches. Keep development versions
unreleased until packaging is requested. A tag `vX.Y.Z` identifies the exact
source of the distributed binaries; never point an old version at newer fixes
or replace an already published tag/installer in place. Use a new patch version.

默认主分支为 `main`。功能完成后合入主分支，确认推送成功，再删除临时开发分支。
标签必须对应安装包的真实源码；已发布的标签与安装包不覆盖，新修复使用新补丁版本。

## Release procedure / 发布步骤

1. Complete the checks above on the intended source, including private ROM
   regressions for every supported profile. Private ROMs and saves stay local.
2. Set matching Cargo, npm (including lockfiles), and Tauri versions. Date the
   bilingual changelog section, then run `python3 scripts/release.py check --tag vX.Y.Z`.
3. Commit and merge into `main`, then create an annotated tag:
   `git tag -a vX.Y.Z -m "Release vX.Y.Z"`. Push the branch and that exact tag.
4. The tag workflow validates versions, ancestry, tests and builds Windows x64
   and macOS arm64 installers. It stages only installers and allowlisted docs,
   records source SHA and hashes, and creates a **draft** Release. A rerun cannot
   overwrite an existing Release; inspect and reconcile draft artifacts first.
5. Test the downloaded installers on the target systems, audit payloads, verify
   checksums, and add bilingual compatibility/signing/validation notes. Mark
   untested platforms explicitly. Publish the draft only when authorized.
6. Link announcements to `/releases/tag/vX.Y.Z` for a fixed version or
   `/releases/latest` for the maintained download entry. A netdisk mirror is a
   second download option, and must match the same installer SHA-256 values.

先完成各 ROM 回归，再统一版本、确认中英文日志、提交并标注准确的源码。
标签触发双平台构建，附件含安装包、双语说明、构建信息与校验值；先生成草稿。
检查安装包内容和目标平台运行情况后才公开，不把编译成功当作实机验证。
宣传可同时给出 GitHub Release 与网盘镜像，两个入口的安装包校验值应一致。

## Historical baseline / 历史基线

GitHub tags and Releases begin with `v0.2.0`; the changelog still records the
first public version, 0.1.5. The 0.2.0 tag points to the final reviewed-mapping
source, before the 0.2.1 changes. Its previously built Windows EXE and macOS ZIP
are retained byte-for-byte. Their build manifests contain commit IDs from before
author anonymization; the Release provenance records the equivalent current
source and artifact hashes. Do not rebuild an older tag with newer source.

GitHub 的标签与 Release 从 0.2.0 开始维护，更新日志继续保留最早公开版本 0.1.5。
0.2.0 对应最终人工映射完成的源码，复用已分发安装包，不纳入 0.2.1 修复。
旧构建记录中的提交号来自署名重写之前，Release 附件另记等价源码与校验值。

Workflow behavior follows [GitHub release documentation](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
and [tag-triggered workflow events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows).
