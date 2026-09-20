# Evolution viewer identity / 进化树条目身份

The old viewer rendered both endpoints of every evolution and battle edge,
then displayed whole form-family lists. Pikachu therefore appeared repeatedly
alongside additional entries from its native cosmetic-form family. This was a
presentation problem; distinct internal species IDs must not be merged by name
or by their reviewed official reference.

The shared viewer now allocates one card per internal species ID. Incoming
conditions appear under that card. Exact duplicate relation records are merged
by semantic identity (source, target, method/condition, parameter or battle
trigger), while different conditions remain visible. References back to a source
are lightweight navigation links, not additional Pokémon cards.

Main entries are ordered with available roots first. Cyclic ROM transformations
are bounded and retain their condition links. Battle targets appear once;
remaining form-family members are folded by default and automatically open when
the current entry is inside that family. Family members already represented by
an evolution or battle card are excluded from those folded lists. Name-only
links remain explicitly unverified and never affect legal origin calculations.

Tests cover exact BW, DP and Rocket Pikachu catalogs, cosmetic-form expansion,
Mega and Deoxys navigation, overlapping families, duplicated records with distinct
ROM offsets, alternate evolution conditions, cycles and narrow windows.

中文：相同内部编号只画一张卡片，保留多种进化条件；不同编号即使同名也不会被
合并。皮卡丘的装扮等其他形态折叠展示，已出现在主路线的本体不再重复显示。
关系图本身与存档写入规则不变。
