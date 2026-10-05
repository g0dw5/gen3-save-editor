import type { AcquisitionSource, Catalog } from "./types";
import { useI18n } from "./i18n";

type Translate = (key: string) => string;
export function heldSummary(
  source: AcquisitionSource,
  item: number,
  catalog: Catalog,
  t: Translate,
): string[] {
  if (!["wild_held", "wild_held_unreferenced"].includes(source.kind)) return [];
  if (source.kind === "wild_held_unreferenced")
    return [t("heldUnreferencedHelp")];
  const context = source.held_context;
  if (!context)
    return [
      `${t("acqHeldChance")}: ${t("unresolved")}`,
      t("heldNativeUnknown"),
    ];
  const chance = (distribution: typeof context.baseline) => {
    const count =
      distribution.outcomes.find((o) => o.item === item)?.count ?? 0;
    if (count === 0) return "0%";
    if (count === distribution.denominator) return "100%";
    return `≈ ${((100 * count) / distribution.denominator).toFixed(2)}%`;
  };
  const rows = [`${t("heldBaseline")}: ${chance(context.baseline)}`];
  if (context.current_party) {
    const lead = context.current_party;
    const species =
      catalog.species.find((s) => s.id === lead.lead_species)?.name ??
      `#${lead.lead_species}`;
    const ability =
      catalog.abilities.find((a) => a.id === lead.lead_ability)?.name ??
      `#${lead.lead_ability}`;
    rows.push(
      `${t("heldCurrentLead")}: ${species} · ${ability}${lead.lead_egg ? ` · ${t("egg")}` : ""} → ${chance(lead)}`,
    );
  }
  rows.push(t("acqHeldHelp"), t("heldStaticContextHelp"));
  return rows;
}
export function WildHeldDetails({
  source,
  item,
  catalog,
}: {
  source: AcquisitionSource;
  item: number;
  catalog: Catalog;
}) {
  const { t } = useI18n();
  const lines = heldSummary(source, item, catalog, t);
  if (!lines.length) return null;
  return (
    <div className="wild-held-details small">
      {lines.map((line, i) => (
        <p className={i > 1 ? "muted" : ""} key={i}>
          {line}
        </p>
      ))}
    </div>
  );
}
