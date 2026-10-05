import { useContext, useEffect, useRef, useState } from "react";
import { Search } from "lucide-react";
import { api } from "./api";
import {
  ConditionDetails,
  ConditionQueryRevision,
  effectLabel,
} from "./ConditionDetails";
import { useI18n } from "./i18n";
import type {
  Catalog,
  EventClueReport,
  GameMap,
  MapFocus,
  QueryTarget,
} from "./types";

export function EventCluesPanel({
  catalog,
  maps,
  search,
  onSearch,
  selected,
  onSelect,
  mapId,
  onMapFilter,
  onMap,
  onTarget,
  offset,
  setOffset,
}: {
  catalog: Catalog;
  maps: GameMap[];
  search: string;
  onSearch: (value: string) => void;
  selected: string;
  onSelect: (id: string) => void;
  mapId: string;
  onMapFilter: (id: string) => void;
  onMap: (id: string, focus?: MapFocus) => void;
  onTarget: (target: QueryTarget) => void;
  offset: number;
  setOffset: (offset: number) => void;
}) {
  const { t } = useI18n();
  const revision = useContext(ConditionQueryRevision);
  const [refresh, setRefresh] = useState(0);
  const [report, setReport] = useState<EventClueReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const generation = useRef(0);
  const md5 = catalog.profile.md5;
  useEffect(() => {
    const ticket = ++generation.current;
    setReport(null);
    setError(null);
    setBusy(true);
    const timer = setTimeout(() => {
      api<EventClueReport>("event_search", {
        expected_rom_md5: md5,
        search,
        map_id: mapId || null,
        offset,
        selected_id: selected || null,
      })
        .then((value) => {
          if (generation.current === ticket && value.rom_md5 === md5)
            setReport(value);
        })
        .catch((e) => {
          if (generation.current === ticket) setError(e);
        })
        .finally(() => {
          if (generation.current === ticket) setBusy(false);
        });
    }, 160);
    return () => {
      clearTimeout(timer);
      generation.current++;
    };
  }, [md5, search, mapId, offset, selected, revision, refresh]);
  const current =
    report?.entries.find((entry) => entry.id === selected) ??
    (report?.selected?.id === selected ? report.selected : null) ??
    report?.entries[0];
  return (
    <div className="event-clues-panel">
      <p className="muted">{t("eventCluesHelp")}</p>
      <div className="reference-layout">
        <div className="reference-list">
          <label className="search-field">
            <Search size={16} />
            <input
              aria-label={t("eventCluesSearch")}
              placeholder={t("eventCluesSearch")}
              value={search}
              maxLength={128}
              onChange={(e) => {
                onSelect("");
                setOffset(0);
                onSearch(e.target.value);
              }}
            />
          </label>
          <label className="event-map-filter">
            {t("maps")}
            <select
              aria-label={t("eventCluesMapFilter")}
              value={mapId}
              onChange={(e) => {
                onSelect("");
                setOffset(0);
                onMapFilter(e.target.value);
              }}
            >
              <option value="">{t("trainerFilterAll")}</option>
              {maps.map((map) => (
                <option key={map.id} value={map.id}>
                  {map.name} · {map.id}
                </option>
              ))}
            </select>
          </label>
          <button
            className="link-button"
            onClick={() => setRefresh((v) => v + 1)}
          >
            {t("dependencyRefresh")}
          </button>
          {busy && <p role="status">{t("loading")}</p>}
          {!!error && (
            <details open>
              <summary>{t("eventCluesError")}</summary>
              <pre>{JSON.stringify(error, null, 2)}</pre>
            </details>
          )}
          {report && (
            <>
              <p className="small">
                {t("eventCluesMatches")} {report.total_matches}
              </p>
              <div className="reference-rows">
                {report.entries.map((entry) => (
                  <button
                    key={entry.id}
                    type="button"
                    className={current?.id === entry.id ? "selected" : ""}
                    onClick={() => onSelect(entry.id)}
                  >
                    <span>
                      {entry.text[0]?.text.slice(0, 90) ||
                        t(`dependency_${entry.reference.kind}`)}
                      <small className="event-clue-context">
                        {entry.reference.map_name} ·{" "}
                        {t(`dependency_${entry.reference.kind}`)}
                      </small>
                    </span>
                  </button>
                ))}
              </div>
              {!report.entries.length && (
                <p className="muted">{t("eventCluesNone")}</p>
              )}
              <div className="event-clue-paging">
                <button
                  disabled={busy || offset === 0}
                  onClick={() => {
                    onSelect("");
                    setOffset(Math.max(0, offset - 32));
                  }}
                >
                  {t("eventCluesPrevious")}
                </button>
                <button
                  disabled={busy || report.next_offset == null}
                  onClick={() => {
                    onSelect("");
                    setOffset(report.next_offset!);
                  }}
                >
                  {t("eventCluesNext")}
                </button>
              </div>
            </>
          )}
        </div>
        <div className="reference-detail">
          {current && (
            <>
              <span className="eyebrow">
                {t("readOnly")} · {catalog.profile.label}
              </span>
              <h2>{current.reference.map_name}</h2>
              <p>
                {t(`dependency_${current.reference.kind}`)} ·{" "}
                {t("eventCluesUnknownTask")}
              </p>
              <button
                className="link-button"
                onClick={() =>
                  onMap(
                    current.reference.map_id,
                    current.reference.x != null && current.reference.y != null
                      ? {
                          x: current.reference.x,
                          y: current.reference.y,
                        }
                      : undefined,
                  )
                }
              >
                {current.reference.map_name}
                {current.reference.x != null && current.reference.y != null
                  ? ` (${current.reference.x}, ${current.reference.y})`
                  : ""}{" "}
                ↗
              </button>
              <p className="small muted">{t("dependencyAccessUnknown")}</p>
              {!!current.text.length && (
                <section className="event-clue-text">
                  <h3>{t("dependencyText")}</h3>
                  <p className="small muted">{t("dependencyTextHelp")}</p>
                  {current.text.map((text) => (
                    <blockquote key={text.offset}>{text.text}</blockquote>
                  ))}
                </section>
              )}
              {!!current.visibility.length && (
                <ConditionDetails
                  checks={current.visibility}
                  heading={t("eventCluesVisibility")}
                  catalog={catalog}
                  onTarget={onTarget}
                  onMap={onMap}
                />
              )}
              <h3>{t("eventCluesChanges")}</h3>
              <p className="small muted">{t("eventCluesObservedHelp")}</p>
              {!current.effects.length && (
                <p className="muted">{t("eventCluesNoWrites")}</p>
              )}
              {current.effects.map((effect, i) => (
                <article className="event-clue-effect" key={i}>
                  <strong>{effectLabel(effect.effect, t)}</strong>
                  <p>
                    {t(
                      effect.observed === true
                        ? "eventCluesObservedYes"
                        : effect.observed === false
                          ? "eventCluesObservedNo"
                          : "acqStatus_unknown",
                    )}
                  </p>
                  <ConditionDetails
                    checks={effect.conditions}
                    catalog={catalog}
                    onTarget={onTarget}
                    onMap={onMap}
                  />
                  <details>
                    <summary>{t("evidence")}</summary>
                    <pre>{JSON.stringify(effect.effect, null, 2)}</pre>
                  </details>
                </article>
              ))}
              {current.effects_truncated && (
                <p className="muted">{t("eventCluesEffectsLimit")}</p>
              )}
              <details>
                <summary>{t("evidence")}</summary>
                <pre>
                  {JSON.stringify(
                    {
                      reference: current.reference,
                      path_complete: current.path_complete,
                      stopped_at: current.stopped_at,
                      coverage: report?.coverage,
                    },
                    null,
                    2,
                  )}
                </pre>
              </details>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
