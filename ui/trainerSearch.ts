import type { Catalog, Opponent, World } from "./types";
import { en, zh, type Key } from "./i18n";

export type TrainerFacet = "role" | "location" | "battle" | "level";
export type TrainerFilters = Record<TrainerFacet, string>;
export const emptyTrainerFilters: TrainerFilters = {
  role: "",
  location: "",
  battle: "",
  level: "",
};
export interface TrainerTag {
  facet: TrainerFacet;
  value: string;
  label: string;
}
export interface TrainerEntry {
  trainer: Opponent;
  tags: TrainerTag[];
  search: string;
}

// Build once per ROM/language, not once per keystroke. Class names, region names
// and parties come from the ROM; map purposes are verified profile metadata.
export function indexTrainers(
  world: World | null,
  catalog: Catalog,
  t: (key: string) => string,
): TrainerEntry[] {
  if (!world) return [];
  const species = new Map(catalog.species.map((s) => [s.id, s.name]));
  const maps = new Map(world.maps.map((m) => [m.id, m]));
  const locations = new Map<number, World["trainer_locations"]["locations"]>();
  for (const link of world.trainer_locations.locations) {
    const list = locations.get(link.trainer_id) ?? [];
    list.push(link);
    locations.set(link.trainer_id, list);
  }
  return world.trainers.map((trainer) => {
    const tags: TrainerTag[] = [
      {
        facet: "role",
        value: trainer.class_name || `unknown:${trainer.class}`,
        label:
          trainer.class_name || `${t("trainerClassUnknown")} #${trainer.class}`,
      },
    ];
    const links = locations.get(trainer.id) ?? [];
    const aliases: string[] = [];
    for (const link of links) {
      const map = maps.get(link.map_id);
      if (!map) continue;
      const region = map.name.replace(/ · \d+-\d+$/, "");
      tags.push({
        facet: "location",
        value: `region:${map.region}`,
        label: region,
      });
      for (const group of world.map_groups.filter((g) =>
        g.map_ids.includes(map.id),
      )) {
        const key = `landmark_${group.kind}` as Key;
        tags.push({
          facet: "location",
          value: `place:${map.region}:${group.kind}`,
          label: `${region}${t(key)}`,
        });
        for (const label of [en[key], zh[key]]) {
          if (label) aliases.push(`${region}${label}`);
        }
      }
      aliases.push(link.map_id, link.map_name);
    }
    if (!links.length) {
      tags.push({
        facet: "location",
        value: "unresolved",
        label: t("contextUnresolved"),
      });
    }
    const battle = trainer.double_battle ? "double" : "single";
    const level = trainer.party.some((p) => p.level_rule === "party_max")
      ? "dynamic"
      : "fixed";
    tags.push({ facet: "battle", value: battle, label: t(`battle_${battle}`) });
    tags.push({ facet: "level", value: level, label: t(`level_${level}`) });
    const unique = [
      ...new Map(
        tags.map((tag) => [`${tag.facet}:${tag.value}`, tag]),
      ).values(),
    ];
    // English equivalents are search aliases for ROM terms, not inferred roles.
    const roleAliases = (trainer.class_name ?? "")
      .replace(/四天王/g, " elite four ")
      .replace(/联盟冠军/g, " champion ")
      .replace(/馆主/g, " gym leader ");
    return {
      trainer,
      tags: unique,
      search: [
        trainer.id,
        trainer.name,
        roleAliases,
        ...unique.map((tag) => tag.label),
        ...aliases,
        battle,
        level,
        ...trainer.party.map((p) => species.get(p.species) ?? ""),
      ]
        .join(" ")
        .toLocaleLowerCase(),
    };
  });
}

export function searchTrainers(
  entries: TrainerEntry[],
  query: string,
  filters: TrainerFilters,
) {
  const tokens = query.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
  return entries.filter(
    (entry) =>
      tokens.every((token) => entry.search.includes(token)) &&
      Object.entries(filters).every(
        ([facet, value]) =>
          !value ||
          entry.tags.some((tag) => tag.facet === facet && tag.value === value),
      ),
  );
}

// Counts respect all other facets and the query, so incompatible combinations
// stay understandable. Keep selected zero-result options available for clearing.
export function trainerFacetOptions(
  entries: TrainerEntry[],
  query: string,
  filters: TrainerFilters,
  facet: TrainerFacet,
) {
  const options = new Map<string, TrainerTag & { count: number }>();
  for (const entry of entries) {
    for (const tag of entry.tags.filter((tag) => tag.facet === facet)) {
      if (!options.has(tag.value)) options.set(tag.value, { ...tag, count: 0 });
    }
  }
  for (const entry of searchTrainers(entries, query, {
    ...filters,
    [facet]: "",
  })) {
    for (const tag of entry.tags.filter((tag) => tag.facet === facet))
      options.get(tag.value)!.count++;
  }
  return [...options.values()].sort((a, b) =>
    a.label.localeCompare(b.label, "zh-CN"),
  );
}
