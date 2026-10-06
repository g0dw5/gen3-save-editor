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
  };
  const connected = new Set([detail.species.id]);
  let discovered = true;
  while (discovered) {
    discovered = false;
    const groups = [
      ...relations.evolutions.map((e) => [e.source, e.target]),
      ...relations.battle_forms.map((e) => [e.source, e.target]),
      ...relations.form_families.map((f) => f.species),
    ];
    for (const group of groups)
      if (group.some((id) => connected.has(id)))
        for (const id of group)
          if (!connected.has(id)) {
            connected.add(id);
            discovered = true;
          }
  }
  const evolutions = unique(
    relations.evolutions.filter((e) => connected.has(e.source)),
    (e) =>
      JSON.stringify([
        e.source,
        e.target,
        e.method,
        e.condition,
        e.parameter,
        e.auxiliary ?? 0,
        [...(e.requirements ?? [])].sort(
          (a, b) => a.kind.localeCompare(b.kind) || a.value - b.value,
        ),
      ]),
  );
  const battles = unique(
    relations.battle_forms.filter((e) => connected.has(e.source)),
    (e) =>
      JSON.stringify([
        e.source,
        e.target,
        e.kind,
        e.trigger.kind,
        e.trigger.id,
      ]),
  );
  // Normal ancestry is a separate connected component. Form tables must not
  // promote every alternate form and its descendants into the ordinary tree.
  const primary = new Set([detail.species.id]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const e of evolutions)
      if (primary.has(e.source) || primary.has(e.target)) {
        if (!primary.has(e.source) || !primary.has(e.target)) changed = true;
        primary.add(e.source);
        primary.add(e.target);
      }
  }
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
  const battleIds = claim(
    battles
      .filter((e) => primary.has(e.source) || e.target === detail.species.id)
      .map((e) => e.target),
  );
  const families = relations.form_families
    .filter((f) => f.species.some((id) => connected.has(id)))
    .map((f) => ({
      ...f,
      members: claim(f.species),
    }))
    .filter((f) => f.members.length);
  // Unrelated/no-evolution entries still have a single selectable card.
  if (!shown.has(detail.species.id)) main.push(...claim([detail.species.id]));
  return { main, battleIds, families, evolutions, battles };
}
