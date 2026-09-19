# Project memory

## Editing philosophy

User preference recorded on 2026-09-19. Apply it to editor development, advice,
and direct save edits performed by an agent.

- The purpose of editing is to save time and reduce excessive grinding and
  training. By default, aim for outcomes equivalent to legitimate in-game
  actions in the exact ROM being used, even when the hack has no Bank/HOME
  legality checks. Binary validity alone does not establish this equivalence.
- Prefer the game's own mechanism when it is known. For example, in the
  supported Rocket ROM, changing the nature override is equivalent to using
  the corresponding mint; preserve PID and trainer identity as the game does.
  Check the particular ROM's behavior instead of assuming official-game rules
  or another hack's implementation. Clearly identify unverified equivalence.
- Preserve unrelated individual data and history. Explain material differences
  between an edit and its in-game counterpart; do not claim an edit reproduces
  a game operation without evidence.
- For Pokédex completion, the user's preferred approach is to obtain Pokémon
  through normal gameplay, including catching, breeding and evolution. Do not
  default to creating Pokémon or directly marking the Pokédex complete.
- The previously customized Breloom (斗笠菇) used as a catching assistant is an
  explicitly accepted exception: one exceptional helper reduces the effort of
  subsequent normal captures. It is not blanket authorization for otherwise
  unobtainable Pokémon, moves, abilities, or fabricated collection records.
- Continue to honor explicitly requested exceptions and free editing. This
  preference is a default for decisions, not a new confirmation requirement or
  a reason to remove existing features. Structural integrity and save safety
  remain required for every edit, including exceptions.

用户原则：省肝、省练、省时间，默认追求游戏内正规操作可以达到的效果。抓宠斗笠菇
是已明确允许的一次性辅助特例；后续图鉴仍希望通过正常捕捉、孵蛋、进化完成，
而不是直接生成宝可梦或改亮图鉴。不要把这个特例扩大成所有修改的默认做法。

## ROM data and behavior

User preference recorded on 2026-09-19. Apply across every supported ROM and
future adapter, including reference displays, calculations and save editing.

- Treat the currently loaded ROM as the primary source of game content. Read
  names, descriptions, stats, evolutions, encounters, trainer parties, maps,
  artwork and other available data at runtime wherever practical. Do not extract
  a ROM once and ship those results as a built-in catalog, lookup list or asset
  bundle. Avoid duplicating readable ROM data in source code or release inputs.
- Reproduce the exact ROM's own calculation and selection rules as closely as
  possible, using its tables, parameters and control flow. Do not substitute
  official-game formulas or another hack's behavior without verification. For
  example, if the game obtains experience thresholds from a ROM table, read that
  table rather than generate thresholds from a standard growth formula.
- Keep necessary adapter configuration (fingerprints, addresses, layouts,
  bounds, encoding mappings and verified engine semantics) distinct from game
  content. Prefer reusable readers and configurable rules; verify differences
  per ROM and maintain cross-ROM regression coverage. Compare important derived
  results with native ROM routines when feasible; label unresolved behavior.
- Runtime caching is acceptable for performance, but tie it to the loaded ROM
  and invalidate it when the relevant ROM data changes. A cache must not become
  a bundled snapshot or a silent fallback to another version's content.
- UI labels/translations and explicitly requested external references, such as
  official base-stat comparisons, are separate from ROM content. Identify their
  source and purpose; never let them silently replace the loaded ROM's values.
  Do not bundle ROMs, saves or extracted game assets in releases.

用户原则：展示内容尽量直接读取当前 ROM，不要“读取一次 ROM，再把结果内置到
修改器”。计算逻辑也尽量复刻该 ROM 自身的表、参数和执行规则，例如经验阈值应
读取游戏使用的经验表，而不是套用通用公式。必要的偏移、格式和已验证规则属于
适配配置；运行时缓存必须随 ROM 数据变化失效。官方种族值等明确要求的外部参照
应单独标明来源，不能覆盖实际 ROM 数据。不同版本必须交叉验证，避免相互影响。
