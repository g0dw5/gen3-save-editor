import { acquisitionSourceSummary } from "./acquisitionLabels";
import { AcquisitionSourceFacts } from "./AcquisitionSourceFacts";
import { CollectionBreeding, breedingRouteSummary } from "./CollectionBreeding";
import { ConditionDetails, conditionLabel } from "./ConditionDetails";
import { TradeDetails, tradeSummary } from "./TradeDetails";
import { evolutionLabel } from "./referenceLabels";
import { useI18n } from "./i18n";
import type {
  Catalog,
  CollectionPlan,
  CollectionPreparation as Preparation,
  GameMap,
  MapFocus,
  QueryTarget,
} from "./types";

export function preparationSummary(
  p: Preparation,
  catalog: Catalog,
  maps: GameMap[],
  t: (key: string) => string,
): string[] {
  const name = (id: number) =>
    catalog.species.find((s) => s.id === id)?.name ?? `#${id}`;
  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  const lines = [
    t(p.breeding ? "planBreedPreparation" : "planPreparation"),
    p.current_count > 0
      ? t("planPreparationOwned")
          .replace("{species}", name(p.origin))
          .replace("{count}", String(p.current_count))
      : t("planPreparationAcquire").replace("{species}", name(p.origin)),
  ];
  const s = p.source;
  if (p.breeding) lines.push(...breedingRouteSummary(p.breeding, catalog, t));
  if (s) {
    lines.push(
      ...acquisitionSourceSummary(s, catalog, t),
      t(`acqStatus_${s.status}`),
    );
    if (s.map_id)
      lines.push(
        `${mapName(s.map_id)}${s.x !== null ? ` (${s.x}, ${s.y})` : ""}`,
      );
    if (s.in_scenario != null)
      lines.push(
        t(s.in_scenario ? "clockInsideScenario" : "clockOutsideScenario"),
      );
    for (const check of s.conditions)
      lines.push(
        `${t(check.satisfied === true ? "planConditionYes" : check.satisfied === false ? "planConditionNo" : "acqStatus_unknown")}: ${conditionLabel(check, catalog, t)}${check.unresolved ? ` · ${t(check.unresolved)}` : ""}`,
      );
    if (s.script_source?.trade)
      lines.push(...tradeSummary(s.script_source, s.trade_context, catalog, t));
  }
  if (p.needs_hatching) lines.push(t("planPreparationHatch"));
  for (const step of p.steps)
    lines.push(
      `${name(step.from)} → ${name(step.evolution.target)}: ${evolutionLabel(step.evolution, catalog, catalog.type_names, t)}`,
    );
  lines.push(t("planPreparationHelp"));
  if (p.truncated) lines.push(t("planPreparationTruncated"));
  return lines;
}

export function CollectionPreparation({
  preparation: p,
  catalog,
  maps,
  entrances,
  onTarget,
  onMap,
  onError,
}: {
  preparation?: Preparation | null;
  catalog: Catalog;
  maps: GameMap[];
  entrances: CollectionPlan["entrances"];
  onTarget: (t: QueryTarget) => void;
  onMap: (id: string, focus?: MapFocus) => void;
  onError: (error: unknown) => void;
}) {
  const { t } = useI18n();
  if (!p) return null;
  const name = (target: QueryTarget) =>
    (target.kind === "species"
      ? catalog.species
      : target.kind === "move"
        ? catalog.moves
        : catalog.items
    ).find((s) => s.id === target.id)?.name ?? `#${target.id}`;
  const link = (target: QueryTarget) => (
    <button className="link-button" onClick={() => onTarget(target)}>
      {name(target)} ↗
    </button>
  );
  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  const s = p.source;
  const entrance = entrances.find((e) => e.map_id === s?.map_id);
  return (
    <section className="collection-preparation small">
      <strong>
        {t(p.breeding ? "planBreedPreparation" : "planPreparation")}
      </strong>
      <p>
        {t(
          p.current_count > 0
            ? "planPreparationExisting"
            : "planPreparationStart",
        )}
        : {link({ kind: "species", id: p.origin })}
        {p.current_count > 0 ? ` × ${p.current_count}` : ""}
      </p>
      {p.breeding && (
        <CollectionBreeding
          route={p.breeding}
          origin={p.origin}
          catalog={catalog}
          onTarget={onTarget}
          onError={onError}
        />
      )}
      {s && (
        <>
          <AcquisitionSourceFacts source={s} catalog={catalog} />
          <p>{t(`acqStatus_${s.status}`)}</p>
          {s.in_scenario != null && (
            <p>
              {t(
                s.in_scenario ? "clockInsideScenario" : "clockOutsideScenario",
              )}
            </p>
          )}
          <ConditionDetails
            checks={s.conditions}
            catalog={catalog}
            onMap={onMap}
            onTarget={onTarget}
          />
          <TradeDetails
            mon={s.script_source}
            context={s.trade_context}
            catalog={catalog}
            onSpecies={(id) => onTarget({ kind: "species", id })}
            onItem={(id) => onTarget({ kind: "item", id })}
          />
          {s.map_id && (
            <button
              className="link-button"
              onClick={() =>
                onMap(
                  s.map_id!,
                  s.x !== null && s.y !== null ? { x: s.x, y: s.y } : undefined,
                )
              }
            >
              {mapName(s.map_id)}
              {s.x !== null ? ` (${s.x}, ${s.y})` : ""} ↗
            </button>
          )}
          {!!entrance?.chains.length && (
            <p>
              {t("navApproaches")}:{" "}
              {entrance.chains[0].map((e, i) => (
                <span key={i}>
                  <button
                    className="link-button"
                    onClick={() =>
                      onMap(
                        e.from,
                        e.x !== null && e.y !== null
                          ? { x: e.x, y: e.y }
                          : undefined,
                      )
                    }
                  >
                    {mapName(e.from)}
                    {e.x !== null ? ` (${e.x}, ${e.y})` : ""}
                  </button>{" "}
                  →{" "}
                </span>
              ))}
              {mapName(entrance.map_id)}
            </p>
          )}
        </>
      )}
      {p.needs_hatching && <p>{t("planPreparationHatch")}</p>}
      <ol>
        {p.steps.map((step, i) => (
          <li key={i}>
            {link({ kind: "species", id: step.from })} →{" "}
            {link({ kind: "species", id: step.evolution.target })}
            <p>
              {evolutionLabel(step.evolution, catalog, catalog.type_names, t)}
            </p>
            {step.related.map((target) => (
              <span key={`${target.kind}-${target.id}`}>{link(target)} </span>
            ))}
            {(step.evolution.requirements ?? [])
              .filter((r) => r.kind === "map")
              .map((r) => {
                const id = `${r.value >>> 8}-${r.value & 255}`;
                return (
                  <button
                    key={id}
                    className="link-button"
                    onClick={() => onMap(id)}
                  >
                    {mapName(id)} ↗
                  </button>
                );
              })}
          </li>
        ))}
      </ol>
      <p className="muted">{t("planPreparationHelp")}</p>
      {p.truncated && <p className="muted">{t("planPreparationTruncated")}</p>}
    </section>
  );
}
