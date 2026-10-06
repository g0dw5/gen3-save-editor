import { useEffect, useState } from "react";
import { EvolutionRuleDetails } from "./EvolutionRuleDetails";
import { speciesDisplayName } from "./speciesDisplay";
import { useI18n } from "./i18n";
import type { AcquisitionReport, Catalog, GameMap, QueryTarget } from "./types";

export function EvolutionUsesPanel({
  rows,
  catalog,
  maps,
  onTarget,
  onMap,
}: {
  rows: NonNullable<AcquisitionReport["evolution_uses"]>;
  catalog: Catalog;
  maps: GameMap[];
  onTarget: (target: QueryTarget) => void;
  onMap: (id: string) => void;
}) {
  const { t } = useI18n();
  const [limit, setLimit] = useState(12);
  useEffect(() => setLimit(12), [rows, catalog.profile.md5]);
  if (!rows.length) return null;
  return (
    <details className="evolution-uses">
      <summary>
        {t("evolutionUses")} · {rows.length}
      </summary>
      <p className="small muted">{t("evolutionUsesHelp")}</p>
      {rows.slice(0, limit).map((row, index) => (
        <article
          className="encounter-card"
          key={`${row.source}:${row.evolution.offset}:${index}`}
        >
          <div className="source-related">
            {[row.source, row.evolution.target].map((id, position) => (
              <span key={position}>
                {position === 1 && " → "}
                <button
                  className="link-button"
                  onClick={() => onTarget({ kind: "species", id })}
                >
                  {speciesDisplayName(catalog, id, t)} ↗
                </button>
              </span>
            ))}
          </div>
          <EvolutionRuleDetails
            rule={row.evolution}
            catalog={catalog}
            maps={maps}
            related={row.related}
            onTarget={onTarget}
            onMap={onMap}
          />
          <details>
            <summary>{t("evidence")}</summary>
            <pre>{JSON.stringify(row, null, 2)}</pre>
          </details>
        </article>
      ))}
      {rows.length > limit && (
        <button onClick={() => setLimit((n) => n + 12)}>
          {t("acqMore")} ({rows.length - limit})
        </button>
      )}
    </details>
  );
}
