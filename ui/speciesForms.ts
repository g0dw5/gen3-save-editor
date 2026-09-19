import type { Catalog, SpeciesDetail } from "./types";
import type { Key } from "./i18n";
import type { MappingEntry } from "./speciesMappings";

export interface VerifiedFormFamily {
  rom_md5: string;
  members: number[];
  labels: Key[];
}

type Translate = (key: Key) => string;
type ApprovedForm = { decision: MappingEntry; form: string } | undefined;

/** Presentation only: never change species, PID, held items or save bytes. */
export function formLabel(
  catalog: Catalog,
  id: number,
  t: Translate,
  options: {
    pid?: number;
    detail?: SpeciesDetail;
    approved?: ApprovedForm;
    verified?: VerifiedFormFamily[];
  } = {},
): string {
  const families =
    catalog.form_families ?? options.detail?.relations?.form_families ?? [];
  const battles =
    catalog.battle_forms ??
    options.detail?.relations?.battle_forms ??
    options.detail?.battle_forms ??
    [];
  const family = families.find((row) => row.species.includes(id));
  if (
    options.pid !== undefined &&
    id === catalog.profile.sprite_rules?.unown_species
  ) {
    const pid = options.pid >>> 0;
    const letter =
      (((pid >>> 24) & 3) * 64 +
        ((pid >>> 16) & 3) * 16 +
        ((pid >>> 8) & 3) * 4 +
        (pid & 3)) %
      28;
    return "ABCDEFGHIJKLMNOPQRSTUVWXYZ!?"[letter];
  }
  // Exact-ROM semantic rule: native family table plus separately verified ROM
  // sprites establish these identities. Modified stats are not identity evidence.
  const verified = options.verified?.find(
    (rule) =>
      rule.rom_md5 === catalog.profile.md5 &&
      family?.species.length === rule.members.length &&
      rule.labels.length === rule.members.length &&
      family.species.every((value, index) => value === rule.members[index]),
  );
  if (verified && family) return t(verified.labels[family.species.indexOf(id)]);
  const incoming = battles.filter((row) => row.target === id);
  if (incoming.length) {
    const labels = incoming.map((row) => {
      const item =
        row.trigger.kind === "held_item"
          ? catalog.items.find((item) => item.id === row.trigger.id)
          : undefined;
      const branch = item?.name.normalize("NFKC").match(/([XY])$/)?.[1];
      return `${t(row.kind)}${branch ? ` ${branch}` : ""}`;
    });
    return [...new Set(labels)].join(" / ");
  }
  if (options.approved?.decision.status === "direct" && options.approved.form) {
    return options.approved.form;
  }
  if (family) {
    const index = family.species.indexOf(id);
    return index === 0
      ? t("formBase")
      : t("formVariant").replace("{n}", String(index + 1));
  }
  if (battles.some((row) => row.source === id)) return t("formBase");
  const name = catalog.species.find((row) => row.id === id)?.name;
  if (
    name &&
    catalog.species.some((row) => row.id !== id && row.name === name)
  ) {
    return t("formSameName").replace("{n}", String(id));
  }
  return "";
}
