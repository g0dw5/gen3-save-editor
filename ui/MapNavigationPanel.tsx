import { useI18n } from "./i18n";
import type {
  Catalog,
  GameMap,
  MapFocus,
  MapLink,
  MapNavigation,
  QueryTarget,
} from "./types";
import { ConditionDetails } from "./ConditionDetails";

export function MapNavigationPanel({
  report,
  maps,
  onMap,
  catalog,
  onTarget,
}: {
  report: MapNavigation | null;
  maps: GameMap[];
  onMap: (id: string, focus?: MapFocus) => void;
  catalog: Catalog;
  onTarget: (target: QueryTarget) => void;
}) {
  const { t } = useI18n();
  const name = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  const link = (edge: MapLink, incoming: boolean, index: number) => {
    const id = incoming ? edge.from : edge.to;
    const x = incoming ? edge.x : edge.target_x;
    const y = incoming ? edge.y : edge.target_y;
    return (
      <div className="reference-line" key={`${edge.offset}-${index}`}>
        {id ? (
          <button
            className="link-button"
            onClick={() =>
              onMap(id, x !== null && y !== null ? { x, y } : undefined)
            }
          >
            {name(id)} ↗
          </button>
        ) : (
          <span>{t("navDynamic")}</span>
        )}
        <small>
          {t(
            edge.kind === "script_warp"
              ? "navScriptWarp"
              : edge.kind === "warp"
                ? "navWarp"
                : "navConnection",
          )}
          {edge.kind !== "connection"
            ? edge.x !== null && edge.y !== null
              ? ` · (${edge.x}, ${edge.y})`
              : ` · ${t("navUnplaced")}`
            : ` · ${t(({ 1: "navSouth", 2: "navNorth", 3: "navWest", 4: "navEast", 5: "navDive", 6: "navEmerge" } as const)[edge.direction as 1] ?? "unresolved")}`}
        </small>
        {edge.unresolved && (
          <span className="small warning-text">{t("navUnresolved")}</span>
        )}
        {edge.script && (
          <details className="nav-script-details">
            <summary>{t("navScriptConditions")}</summary>
            <p className="small muted">
              {t(
                edge.script.source_kind === "npc"
                  ? "navTalkPassage"
                  : "navScriptHelp",
              )}
            </p>
            <ConditionDetails
              checks={edge.script.checks}
              catalog={catalog}
              onTarget={onTarget}
              onMap={onMap}
            />
            <p className="small warning-text">{t("navScriptAccessUnknown")}</p>
          </details>
        )}
      </div>
    );
  };
  return (
    <section className="map-navigation">
      <h3>{t("navTitle")}</h3>
      <details className="reference-help">
        <summary>{t("referenceSourceHelp")}</summary>
        <p className="small muted">{t("navHelp")}</p>
      </details>
      {!report ? (
        <p>{t("loading")}</p>
      ) : (
        <>
          <h4>{t("navApproaches")}</h4>
          {report.approaches.length ? (
            report.approaches.map((path, i) => (
              <div key={i} className="nav-path">
                {path.map((e, j) => (
                  <span key={j}>
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
                      {name(e.from)}
                      {e.x !== null ? ` (${e.x}, ${e.y})` : ""}
                    </button>{" "}
                    {e.script && (
                      <span className="small warning-text">
                        {t("navConditionalPassage")}{" "}
                      </span>
                    )}
                    →{" "}
                  </span>
                ))}
                <span>{name(report.map_id)}</span>
              </div>
            ))
          ) : (
            <p className="small muted">{t("navNoApproach")}</p>
          )}
          {report.truncated && <p className="small muted">{t("navBounded")}</p>}
          <details>
            <summary>
              {t("navIncoming")} ({report.incoming.length})
            </summary>
            {report.incoming.map((e, i) => link(e, true, i))}
          </details>
          <details>
            <summary>
              {t("navOutgoing")} ({report.outgoing.length})
            </summary>
            {report.outgoing.map((e, i) => link(e, false, i))}
          </details>
          <details>
            <summary>{t("evidence")}</summary>
            <pre>{JSON.stringify(report, null, 2)}</pre>
          </details>
        </>
      )}
    </section>
  );
}
