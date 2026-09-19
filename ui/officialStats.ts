import data from "./data/official-stats.json";
import type { Catalog, SpeciesDetail } from "./types";

export interface OfficialStats {
  key: string;
  dex: number;
  name: string;
  form: string;
  generation: number;
  stats: number[];
}
export const officialStats: OfficialStats[] = data.rows.map((row) => ({
  key: `${row[0]}:${row[2]}`,
  dex: Number(row[0]),
  name: String(row[1]),
  form: String(row[2]),
  generation: Number(row[3]),
  stats: row.slice(4).map(Number),
}));
export function referenceSource(generation: number) {
  const source = data.sources.find(
    (source) => source.generation === generation,
  )!;
  return source.revision
    ? `https://wiki.52poke.com/index.php?oldid=${source.revision}`
    : source.url;
}
const normalized = (value: string) =>
  value.normalize("NFKC").replace(/[\s·・]/g, "");

/** Names, not reused National Dex slots, identify reference candidates.
 * Battle forms must not silently inherit their ordinary form's official stats.
 * Ambiguous translations/forms require the user's explicit reference selection.
 */
export function suggestedReference(
  detail: SpeciesDetail,
  catalog: Catalog,
): OfficialStats | undefined {
  const incoming =
    detail.battle_forms?.filter((form) => form.target === detail.species.id) ??
    [];
  if (incoming.length) {
    const candidates = incoming.flatMap((form) => {
      const base = catalog.species.find((row) => row.id === form.source);
      if (!base) return [];
      return officialStats.filter(
        (row) =>
          normalized(row.name) === normalized(base.name) &&
          (form.kind === "mega"
            ? row.form.startsWith("超级")
            : row.form.includes("原始")),
      );
    });
    const unique = [
      ...new Map(candidates.map((row) => [row.key, row])).values(),
    ];
    // X/Y and other multiple-form branches are not inferred from slot ordering.
    return unique.length === 1 ? unique[0] : undefined;
  }
  const family = detail.relations?.form_families.find((family) =>
    family.species.includes(detail.species.id),
  );
  if (family && family.species[0] !== detail.species.id) return undefined;
  const candidates = officialStats.filter(
    (row) =>
      !row.form && normalized(row.name) === normalized(detail.species.name),
  );
  return candidates.length === 1 ? candidates[0] : undefined;
}
