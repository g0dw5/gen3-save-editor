import { useI18n } from "./i18n";
import type { AcquisitionSource, Catalog, PokemonSource } from "./types";

export function tradeSummary(
  mon: PokemonSource,
  context: AcquisitionSource["trade_context"] | undefined,
  catalog: Catalog,
  t: (key: string) => string,
): string[] {
  if (!mon.trade) return [];
  const donor =
    catalog.species.find((s) => s.id === mon.trade!.requested_species)?.name ??
    `#${mon.trade.requested_species}`;
  const lines = [
    t("tradeGiveSummary").replace("{species}", donor),
    t("tradeLevelRule"),
    t("tradeOfferHelp"),
  ];
  if (mon.held_item)
    lines.push(
      `${t("held_item")}: ${catalog.items.find((i) => i.id === mon.held_item)?.name ?? `#${mon.held_item}`}`,
    );
  if (context) {
    const key = context.party_levels.length
      ? "tradePartyReady"
      : context.box_levels.length
        ? "tradeBoxReady"
        : "tradeDonorMissing";
    lines.push(
      t(key).replace(
        "{levels}",
        (context.party_levels.length
          ? context.party_levels
          : context.box_levels
        )
          .map((l) => `Lv. ${l}`)
          .join(" / "),
      ),
    );
  }
  return lines;
}

export function TradeDetails({
  mon,
  context,
  catalog,
  onSpecies,
  onItem,
}: {
  mon?: PokemonSource | null;
  context?: AcquisitionSource["trade_context"];
  catalog: Catalog;
  onSpecies?: (id: number) => void;
  onItem?: (id: number) => void;
}) {
  const { t } = useI18n();
  if (!mon?.trade) return null;
  const donor = mon.trade.requested_species;
  const heldName = mon.held_item
    ? (catalog.items.find((i) => i.id === mon.held_item)?.name ??
      `#${mon.held_item}`)
    : null;
  const heldLine = heldName ? `${t("held_item")}: ${heldName}` : null;
  return (
    <div className="small trade-details">
      <p>
        {t("tradeGive")}:{" "}
        <button className="link-button" onClick={() => onSpecies?.(donor)}>
          {catalog.species.find((s) => s.id === donor)?.name ?? `#${donor}`} ↗
        </button>
      </p>
      {tradeSummary(mon, context, catalog, t)
        .slice(1)
        .filter((line) => line !== heldLine)
        .map((line, i) => (
          <p className="muted" key={i}>
            {line}
          </p>
        ))}
      {heldName && (
        <p>
          {t("held_item")}:{" "}
          <button
            className="link-button"
            onClick={() => onItem?.(mon.held_item!)}
          >
            {heldName} ↗
          </button>
        </p>
      )}
    </div>
  );
}
