import type { Catalog, SpeciesDetail } from "./types";

/** Resolve references using this ROM; unknown conditions remain explicit. */
export function evolutionLabel(
  evolution: SpeciesDetail["evolutions"][number],
  catalog: Catalog,
  typeNames: string[],
  t: (key: string) => string,
): string {
  const common = [
    "unknown",
    "friendship",
    "friendship_day",
    "friendship_night",
    "level",
    "trade",
    "trade_item",
    "item",
    "attack_higher",
    "attack_equal",
    "defense_higher",
    "personality",
    "personality",
    "level",
    "shedinja",
    "beauty",
  ];
  const kind = evolution.condition ?? common[evolution.method] ?? "unknown";
  const value = evolution.parameter;
  const values: Record<string, string> = {
    n: String(value),
    item:
      catalog.items.find((item) => item.id === value)?.name ?? t("unknownItem"),
    move:
      catalog.moves.find((move) => move.id === value)?.name ?? t("unknownMove"),
    type: typeNames[value] ?? t("unknown"),
    species:
      catalog.species.find((species) => species.id === value)?.name ??
      t("unknown"),
    region:
      catalog.met_locations.find((region) => region.id === value)?.name ??
      t("unresolved"),
  };
  return t(`evo_${kind}`).replace(
    /\{(\w+)\}/g,
    (_, key: string) => values[key] ?? t("unknown"),
  );
}

export function itemPocketLabel(
  category: number,
  catalog: Catalog,
  t: (key: string) => string,
): string {
  const pocket = catalog.profile.save?.pockets.find(
    (pocket) => pocket.category === category && pocket.id !== "pc",
  );
  // Older browser fixtures predate the serialized profile pocket list.
  const legacy = ["unknown", "items", "balls", "tmhm", "key_items", "berries"];
  return t(
    pocket?.id ??
      (catalog.profile.save ? "unknown" : (legacy[category] ?? "unknown")),
  );
}
