# Mercury Wiki audit and native journal

Developer record; excluded from player distributions. Exact input: Mercury FC
1.2, MD5 `f323df1792ac68462a34b42fe8571533`. Source ROM/SAV files are read-only.

## Reference and boundaries

Compared the [Azoth Wiki](https://sum-light.github.io/azoth-wiki/), its side-quest,
time-event, broadcast and tutor pages with the current reader and local ROM.
The site identifies itself as unofficial. It is a useful presentation/content
check, not authority to override this exact ROM's tables or execution rules.

- Wiki groups 96 quest articles. The ROM journal iterates 100 records, including
  duplicated titles/main-story branches. Keep each native flag pair independent;
  neither count proves coverage of unique side quests.
- Wiki's indexed journal content and the exact ROM have different page counts.
  This ROM has 306 pages. No Wiki text, extracted catalog or images were bundled.
- Wiki marks main-story walkthrough/important trainer rosters as planned. Do not
  advertise those sections as an independent complete trainer/story oracle.
- Weekly events and broadcasts are useful checks, but broadcast-specific wild
  pools, all weekday events, full quest prerequisites and story reachability are
  still incomplete. Existing Mercury time filters retain its saved virtual clock
  and verified time selector; other ROMs' time rules are not reused.

## Native rules

| Role | ROM offset / rule |
|---|---|
| Quest records | `0xE3BA28`, 100 × 20 bytes; name/objective pointers at +0/+4 |
| Receipt flags | Accept at +10, complete at +12; +8 is icon metadata, not a region |
| Journal books | `0x8F5644`, 8-byte records: pages, appearance flag, page count |
| Journal page | 16 bytes: title/text, condition ID, threshold, extra flag, kind, completed-check policy |
| Native quest iteration | `0xDF8530` / `0xDF8572`, count 100 |
| Eligible journal | `0xDF7BF0`: accepted, completed or an explicitly nonzero appearance flag |
| Page visibility | `0xDF7C40`; completed + zero policy bypasses remaining checks; variable comparison uses **>=**, not equality |

The reader matches native flag/variable checks, including extended save banks.
Missing SAV context produces unknown visibility. Titles/objectives/pages are read
from the loaded ROM; NPC/map positions come from bounded referenced scripts.
Acceptance/complete flags do not prove every map is currently reachable. Native
stages are not inferred from a monotonically increasing number or bag holdings.

## Validation

`adventure::journal::tests::exact_rom_journal_native_vectors` emits private
vectors for five main/extension save patterns. `verify_mercury_journal.py`
executes the complete native page predicate in mGBA: **1,530 comparisons passed**
(306 × 5), no stubbed predicate and no unresolved expected values.

The current private SAV shows one accepted journal record and no completed ones;
90 of 100 native records have an identified scene position. These are record and
parsed-position counts, not a claim of 90 accessible or fully modeled quests.

Five-fingerprint read-only reference/guide regressions and bilingual UI scenarios
passed. UI checks cover searching title/objective/pages, current/later visibility,
collapsed later logs and source → map → return. Full quest dependency DAGs and
branch-perfect walkthrough narration remain partial.
