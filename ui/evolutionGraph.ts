import type { SpeciesDetail } from "./types";

type Relations = NonNullable<SpeciesDetail["relations"]>;
const unique = <T>(rows: T[], key: (row: T) => string): T[] => [
  ...new Map(rows.map((row) => [key(row), row])).values(),
];

/** Presentation only: one card per internal species ID, never merge forms by name. */
export function evolutionGraph(detail: SpeciesDetail) {
  const relations: Relations = detail.relations ?? {
    species: [detail.species.id],
    evolutions: detail.evolutions.map((row) => ({
      ...row,
      source: detail.species.id,
    })),
    battle_forms: detail.battle_forms ?? [],
    form_families: [],
    name_relations: [],
  };
  const evolutions = unique(relations.evolutions, (e) =>
    JSON.stringify([e.source, e.target, e.method, e.condition, e.parameter]),
  );
  const battles = unique(relations.battle_forms, (e) =>
    JSON.stringify([e.source, e.target, e.kind, e.trigger.kind, e.trigger.id]),
  );
  const names = unique(
    relations.name_relations,
    (e) => `${e.source}:${e.target}`,
  );
  const primary = new Set(evolutions.flatMap((e) => [e.source, e.target]));
  battles.forEach((e) => primary.add(e.source));
  relations.form_families.forEach((f) => {
    if (f.species.length) primary.add(f.species[0]);
  });
  names.forEach((e) => primary.add(e.source));
  // Roots first. Cyclic in-ROM transitions retain their incoming conditions and
  // are visited once rather than recursively duplicating a branch forever.
  const pending = [...primary].sort((a, b) => a - b);
  const main: number[] = [];
  while (pending.length) {
    const ready = pending.findIndex((id) =>
      evolutions.every((e) => e.target !== id || !pending.includes(e.source)),
    );
    main.push(...pending.splice(Math.max(0, ready), 1));
  }
  const shown = new Set(main);
  const claim = (ids: number[]) =>
    ids.filter((id) => {
      if (shown.has(id)) return false;
      shown.add(id);
      return true;
    });
  const battleIds = claim(battles.map((e) => e.target));
  const families = relations.form_families
    .map((f) => ({
      ...f,
      members: claim(f.species),
    }))
    .filter((f) => f.members.length);
  const nameIds = claim(names.map((e) => e.target));
  // Unrelated/no-evolution entries still have a single selectable card.
  if (!shown.has(detail.species.id)) main.push(...claim([detail.species.id]));
  return { main, battleIds, families, nameIds, evolutions, battles, names };
}
