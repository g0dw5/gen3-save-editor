# Multi-ROM adapters / 多 ROM 适配

Status: development branch, 2026-09-19. This is the first executable Rocket
adapter milestone, not full Rocket editing support. No new release is produced.

## Support boundary / 支持边界

| Capability | Dark Phantom BW / DP | Team Rocket 2.1 Chinese |
| --- | --- | --- |
| Exact ROM MD5 and length check | Required | Required |
| Party, boxes, inventory | Read/write | Read-only |
| Species, ordinary moves, items, abilities | Available | Available |
| Learnsets | Existing level/TM/tutor/egg readers | Level-up and ancestor level-up only |
| Individual artwork | Verified Unown/Spinda rules | Representative species artwork |
| Maps, encounters, trainers, Pokédex progress | Existing supported readers | Disabled pending adapter verification |
| Mega/primal associations | Not enabled | Read-only source/target/trigger references |
| Z attacks | Not enabled | Reserved internal IDs excluded from ordinary moves; no editor UI |
| Dynamax / Tera | Not enabled | Unverified; no invented persistent fields |

中文：火箭队已能通过现有应用打开 ROM、查看同行与所有盒子、读取六类道具栏位，
并浏览种族、普通招式、道具、特性、升级招式与战斗形态关系。右侧显示 PID 性格与
实际性格、特性、六项能力、IV/EV 和 PP。地图、图鉴进度和所有写入暂未开放。
“没有验证相遇规则”不会显示为“没有相遇地点”。ROM 中保留的表格行不等于独立
可收集的宝可梦数量；招式目录 755 行包含空招式 0，排除内部 Z 招式。

## Composition / 组合方式

A profile combines independent components, rather than subclassing another game:

1. **Identity and tables:** exact MD5, byte length, counts, addresses and strides.
2. **ROM table formats:** species, moves and learnsets each select their own
   decoder. A future hybrid can reuse one expanded table without taking all
   other tables from Rocket. Wide move powers and three `u16` ability IDs fit
   the common domain model without truncation.
3. **Pokémon codec:** encrypted shell, substructure permutation and checksums
   remain shared; field ownership describes precise bits in headers/canonical
   data. Rocket moves the ball, PP bonuses, friendship and ability; its
   experience is 23 bits and its effective nature can override PID nature.
4. **Save layout:** sector payload sizes, party/storage placement, pocket
   capacities/encryption and optional Pokédex layout. Unknown extended storage
   remains intact. A missing dex layout never falls back to Emerald's flags.
5. **Rules:** battle transformations are separate from permanent evolutions,
   learnable moves and storage species. Optional rules select verified behavior;
   source and target links retain their ROM evidence offset. A transformation
   association does not prove battle eligibility or unlock state.
6. **Capabilities:** the UI and backend use the same profile. Session mutations,
   Save mutation entry points, ROM patching and save export enforce support;
   free editing cannot bypass it. Low-level in-memory codecs remain testable.

中文：不能承诺所有新 ROM 只需填写偏移量。相同二进制格式可直接复用解析器；
字段布局不同需增加编解码器；Mega 等生命周期不同需增加规则。究极绿宝石后续应
逐项选择或补充这些组件，不应把它直接标成“火箭队同款”。本轮没有声称已经适配
究极绿宝石，也没有给未调查的能力自动开启开关。

The UI keeps its existing party/all-box arrangement. Read-only adapters get a
separate inspection panel, explicit coverage text and no active mutation controls.
Sprite cache keys include ROM MD5. Opening a new ROM discards reference windows,
drafts and old save state; an in-flight world response from a previous profile
cannot populate the new profile's map/trainer cache.

## Regression contract / 回归约束

Public generated fixtures run in CI without ROMs or saves:

- All 24 Pokémon permutation orders across BW, DP and Rocket codecs; exact
  no-op preservation and independent literal masks for owned/unowned bits.
- Wider ability IDs, effective nature, PP/experience separation and packed balls.
- Bad eggs and checksum errors rejected for both header formats.
- Read-only writes rejected without mutations or undo entries, including free
  mode and direct Save calls; expanded PC pocket capacity.
- Battle transformations excluded from evolution ancestry; incoming and outgoing
  form references; independent composition of ROM table formats.
- Browser switching BW → Rocket → DP, read-only drag rejection, disabled export,
  restored legacy controls, form references and delayed-world isolation.
- Existing drag transactions, editor navigation, Hidden Power and origin search
  tests continue to run alongside the new adapter tests.

Local tests additionally open both exact Dark Phantom ROMs and the exact Rocket
ROM. The native probe compares Rust-decoded fields, recalculated stats and
synthetic field edits against the ROM's actual ARM functions under Unicorn.
The original save is read only; synthetic edit vectors contain no user file writes.

```sh
# User-supplied paths; no ROMs are downloaded or embedded.
GEN3_ROM_BW='/path/BW.gba' GEN3_ROM_DP='/path/DP.gba' \
GEN3_ROM_ROCKET='/path/Rocket.gba' GEN3_SAVE_ROCKET='/path/Rocket.sav' \
GEN3_ADAPTER_PROBES='/private/local/probes.json' \
  cargo test -p gen3-core -- --include-ignored

# Python with Unicorn installed; probe JSON stays private.
python3 scripts/verify_adapter_native.py --rom '/path/Rocket.gba' \
  --probes '/private/local/probes.json'

# With Vite running; generated browser fixtures, no backend required.
python3 scripts/test_adapter_ui.py
```

Observed on 2026-09-19: all 50 core tests (including the two local-ROM
regressions), six browser regression scripts, Rust formatting/Clippy, TypeScript
and the production UI build passed. Also passed: 153 native stat comparisons, 2,907 native getter
comparisons and 144 byte-for-byte setter comparisons passed, including all
24 PID permutations and the nine Pokémon in the supplied gameplay save.
Both existing Dark Phantom real-ROM regressions passed.

## Next gates / 后续阶段

- Validate Rocket item/mail mappings, egg/ribbon/form invariants and complete
  evolution/learnset legality before exposing Pokémon editing.
- Audit save mutations and derived party data against native functions; test
  disposable edited saves in-game, save again and re-import. Native probes alone
  are not an emulator/gameplay round trip.
- Add map/event/trainer formats and references with independent coverage tests,
  then verify Pokédex offsets and story-dependent sources.
- Introduce Z eligibility previews without treating internal Z attacks as stored
  move slots; add other battle mechanisms only with evidence of functioning code.
- For each later ROM, commit identity, evidence, capability matrix and regression
  fixtures together. Run all existing adapters before enabling a new writer.

中文：后续按“只读结构 → 规则完整性 → 可修改 → 游戏内重存回读”的顺序逐项开放。
新增 A 的规则时必须运行 B 的回归；不能仅用 A 的样本证明公共代码正确。详见
[Rocket field evidence](research/rocket-21-compatibility.md) 和
[battle lifecycle evidence](research/rocket-21-battle-forms.md)。
