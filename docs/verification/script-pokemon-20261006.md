# Script Pokémon source increment / 脚本宝可梦来源增量

This increment connects bounded parsed command inputs to the existing acquisition,
map and collection workflow. It does not complete exchanges, native custom gifts,
roamers, event receipts, quest prerequisites or current reachability.

The runtime decoder is shared; each exact fingerprint supplies command-format
rules and the address of its native gift-egg level instruction. Rocket's wild
command occupies 11 bytes; ordinary Emerald commands occupy 6. Mercury uses 6
bytes for a single opponent or 18 for its sentinel-selected double encounter.
Mercury species operands use VarGet; the other wild handlers use literal values.
Extra battle-generation parameters are not presented as final individual values.

The source carries resolved species, command-input level/item, branch evidence
and the member index. Object and coordinate scripts retain their actual tile;
map-level sources without a verified tile remain separate. Native calls invalidate
unproven variable/flag state. Unknown input does not become a fabricated record.
The query retains partial/undetermined status even when a tile is known.

Gift-egg levels are read from the currently loaded constructor instruction,
rather than assumed to be five. A private synthetic-RAM Unicorn probe ran native
egg creation up to delivery, copied the generated record and read it with the
native getters: BW/DP/Mercury species 1 eggs were level 1; Rocket/Ultimate eggs
were level 5. All were marked eggs. This narrow probe does not prove delivery,
all species generation, gameplay access or a save-edit round trip. No input
ROM or SAV was modified.

Public regression fixtures exercise all five adapters' operand widths,
literal/variable selection, Mercury double members, runtime egg-level changes
and invalid instructions. A map fixture verifies NPC coordinates and separates
unplaced egg sources without inventing a tile. Prior query/map/back/planner/HTML
and NPC receipt browser fixtures remain applicable. Detailed local build results
are recorded next to the delivered application in BUILD-INFO.txt.

五款精确指纹共用来源模型，差异保留在适配规则中。查询、地图与收集 HTML 共享
已解析来源及未知提示；没有可靠格位时不猜测 NPC 位置。当前仅补齐有界脚本来源
的关联，交换、自定义原生赠送、完整发放结果、领奖状态、剧情和可达性仍有缺口。
不把参数解析等同完整支持，不写 ROM／SAV，也不宣称新的模拟器存档回读验证。
