import type { Catalog } from "./types";

/** Keep selection labels and stored IDs tied to the loaded ROM. */
export function romOption(row: { id: number; name: string }) {
  return { value: row.id, label: `${row.name} #${row.id}` };
}

export function itemOption(catalog: Catalog, row: Catalog["items"][number]) {
  const option = romOption(row);
  const move = catalog.moves.find((m) => m.id === row.tm_move);
  if (move) option.label += ` · ${romOption(move).label}`;
  return option;
}
