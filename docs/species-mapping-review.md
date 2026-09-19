# Species mapping review / 宝可梦映射审核

The reviewer is an independent local developer tool. It is not part of the
editor UI or release bundle. It produces the game-scoped identity configuration
consumed by the editor's official base-stat comparison.

审核页是独立的开发辅助工具，用来生产修改器加载的配置文件，不是修改器内的功能。
它不修改 ROM 或存档，也不影响宝可梦名称搜索、进化合法性、存档写入或实际种族值。

## Run / 启动

Build the CLI with `cargo build -p gen3-cli`, then supply your own supported ROMs:

```sh
python3 scripts/review_species_mappings.py \
  --cli target/debug/gen3 \
  --rom /path/to/game-a.gba \
  --rom /path/to/game-b.gba
```

Open `http://127.0.0.1:8791`. Use `--port` to choose a different local port.
The CLI reads the ROM catalog at startup and checks its MD5 against the profile.
Keep the server running while reviewing. Restart with the same ROM paths to
resume; decisions are stored on disk, not just in browser storage.

先构建命令行工具，再通过 `--rom` 提供自己的 ROM，可重复指定多个游戏。
启动时从 ROM 读取名称和六围并验证 MD5。保持服务运行，浏览器打开上面的地址即可。
重启后会读取已保存的审核进度。页面默认只显示原来未能自动关联的待审核条目；取消
「仅看原来未匹配」可复核原先已匹配的记录。

## Decisions / 审核决定

- **Direct / 直接映射**: the same Pokémon and official form, even if the hack
  changes its stats. Mega Mewtwo X and Y have separate official reference keys.
- **Comparison / 仅作数值参照**: compare a custom Pokémon/form with a selected
  official entry without asserting they share an identity.
- **None / 无官方对应**: deliberately leave the official comparison blank.
- **Pending / 待审核**: a proposal only; never exported as an approved decision.

每次确认写入审核文件，并备份之前的版本。「稍后再审」仅切换条目；「恢复待审核」
取消该条决定；「撤销上次」撤销当前会话最近一次条目修改。修改候选会恢复为待审核。
备注可以单独保存，也会随确认一起保存。未保存备注离开页面时会提示。

The proposal reason and confidence explain the evidence, not a statistical
probability. Assistant-prepared Chinese alias/semantic suggestions, ROM battle
relations, Mega-stone X/Y names, and form-family relationships reduce lookup
work. Stat distance is used to order plausible form candidates, never to certify
identity. Custom suffixes do not establish equivalence to an official Mega or
regional form. No remote model service is called by the reviewer.

候选由本次助手预审准备：结合旧译名知识、名称语义、ROM 战斗变身关系及形态家族，
再用六围接近程度辅助排列形态。高把握不等于已确认；自创形态仍需人工判断。
超梦 #990 / #991 的 X/Y 候选来自 ROM 的超梦石 X/Y 触发关系，不是按编号猜测。
审核工具运行期间不会调用在线模型。

## Files and publishing / 文件与生成

| Location | Purpose |
| --- | --- |
| `config/species-mapping-reviews/*.json` | Review source: proposals, evidence labels, decisions, notes and revision |
| `.local/mapping-review-backups/` | Previous review revisions; private, outside releases |
| `ui/data/species-mappings/*.json` | Minimal generated configuration loaded by the editor |

「导出审核进度」保存可转移的完整审核文件。「导入审核进度」只接受相同 ROM MD5；
校验全部条目后一次性保存。不同游戏的相同内部编号不会互相覆盖。多个页面同时
修改时，版本冲突会拒绝覆盖，请刷新后继续。

Click **Generate editor configuration / 生成修改器配置** for each game after
reviewing. This writes the minimal file under `ui/data/species-mappings/` and
also downloads a copy. Only reviewed `status` and `target` pairs are exported;
no candidates, notes, ROM names, stats, sprites or ROM bytes are included.
The editor loads these files on its next development refresh/build. A previously
installed release is not modified by generating files; rebuild it to include the
new mappings. Review sources and this tool are outside the Vite application
import graph and do not enter the application bundle.

每个游戏分别点击「生成修改器配置」：输出目录和浏览器下载的副本均只包含已审核的
映射。候选、备注和 ROM 目录不进入正式配置。修改器下一次构建加载生成文件；不会
自动替换已经安装的旧应用。游戏中的名称和六围仍直接从 ROM 读取。

```json
{
  "schema": 1,
  "profile_id": "example-game",
  "rom_md5": "exact-rom-fingerprint",
  "entries": {
    "990": { "status": "direct", "target": "150:超级超梦Ｘ" },
    "899": { "status": "comparison", "target": "9:" },
    "976": { "status": "none", "target": null }
  }
}
```

The target is `official National Dex number:exact form label`; it is not the
hack's reused Dex number. Approved game mappings take priority over automatic
name matching. Existing per-user manual comparison choices remain highest
priority. Entries absent from the generated file retain the previous automatic
matching behavior. An explicit `none` suppresses that fallback.

## Verification / 验证

```sh
node --experimental-strip-types scripts/test_species_mappings.mjs
python3 scripts/test_species_mapping_review.py
python3 scripts/test_species_mapping_review_ui.py  # Playwright + Chrome
npm run build
```

Tests cover X/Y proposals, cross-ROM isolation, rejection of stale/invalid
writes, all-or-nothing import validation, persisted progress, undo, explicit
no-counterpart decisions, candidate search and publishing without unreviewed
metadata. Browser tests use temporary synthetic configurations, not real reviews.
