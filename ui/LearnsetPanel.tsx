import { useEffect, useMemo, useState } from "react";
import { moveCategoryNames, useI18n } from "./i18n";
import { Types } from "./components";
import type { Catalog, SpeciesDetail } from "./types";

export function LearnsetPanel({
  detail,
  catalog,
  onMove,
}: {
  detail: SpeciesDetail;
  catalog: Catalog;
  onMove: (id: number) => void;
}) {
  const { t, locale } = useI18n();
  const [source, setSource] = useState("all");
  const [category, setCategory] = useState("all");
  const [type, setType] = useState("all");
  const [query, setQuery] = useState("");
  const [limit, setLimit] = useState(24);
  useEffect(() => {
    setSource("all");
    setCategory("all");
    setType("all");
    setQuery("");
  }, [detail.species.id, catalog.profile.md5]);
  useEffect(
    () => setLimit(24),
    [source, category, type, query, detail.species.id],
  );
  const moves = useMemo(
    () => new Map(catalog.moves.map((m) => [m.id, m])),
    [catalog],
  );
  const label = (value: string) =>
    t(
      value === "level" ? "levelSource" : value === "egg" ? "eggSource" : value,
    );
  const rows = detail.learnset.filter((s) => {
    const m = moves.get(s.move_id);
    return (
      (source === "all" || s.source === source) &&
      (category === "all" || m?.category === Number(category)) &&
      (type === "all" || m?.move_type === Number(type)) &&
      `${m?.name ?? s.move_id}`
        .toLocaleLowerCase()
        .includes(query.toLocaleLowerCase())
    );
  });
  return (
    <details className="reference-section" key={detail.species.id}>
      <summary>
        {t("learnset")} · {detail.learnset.length}
      </summary>
      <div className="reference-filters">
        <input
          aria-label={t("learnsetSearch")}
          placeholder={t("learnsetSearch")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <label>
          {t("source")}{" "}
          <select
            aria-label={t("source")}
            value={source}
            onChange={(e) => setSource(e.target.value)}
          >
            <option value="all">{t("filterAll")}</option>
            {[...new Set(detail.learnset.map((s) => s.source))].map((s) => (
              <option key={s} value={s}>
                {label(s)}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("moveCategory")}{" "}
          <select
            aria-label={t("moveCategory")}
            value={category}
            onChange={(e) => setCategory(e.target.value)}
          >
            <option value="all">{t("filterAll")}</option>
            {moveCategoryNames[locale].map((s, i) => (
              <option key={i} value={i}>
                {s}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("type")}{" "}
          <select
            aria-label={t("type")}
            value={type}
            onChange={(e) => setType(e.target.value)}
          >
            <option value="all">{t("filterAll")}</option>
            {catalog.type_names.map((name, i) => (
              <option key={i} value={i}>
                {name}
              </option>
            ))}
          </select>
        </label>
      </div>
      {catalog.profile.capabilities?.complete_learnsets === false && (
        <p className="small muted">{t("partialLearnsetHelp")}</p>
      )}
      {detail.teaching_list_present === false && (
        <p className="small muted">{t("missingTeachingList")}</p>
      )}
      <div className="learnset-table">
        <table className="compact-table">
          <thead>
            <tr>
              <th>{t("move")}</th>
              <th>{t("moveCategory")}</th>
              <th>{t("type")}</th>
              <th>{t("power")}</th>
              <th>{t("source")}</th>
              <th>{t("level")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.slice(0, limit).map((s, i) => {
              const m = moves.get(s.move_id);
              return (
                <tr key={`${s.offset}:${s.species}:${i}`}>
                  <td>
                    <button
                      className="link-button"
                      onClick={() => onMove(s.move_id)}
                    >
                      {m?.name ?? s.move_id}
                    </button>
                  </td>
                  <td>{m && moveCategoryNames[locale][m.category]}</td>
                  <td>
                    {m && <Types catalog={catalog} values={[m.move_type]} />}
                  </td>
                  <td>{m?.power || "—"}</td>
                  <td>
                    {label(s.source)}
                    {s.species !== detail.species.id && (
                      <small>
                        {" "}
                        ·{" "}
                        {catalog.species.find((p) => p.id === s.species)?.name}
                      </small>
                    )}
                  </td>
                  <td>{s.level ?? "—"}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      {!rows.length && <p className="muted">{t("noResults")}</p>}
      {rows.length > limit && (
        <button onClick={() => setLimit((n) => n + 24)}>
          {t("acqMore")} ({rows.length - limit})
        </button>
      )}
    </details>
  );
}
