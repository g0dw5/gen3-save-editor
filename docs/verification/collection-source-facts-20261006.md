# Collection source presentation / 收集来源展示

Scope: shared collection UI and standalone HTML, across the five registered
fingerprints. This increment changes presentation, not acquisition parsing or
receipt/access rules. Collection coverage remains **partial**.

## Behavior

- Show known acquisition type, level range, quantity, encounter-slot probability,
  periods and repeatability. Null remains unknown; held-item probability remains
  separate. Fishing rod names resolve from the loaded catalog/profile IDs.
- Related Pokémon, item and move references use their own runtime name tables and
  open the existing reference/map/back workflow. Search includes related names.
- HTML links point to an existing target task when present. Missing tasks are
  plain references, not fabricated prerequisites. Duplicate targets retain unique
  task anchors. All runtime text is escaped; reports contain no executable script.
- Period hours use verified clock boundaries from the current adapter/report.
  Missing or malformed boundaries show the period name only. Forced-night clock
  summaries do not imply that normal night hours constrain that override.

## Evidence and limits

`local_query_acquisition_and_collection_all_profiles` passed with exact local
ROMs: BW/DP 411 species and 415 shop rows each; Rocket 1,394/987; Ultimate
1,198/1,200; Mercury 1,435/1,470. The Mercury SAV fixture produced 767 missing
family goals, 116 regions and 452 entrance reports. Other profiles in this run
checked ROM-only queries; this is not five real-SAV or in-game route validation.

`test_query_collection_ui.py` passed query → map tile → exterior entrance → back,
reward links, prerequisite clues, preparation, escaped HTML and compact layout.
`test_collection_source_facts_ui.py` uses synthetic API fixtures to check source
facts, related targets, bilingual compact UI, HTML anchors, ROM switch and SAV
reload for five profile scenarios. Synthetic names are test-only, never bundled.

Period-label checks cover configured boundaries, rollover, adapter changes,
missing/malformed rules and the base table. Mercury's current verified boundaries
are unchanged; no new native clock calculation is claimed by this increment.

Full task dependencies, dynamic reachability, weekday/event refresh and route
optimality remain qualified in the capability matrix. No SAV was edited in this
presentation verification, and no new edited-SAV emulator round trip is claimed.

中文：清单与报告补齐已知来源事实及关联跳转，未知数量／概率／重复领取仍未知。
五份真实 ROM 查询通过，但本轮仅水银用了真实 SAV；界面五版本场景使用合成数据。
静态入口不证明当前可达，报告链接不生成新的任务依赖；不宣称全局最短路线。
