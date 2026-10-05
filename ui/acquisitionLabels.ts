import type { Catalog } from "./types";

/** Labels are UI text; fishing rods use the currently loaded ROM. */
export function acquisitionKindName(
  kind: string,
  catalog: Catalog,
  t: (key: string) => string,
): string {
  const rod = ({ old_rod: 0, good_rod: 1, super_rod: 2 } as const)[
    kind as "old_rod"
  ];
  if (rod !== undefined)
    return (
      catalog.items.find((i) => i.id === catalog.profile.fishing_rods?.[rod])
        ?.name ?? t(kind)
    );
  return t(
    (
      {
        pickup: "mapPickups",
        hidden: "mapHidden",
        gift: "mapGifts",
        pc: "mapGifts",
        shop: "acqShop",
        wild_held: "acqWildHeld",
        wild_held_unreferenced: "heldUnreferenced",
        evolution: "acqEvolution",
        breeding_candidate: "acqBreeding",
        machine: "acqMachine",
        learn_level: "levelSource",
        learn_egg: "eggSource",
        learn_tm: "tm",
        learn_tutor: "tutor",
      } as Record<string, string>
    )[kind] ?? kind,
  );
}
