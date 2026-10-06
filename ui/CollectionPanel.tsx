import { EntranceRoutes } from "./EntranceRoutes";
import { WildHeldDetails } from "./WildHeldDetails";
import { AcquisitionSourceFacts } from "./AcquisitionSourceFacts";
import { acquisitionTargetName } from "./acquisitionLabels";
import { CollectionPreparation } from "./CollectionPreparation";
import { CollectionPrerequisites } from "./CollectionPrerequisites";
import { breedingCoverageSummary } from "./CollectionBreeding";
import { EvolutionRuleDetails } from "./EvolutionRuleDetails";
import { ConditionDetails } from "./ConditionDetails";
import { useEffect, useRef, useState } from "react";
import { api } from "./api";
import { ClockDetails } from "./ClockDetails";
import { TradeDetails } from "./TradeDetails";
import { useI18n } from "./i18n";
import { collectionHtml } from "./collectionHtml";
import { dexReadSummary } from "./DexReadDetails";
import type {
  Catalog,
  CollectionPlan,
  GameMap,
  MapFocus,
  QueryTarget,
  Snapshot,
} from "./types";

export function CollectionPanel({
  catalog,
  maps,
  save,
  onTarget,
  onMap,
  onError,
}: {
  catalog: Catalog;
  maps: GameMap[];
  save: Snapshot | null;
  onTarget: (target: QueryTarget) => void;
  onMap: (id: string, focus?: MapFocus) => void;
  onError: (e: unknown) => void;
}) {
  const { t } = useI18n();
  const [basis, setBasis] = useState<"dex" | "individuals">("individuals");
  const [families, setFamilies] = useState(true);
  const [unknown, setUnknown] = useState(false);
  const [plan, setPlan] = useState<CollectionPlan | null>(null);
  const [query, setQuery] = useState("");
  const [status, setStatus] = useState("all");
  const [exporting, setExporting] = useState(false);
  const [tracing, setTracing] = useState(false);
  const generation = useRef(0);
  useEffect(() => {
    let active = true;
    generation.current++;
    setExporting(false);
    setTracing(false);
    setPlan(null);
    if (save)
      api<CollectionPlan>("collection", {
        basis,
        families,
        include_unknown_rewards: unknown,
      })
        .then((p) => {
          if (active) setPlan(p);
        })
        .catch((e) => {
          if (active) onError(e);
        });
    return () => {
      active = false;
    };
  }, [save, basis, families, unknown, catalog.profile.md5, onError]);
  const name = (v: QueryTarget) => acquisitionTargetName(v, catalog);
  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  return (
    <section className="collection-panel">
      <h2>{t("collection")}</h2>
      <p>{t("planHelp")}</p>
      <p className="small muted">{t("planFamilyHelp")}</p>
      {!save ? (
        <p>{t("planNeedsSave")}</p>
      ) : (
        <>
          <div className="map-tools">
            <label>
              {t("planBasis")}{" "}
              <select
                value={basis}
                onChange={(e) => setBasis(e.target.value as typeof basis)}
              >
                <option value="individuals">{t("planIndividuals")}</option>
                <option value="dex" disabled={save.dex.length === 0}>
                  {t("planDex")}
                </option>
              </select>
            </label>
            <label>
              <input
                type="checkbox"
                checked={families}
                onChange={(e) => setFamilies(e.target.checked)}
              />
              {t("planFamily")}
            </label>
            <label>
              <input
                type="checkbox"
                checked={unknown}
                onChange={(e) => setUnknown(e.target.checked)}
              />
              {t("planUnknownRewards")}
            </label>
          </div>
          <p className="small muted">{t("acqSaveOverlay")}</p>
          {basis === "dex" &&
            dexReadSummary(save.dex_status, t).map((line, i) => (
              <p className="small muted" key={i}>
                {line}
              </p>
            ))}
          {!plan ? (
            <p>{t("loading")}</p>
          ) : (
            <>
              {plan.clock && <ClockDetails report={plan.clock} />}
              {plan.breeding_coverage && (
                <div className="small muted">
                  {breedingCoverageSummary(plan.breeding_coverage, t).map(
                    (line, i) => (
                      <p key={i}>{line}</p>
                    ),
                  )}
                  <details>
                    <summary>{t("evidence")}</summary>
                    <pre>{JSON.stringify(plan.breeding_coverage, null, 2)}</pre>
                  </details>
                </div>
              )}
              <p>
                {t("planOwned")} {plan.owned_count} · {t("planMissing")}{" "}
                {plan.missing_count}
              </p>
              <div className="map-tools">
                <input
                  aria-label={t("planSearch")}
                  placeholder={t("planSearch")}
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                />
                <select
                  aria-label={t("planFilter")}
                  value={status}
                  onChange={(e) => setStatus(e.target.value)}
                >
                  <option value="all">{t("all")}</option>
                  {["available", "blocked", "unknown"].map((v) => (
                    <option key={v} value={v}>
                      {t(`acqStatus_${v}`)}
                    </option>
                  ))}
                </select>
                <button
                  disabled={tracing}
                  onClick={async () => {
                    const ticket = generation.current;
                    setTracing(true);
                    try {
                      const latest = await api<CollectionPlan>(
                        "collection_prerequisites",
                        {
                          expected_rom_md5: catalog.profile.md5,
                          query: {
                            basis,
                            families,
                            include_unknown_rewards: unknown,
                          },
                        },
                      );
                      if (
                        ticket === generation.current &&
                        latest.rom_md5 === catalog.profile.md5
                      )
                        setPlan(latest);
                    } catch (e) {
                      if (ticket === generation.current) onError(e);
                    } finally {
                      if (ticket === generation.current) setTracing(false);
                    }
                  }}
                >
                  {t(tracing ? "loading" : "planTracePrerequisites")}
                </button>
                <button
                  disabled={exporting}
                  onClick={async () => {
                    const ticket = generation.current;
                    setExporting(true);
                    try {
                      const latest = await api<CollectionPlan>(
                        "collection_export",
                        {
                          expected_rom_md5: catalog.profile.md5,
                          query: {
                            basis,
                            families,
                            include_unknown_rewards: unknown,
                          },
                        },
                      );
                      if (
                        ticket !== generation.current ||
                        latest.rom_md5 !== catalog.profile.md5
                      )
                        return;
                      const blob = new Blob(
                        [collectionHtml(latest, catalog, maps, t)],
                        { type: "text/html;charset=utf-8" },
                      );
                      const url = URL.createObjectURL(blob);
                      const a = document.createElement("a");
                      a.href = url;
                      a.download = `collection-${catalog.profile.id}.html`;
                      a.click();
                      setTimeout(() => URL.revokeObjectURL(url), 1000);
                    } catch (e) {
                      if (ticket === generation.current) onError(e);
                    } finally {
                      if (ticket === generation.current) setExporting(false);
                    }
                  }}
                >
                  {t(exporting ? "dependencyExporting" : "planExport")}
                </button>
              </div>
              {(plan.entrance_coverage?.truncated ||
                !!plan.entrance_coverage?.failed_scripts) && (
                <p className="small warning-text">
                  {t("dependencyAppendixLimit")}
                </p>
              )}
              {plan.entrance_coverage && (
                <details className="collection-navigation-evidence">
                  <summary>
                    {t("evidence")} · {t("navApproaches")}
                  </summary>
                  <pre>
                    {JSON.stringify(
                      {
                        coverage: plan.entrance_coverage,
                        diagnostics: plan.entrance_diagnostics,
                      },
                      null,
                      2,
                    )}
                  </pre>
                </details>
              )}
              <CollectionPrerequisites
                plan={plan}
                catalog={catalog}
                maps={maps}
                onTarget={onTarget}
                onMap={onMap}
              />
              {plan.regions.map((r, i) => {
                const tasks = r.tasks.filter(
                  (task) =>
                    (status === "all" ||
                      (task.source?.status ?? "unknown") === status) &&
                    `${name(task.target)} ${task.source?.map_id ? mapName(task.source.map_id) : ""} ${task.preparation?.source?.map_id ? mapName(task.preparation.source.map_id) : ""} ${task.family.map((id) => name({ kind: "species", id })).join(" ")} ${task.source?.related.map(name).join(" ") ?? ""}`
                      .toLowerCase()
                      .includes(query.toLowerCase()),
                );
                if (!tasks.length) return null;
                return (
                  <details key={i} open={!!query || plan.regions.length < 8}>
                    <summary>
                      {r.region === null
                        ? t("planNoRegion")
                        : (catalog.met_locations.find((l) => l.id === r.region)
                            ?.name ?? `#${r.region}`)}{" "}
                      · {tasks.length}
                    </summary>
                    {tasks.map((task, j) => {
                      const s = task.source;
                      const entrance = plan.entrances.find(
                        (e) => e.map_id === s?.map_id,
                      );
                      return (
                        <article className="encounter-card" key={j}>
                          <button
                            className="link-button"
                            onClick={() => onTarget(task.target)}
                          >
                            {name(task.target)} ↗
                          </button>
                          <small>
                            {t(`acqStatus_${s?.status ?? "unknown"}`)}
                          </small>
                          {s && (
                            <AcquisitionSourceFacts
                              source={s}
                              catalog={catalog}
                            />
                          )}
                          {!!s?.related.length && (
                            <div className="collection-related small">
                              {t("acqRelatedTargets")}:{" "}
                              {s.related.map((target, k) => (
                                <button
                                  className="link-button"
                                  key={k}
                                  onClick={() => onTarget(target)}
                                >
                                  {name(target)} ↗
                                </button>
                              ))}
                            </div>
                          )}
                          {!!task.family.length && (
                            <p className="small muted">
                              {t("planFamily")}:{" "}
                              {task.family
                                .map((id) => name({ kind: "species", id }))
                                .join(" / ")}
                            </p>
                          )}
                          {s?.in_scenario === false && (
                            <p className="small muted">
                              {t("clockOutsideScenario")}
                            </p>
                          )}
                          {s?.in_scenario === true && (
                            <p className="small">{t("clockInsideScenario")}</p>
                          )}
                          {s?.script_source && (
                            <p className="small muted">
                              {t("acqScriptSourceHelp")}
                            </p>
                          )}
                          {s?.evolution && (
                            <EvolutionRuleDetails
                              rule={s.evolution}
                              catalog={catalog}
                              maps={maps}
                              entrances={plan.entrances}
                              onTarget={onTarget}
                              onMap={onMap}
                            />
                          )}
                          <CollectionPreparation
                            preparation={task.preparation}
                            catalog={catalog}
                            maps={maps}
                            entrances={plan.entrances}
                            onTarget={onTarget}
                            onMap={onMap}
                            onError={onError}
                          />
                          <TradeDetails
                            mon={s?.script_source}
                            context={s?.trade_context}
                            catalog={catalog}
                            onSpecies={(id) =>
                              onTarget({ kind: "species", id })
                            }
                            onItem={(id) => onTarget({ kind: "item", id })}
                          />
                          {s && (
                            <WildHeldDetails
                              source={s}
                              item={task.target.id}
                              catalog={catalog}
                            />
                          )}
                          <ConditionDetails
                            checks={s?.conditions}
                            catalog={catalog}
                            onMap={onMap}
                            onTarget={onTarget}
                          />
                          {s?.receipt && (
                            <p className="small muted">
                              {t("acqGiftReceiptHelp")}
                            </p>
                          )}
                          {s &&
                            ["gift", "pc"].includes(s.kind) &&
                            s.receipt_flag == null && (
                              <p className="small muted">
                                {t("acqReceiptUnknown")}
                              </p>
                            )}
                          {s?.underfoot === true && (
                            <p className="small muted">
                              {t("mapHiddenUnderfoot")}
                            </p>
                          )}
                          {s?.map_id && (
                            <button
                              className="link-button"
                              onClick={() =>
                                onMap(
                                  s.map_id!,
                                  s.x !== null && s.y !== null
                                    ? { x: s.x, y: s.y }
                                    : undefined,
                                )
                              }
                            >
                              {mapName(s.map_id)}
                              {s.x !== null ? ` (${s.x}, ${s.y})` : ""} ↗
                            </button>
                          )}
                          <EntranceRoutes
                            entry={entrance}
                            maps={maps}
                            catalog={catalog}
                            onMap={onMap}
                            onTarget={onTarget}
                          />
                          {task.alternatives > 1 && (
                            <small>
                              {t("planAlternatives")} {task.alternatives} ·{" "}
                              {t("planClickSources")}
                            </small>
                          )}
                          <details>
                            <summary>{t("evidence")}</summary>
                            <pre>{JSON.stringify(task, null, 2)}</pre>
                          </details>
                        </article>
                      );
                    })}
                  </details>
                );
              })}
            </>
          )}
        </>
      )}
    </section>
  );
}
