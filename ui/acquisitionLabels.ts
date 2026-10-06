import type { Catalog } from "./types";

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
