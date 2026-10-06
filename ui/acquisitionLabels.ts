import type { AcquisitionSource, Catalog, QueryTarget } from "./types";

export function acquisitionTargetName(
  target: QueryTarget,
  catalog: Catalog,
): string {
  const rows =
    target.kind === "species"
      ? catalog.species
      : target.kind === "move"
        ? catalog.moves
        : catalog.items;
  return rows.find((row) => row.id === target.id)?.name ?? `#${target.id}`;
}

/** Normal period schedule comes from this adapter's verified clock rules.
 * Unconfigured clocks show a period name only, never another game's hours. */
export function encounterPeriodName(
  period: string,
  t: (key: string) => string,
  starts?: number[],
): string {
  const periods = ["morning", "day", "dusk", "night"];
  const labels: Record<string, string> = {
    base: "encounterBase",
    morning: "encounterMorning",
    day: "encounterDay",
    dusk: "encounterDusk",
    night: "encounterNight",
  };
  const label = t(labels[period] ?? period);
  const i = periods.indexOf(period);
  if (
    i < 0 ||
    starts?.length !== 4 ||
    starts.some(
      (hour, j) =>
        !Number.isInteger(hour) ||
        hour < 0 ||
        hour > 23 ||
        (j > 0 && hour <= starts[j - 1]),
    )
  )
    return label;
  const hour = (value: number) => String(value).padStart(2, "0");
  return `${label} (${hour(starts[i])}:00–${hour((starts[(i + 1) % 4] + 23) % 24)}:59)`;
}

/** Shared live-source facts for planning UI and standalone reports. Null is unknown,
 * not zero or a guessed default. Held-item probability is displayed separately. */
export function acquisitionSourceSummary(
  source: AcquisitionSource,
  catalog: Catalog,
  t: (key: string) => string,
): string[] {
  const lines = [
    `${acquisitionKindName(source.kind, catalog, t)}${source.encounter_method ? ` · ${acquisitionKindName(source.encounter_method, catalog, t)}` : ""}`,
  ];
  if (source.min_level != null) {
    const maximum = source.max_level;
    lines.push(
      `Lv. ${source.min_level}${maximum != null && maximum !== source.min_level ? `–${maximum}` : ""}`,
    );
  }
  if (source.quantity != null)
    lines.push(`${t("quantity")} × ${source.quantity}`);
  if (source.encounter_percent != null)
    lines.push(`${t("acqEncounterChance")} ${source.encounter_percent}%`);
  if (source.periods.length) {
    lines.push(
      `${t("planPeriods")}: ${source.periods.map((period) => encounterPeriodName(period, t, catalog.profile.clock?.starts)).join(" / ")}`,
    );
  }
  if (source.repeatable != null)
    lines.push(t(source.repeatable ? "acqRepeatable" : "acqOneTime"));
  return lines;
}

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
