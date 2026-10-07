import { useEffect, useState } from "react";
import { acquisitionKindName, encounterPeriodName } from "./acquisitionLabels";
import { clockSummary } from "./ClockDetails";
import { api } from "./api";
import { Sprite } from "./components";
import { useI18n } from "./i18n";
import { speciesDisplayName } from "./speciesDisplay";
import type {
  AcquisitionReport,
  Catalog,
  Encounter,
  Snapshot,
  Template,
} from "./types";

/** Keep selectors/time tables apart. Weights are conditional on one method/table. */
export function encounterTables(rows: Encounter[], period: string) {
  const filtered = rows.filter(
    (e) =>
      period === "all" ||
      !e.periods?.length ||
      e.periods.includes(period) ||
      (e.periods.includes("base") &&
        !rows.some(
          (other) =>
            other.method === e.method && other.periods?.includes(period),
        )),
  );
  const groups = new Map<
    string,
    {
      method: string;
      periods: string[];
      selector: Encounter["selector"];
      rows: Encounter[];
    }
  >();
  for (const e of filtered) {
    const key = JSON.stringify([e.method, e.periods ?? [], e.selector ?? null]);
    const table = groups.get(key) ?? {
      method: e.method,
      periods: e.periods ?? [],
      selector: e.selector,
      rows: [],
    };
    const same = table.rows.find(
      (r) => r.species === e.species && r.weight !== null && e.weight !== null,
    );
    if (same) {
      same.weight! += e.weight!;
      same.min_level = Math.min(same.min_level, e.min_level);
      same.max_level = Math.max(same.max_level, e.max_level);
    } else table.rows.push({ ...e });
    groups.set(key, table);
  }
  return [...groups.values()];
}
export function MapEncounters({
  catalog,
  save,
  encounters,
  onSpecies,
  onTemplate,
}: {
  catalog: Catalog;
  save: Snapshot | null;
  encounters: Encounter[];
  onSpecies: (id: number) => void;
  onTemplate: (template: Template) => void;
}) {
  const { t } = useI18n();
  const [period, setPeriod] = useState("all");
  const [query, setQuery] = useState("");
  const [savedClock, setSavedClock] =
    useState<AcquisitionReport["clock"]>(null);
  const identity = `${catalog.profile.md5}:${encounters[0]?.map_id ?? ""}`;
  useEffect(() => {
    setPeriod("all");
    setQuery("");
  }, [identity]);
  useEffect(() => {
    let active = true;
    setSavedClock(null);
    if (period === "save" && save && encounters[0])
      api<AcquisitionReport>("acquisition", {
        kind: "species",
        id: encounters[0].species,
        use_save_clock: true,
        hour: null,
      })
        .then((r) => {
          if (active) setSavedClock(r.clock);
        })
        .catch(() => {});
    return () => {
      active = false;
    };
  }, [identity, period, save]);
  const effective = period === "save" ? (savedClock?.period ?? "all") : period;
  const tables = encounterTables(encounters, effective);
  const hasTime =
    !!catalog.profile.clock &&
    encounters.some((e) => e.periods?.some((p) => p !== "base"));
  if (!encounters.length)
    return <p className="small muted">{t("encounterNone")}</p>;
  return (
    <section className="map-encounters">
      <h3>{t("mapPokemonEncounter")}</h3>
      <div className="reference-filters">
        <input
          aria-label={t("encounterSearch")}
          placeholder={t("encounterSearch")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        {hasTime && (
          <label>
            {t("encounterTime")}{" "}
            <select value={period} onChange={(e) => setPeriod(e.target.value)}>
              <option value="all">{t("clockAllPeriods")}</option>
              <option value="save" disabled={!save}>
                {t("clockUseSave")}
              </option>
              {["morning", "day", "dusk", "night"].map((p) => (
                <option key={p} value={p}>
                  {encounterPeriodName(p, t, catalog.profile.clock?.starts)}
                </option>
              ))}
            </select>
          </label>
        )}
      </div>
      {period === "save" && (
        <p className="small muted">
          {savedClock ? clockSummary(savedClock, t) : t("loading")}
          {savedClock &&
            !savedClock.period &&
            ` · ${t("encounterUnknownTime")}`}
        </p>
      )}
      {tables.map((table, i) => {
        const rows = table.rows.filter((e) =>
          speciesDisplayName(catalog, e.species)
            .toLocaleLowerCase()
            .includes(query.toLocaleLowerCase()),
        );
        if (!rows.length) return null;
        return (
          <div key={i} className="encounter-method-table">
            <h4>
              {acquisitionKindName(table.method, catalog, t)}{" "}
              <small className="muted">
                {table.periods
                  .map((p) =>
                    encounterPeriodName(p, t, catalog.profile.clock?.starts),
                  )
                  .join(" / ")}
                {table.selector &&
                  ` · ${t(table.selector.fallback ? "encounterDefault" : "encounterConditionalTable")}`}
              </small>
            </h4>
            <table className="compact-table">
              <thead>
                <tr>
                  <th>{t("species")}</th>
                  <th>{t("level")}</th>
                  <th>{t("encounterProbability")}</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {rows.map((e, j) => {
                  const template = {
                    species: e.species,
                    level: e.min_level,
                    met_location: e.region,
                    egg: e.method === "egg",
                  };
                  return (
                    <tr
                      key={`${e.species}:${j}`}
                      draggable={
                        catalog.profile.capabilities?.save_edit !== false
                      }
                      onDragStart={(event) =>
                        event.dataTransfer.setData(
                          "application/x-gen3",
                          JSON.stringify({
                            kind: "template",
                            template,
                            profile: catalog.profile.md5,
                          }),
                        )
                      }
                    >
                      <td>
                        <button
                          className="link-button encounter-pokemon"
                          onClick={() => onSpecies(e.species)}
                        >
                          <Sprite catalog={catalog} species={e.species} />
                          {speciesDisplayName(catalog, e.species)}
                        </button>
                      </td>
                      <td>
                        {e.min_level === e.max_level
                          ? e.min_level
                          : `${e.min_level}–${e.max_level}`}
                      </td>
                      <td>{e.weight === null ? "—" : `${e.weight}%`}</td>
                      <td>
                        <button
                          className="link-button small"
                          onClick={() => onTemplate(template)}
                          disabled={
                            catalog.profile.capabilities?.save_edit === false
                          }
                        >
                          {t("pickDestination")} ↗
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        );
      })}
      <p className="small muted">{t("encounterProbabilityHelp")}</p>
    </section>
  );
}
