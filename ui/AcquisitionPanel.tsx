import { StaticBattleDetails } from "./StaticBattleDetails";
import { acquisitionKindName, encounterPeriodName } from "./acquisitionLabels";
import { BreedingPanel } from "./BreedingPanel";
import { WildHeldDetails } from "./WildHeldDetails";
import { ConditionDetails } from "./ConditionDetails";
import { useEffect, useMemo, useState } from "react";
import { api } from "./api";
import { ClockDetails } from "./ClockDetails";
import { TradeDetails } from "./TradeDetails";
import { useI18n } from "./i18n";
import { EvolutionRuleDetails } from "./EvolutionRuleDetails";
import { EvolutionUsesPanel } from "./EvolutionUsesPanel";
import type {
  AcquisitionReport,
  AcquisitionSource,
  Catalog,
  GameMap,
  MapFocus,
  QueryTarget,
  Snapshot,
  Template,
} from "./types";

export function AcquisitionPanel({
  target,
  catalog,
  maps,
  save,
  onTarget,
  onMap,
  onError,
  onTemplate,
}: {
  target: QueryTarget;
  catalog: Catalog;
  maps: GameMap[];
  save: Snapshot | null;
  onTarget: (target: QueryTarget) => void;
  onMap: (id: string, focus?: MapFocus) => void;
  onError: (error: unknown) => void;
  onTemplate?: (template: Template) => void;
}) {
  const { t } = useI18n();
  const [report, setReport] = useState<AcquisitionReport | null>(null);
  const [query, setQuery] = useState("");
  const [limit, setLimit] = useState(6);
  const [time, setTime] = useState(() => (save ? "save" : "all"));
  const hour = time !== "all" && time !== "save" ? Number(time) : null;
  useEffect(() => {
    setTime(save ? "save" : "all");
  }, [catalog.profile.md5]);
  useEffect(() => {
    let active = true;
    setReport(null);
    setQuery("");
    setLimit(6);
    api<AcquisitionReport>("acquisition", {
      ...target,
      hour,
      use_save_clock: time === "save",
    })
      .then((r) => {
        if (active) setReport(r);
      })
      .catch((e) => {
        if (active) onError(e);
      });
    return () => {
      active = false;
    };
  }, [target.kind, target.id, catalog.profile.md5, save, onError, time]);
  const name = (v: QueryTarget) =>
    (v.kind === "species"
      ? catalog.species
      : v.kind === "item"
        ? catalog.items
        : catalog.moves
    ).find((x) => x.id === v.id)?.name ?? `#${v.id}`;
  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  const kindName = (kind: string) => acquisitionKindName(kind, catalog, t);
  const filtered = useMemo(
    () =>
      (report?.sources ?? []).filter((s) =>
        `${kindName(s.kind)} ${s.map_id ? mapName(s.map_id) : ""} ${s.related.map(name).join(" ")}`
          .toLowerCase()
          .includes(query.toLowerCase()),
      ),
    [report, query, catalog, maps, t],
  );
  const draft = (s: AcquisitionSource): Template => ({
    species: target.id,
    level: s.min_level ?? 5,
    ...(s.region !== null ? { met_location: s.region } : {}),
    egg: s.kind === "egg",
  });
  return (
    <section className="acquisition-panel">
      <h3>{t("acqTitle")}</h3>
      <EvolutionUsesPanel
        rows={report?.evolution_uses ?? []}
        catalog={catalog}
        maps={maps}
        onTarget={onTarget}
        onMap={onMap}
      />
      {catalog.profile.clock && (
        <fieldset className="clock-scenario">
          <legend>{t("clockScenario")}</legend>

          <label>
            {t("clockHour")}{" "}
            <select value={time} onChange={(e) => setTime(e.target.value)}>
              <option value="save" disabled={!save}>
                {t("clockUseSave")}
              </option>
              <option value="all">{t("clockAllPeriods")}</option>
              {Array.from({ length: 24 }, (_, h) => (
                <option key={h} value={h}>
                  {h}:00
                </option>
              ))}
            </select>
          </label>
          {report?.clock && <ClockDetails report={report.clock} />}
          {time !== "save" && time !== "all" && (
            <p className="small muted">{t("clockForcedNightUnknown")}</p>
          )}
        </fieldset>
      )}
      <details className="reference-help">
        <summary>{t("referenceSourceHelp")}</summary>
        <p className="small muted">{t("acqCoverage")}</p>
        {save && <p className="small muted">{t("acqSaveOverlay")}</p>}
      </details>
      {target.kind === "species" && catalog.profile.breeding && (
        <BreedingPanel
          catalog={catalog}
          save={save}
          species={target.id}
          maps={maps}
          onTarget={onTarget}
          onMap={onMap}
          onError={onError}
        />
      )}
      <input
        className="acquisition-search"
        aria-label={t("acqSearch")}
        placeholder={t("acqSearch")}
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setLimit(6);
        }}
      />
      {!report ? (
        <p>{t("loading")}</p>
      ) : !filtered.length ? (
        <p className="muted">{t("acqNoSource")}</p>
      ) : (
        filtered.slice(0, limit).map((s, i) => {
          const canDraft =
            target.kind === "species" &&
            s.min_level !== null &&
            onTemplate &&
            catalog.profile.capabilities?.save_edit !== false;
          return (
            <article
              className="encounter-card"
              key={`${s.offset}-${i}`}
              draggable={!!canDraft}
              onDragStart={(e) => {
                if (canDraft)
                  e.dataTransfer.setData(
                    "application/x-gen3",
                    JSON.stringify({
                      kind: "template",
                      template: draft(s),
                      profile: catalog.profile.md5,
                    }),
                  );
              }}
            >
              <div>
                <strong>
                  {kindName(s.kind)}
                  {s.encounter_method
                    ? ` · ${kindName(s.encounter_method)}`
                    : ""}
                </strong>
                {save && (
                  <small className={`source-status status-${s.status}`}>
                    {t(`acqStatus_${s.status}`)}
                  </small>
                )}
              </div>
              <StaticBattleDetails
                mon={s.script_source}
                catalog={catalog}
                onTarget={onTarget}
              />
              <TradeDetails
                mon={s.script_source}
                context={s.trade_context}
                catalog={catalog}
                onSpecies={(id) => onTarget({ kind: "species", id })}
                onItem={(id) => onTarget({ kind: "item", id })}
              />
              {s.underfoot === true && (
                <p className="small muted">{t("mapHiddenUnderfoot")}</p>
              )}
              {save &&
                ["pickup", "hidden", "gift", "pc"].includes(s.kind) &&
                s.receipt_flag == null && (
                  <p className="small muted">{t("acqReceiptUnknown")}</p>
                )}
              {s.map_id && (
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
                  {s.x !== null
                    ? ` · (${s.x}, ${s.y})`
                    : ` · ${t("acqNoTile")}`}{" "}
                  ↗
                </button>
              )}
              <div className="source-related">
                {s.related.map((r, j) => (
                  <button
                    className="link-button"
                    key={j}
                    onClick={() => onTarget(r)}
                  >
                    {name(r)} ↗
                  </button>
                ))}
              </div>
              {s.min_level !== null && (
                <span>
                  Lv. {s.min_level}
                  {s.max_level !== null && s.max_level !== s.min_level
                    ? `–${s.max_level}`
                    : ""}
                </span>
              )}
              {s.quantity !== null && <span> × {s.quantity}</span>}
              {s.encounter_percent !== null && (
                <p>
                  {t("acqEncounterChance")} {s.encounter_percent}%
                </p>
              )}
              <WildHeldDetails source={s} item={target.id} catalog={catalog} />
              {!!s.periods.length && (
                <p>
                  {s.periods
                    .map((period) =>
                      encounterPeriodName(
                        period,
                        t,
                        catalog.profile.clock?.starts,
                      ),
                    )
                    .join(" / ")}
                </p>
              )}
              {s.in_scenario === false && (
                <p className="warning-text small">
                  {t("clockOutsideScenario")}
                </p>
              )}
              {s.evolution && (
                <EvolutionRuleDetails
                  rule={s.evolution}
                  catalog={catalog}
                  maps={maps}
                  onTarget={onTarget}
                  onMap={onMap}
                />
              )}
              {s.kind === "breeding_candidate" && (
                <p className="small muted">{t("acqBreedHelp")}</p>
              )}
              {s.repeatable !== null && (
                <small>
                  {t(s.repeatable ? "acqRepeatable" : "acqOneTime")}
                </small>
              )}
              <ConditionDetails
                checks={s.conditions}
                catalog={catalog}
                onMap={onMap}
                onTarget={onTarget}
              />
              {canDraft && (
                <button
                  className="link-button small"
                  onClick={() => onTemplate!(draft(s))}
                >
                  {t("pickDestination")} ↗
                </button>
              )}
              <details>
                <summary>{t("evidence")}</summary>
                <pre>{JSON.stringify(s, null, 2)}</pre>
              </details>
            </article>
          );
        })
      )}
      {filtered.length > limit && (
        <button onClick={() => setLimit((n) => n + 6)}>
          {t("acqMore")} ({filtered.length - limit})
        </button>
      )}
    </section>
  );
}
