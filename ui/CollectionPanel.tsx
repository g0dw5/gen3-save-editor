import { WildHeldDetails } from "./WildHeldDetails";
import { CollectionPreparation } from "./CollectionPreparation";
import { evolutionLabel } from "./referenceLabels";
import { ConditionDetails } from "./ConditionDetails";
import { useEffect, useState } from "react";
import { api } from "./api";
import { ClockDetails } from "./ClockDetails";
import { TradeDetails } from "./TradeDetails";
import { useI18n } from "./i18n";
import { collectionHtml } from "./collectionHtml";
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
  useEffect(() => {
    let active = true;
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
  const name = (v: QueryTarget) =>
    (v.kind === "species" ? catalog.species : catalog.items).find(
      (r) => r.id === v.id,
    )?.name ?? `#${v.id}`;
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
                <option
                  value="dex"
                  disabled={catalog.profile.capabilities?.dex === false}
                >
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
          {!plan ? (
            <p>{t("loading")}</p>
          ) : (
            <>
              {plan.clock && <ClockDetails report={plan.clock} />}
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
                  onClick={() => {
                    const blob = new Blob(
                      [collectionHtml(plan, catalog, maps, t)],
                      { type: "text/html;charset=utf-8" },
                    );
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement("a");
                    a.href = url;
                    a.download = `collection-${catalog.profile.id}.html`;
                    a.click();
                    setTimeout(() => URL.revokeObjectURL(url), 1000);
                  }}
                >
                  {t("planExport")}
                </button>
              </div>
              {plan.regions.map((r, i) => {
                const tasks = r.tasks.filter(
                  (task) =>
                    (status === "all" ||
                      (task.source?.status ?? "unknown") === status) &&
                    `${name(task.target)} ${task.source?.map_id ? mapName(task.source.map_id) : ""} ${task.preparation?.source?.map_id ? mapName(task.preparation.source.map_id) : ""} ${task.family.map((id) => name({ kind: "species", id })).join(" ")}`
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
                            <p className="small">
                              {evolutionLabel(
                                s.evolution,
                                catalog,
                                catalog.type_names,
                                t,
                              )}
                            </p>
                          )}
                          <CollectionPreparation
                            preparation={task.preparation}
                            catalog={catalog}
                            maps={maps}
                            entrances={plan.entrances}
                            onTarget={onTarget}
                            onMap={onMap}
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
                          {!!entrance?.chains.length && (
                            <p className="small">
                              {t("navApproaches")}:{" "}
                              {entrance.chains[0].map((e, k) => (
                                <span key={k}>
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
