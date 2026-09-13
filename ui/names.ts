import aliases from "./data/dark-phantom-aliases.json";
import type { Catalog } from "./types";
import type { Locale } from "./i18n";

type Kind = keyof typeof aliases;
type Alias = {
  canonical_id: number;
  zh?: string;
  en?: string;
  zhHant?: string;
};
const supported = new Set([
  "0d9b129f7dd76895f79bb47ad7dec2fe",
  "cb2940215f4dafb1bef133c3af379f44",
]);

/** Aliases are version mappings, never a replacement for ROM IDs or names. */
export function namedOption(
  catalog: Catalog,
  kind: Kind,
  row: { id: number; name: string },
  locale: Locale,
) {
  const alias = supported.has(catalog.profile.md5)
    ? (aliases[kind] as Record<string, Alias>)[row.id]
    : undefined;
  const translated = locale === "zh" ? alias?.zh : alias?.en;
  return {
    value: row.id,
    label: `${row.name}${translated && translated !== row.name ? ` / ${translated}` : ""} #${row.id}`,
    search: [alias?.zh, alias?.en, alias?.zhHant].filter(
      (s): s is string => !!s,
    ),
  };
}

export function itemOption(
  catalog: Catalog,
  row: Catalog["items"][number],
  locale: Locale,
) {
  const option = namedOption(catalog, "items", row, locale);
  const move = catalog.moves.find((m) => m.id === row.tm_move);
  if (move) {
    const name = namedOption(catalog, "moves", move, locale);
    option.label += ` · ${name.label}`;
    option.search.push(name.label, ...name.search);
  }
  return option;
}
