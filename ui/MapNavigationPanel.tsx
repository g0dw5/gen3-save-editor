import { useI18n } from "./i18n";
import type { GameMap, MapFocus, MapLink, MapNavigation } from "./types";

export function MapNavigationPanel({
  report,
  maps,
  onMap,
}: {
  report: MapNavigation | null;
  maps: GameMap[];
  onMap: (id: string, focus?: MapFocus) => void;
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
          {t(edge.kind === "warp" ? "navWarp" : "navConnection")}
          {edge.kind === "warp"
            ? ` · (${edge.x}, ${edge.y})`
            : ` · ${t(({ 1: "navSouth", 2: "navNorth", 3: "navWest", 4: "navEast", 5: "navDive", 6: "navEmerge" } as const)[edge.direction as 1] ?? "unresolved")}`}
        </small>
        {edge.unresolved && (
          <span className="small warning-text">{t("navUnresolved")}</span>
        )}
      </div>
    );
  };
  return (
    <section className="map-navigation">
      <h3>{t("navTitle")}</h3>
      <p className="small muted">{t("navHelp")}</p>
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
          <details open>
            <summary>
              {t("navIncoming")} ({report.incoming.length})
            </summary>
            {report.incoming.map((e, i) => link(e, true, i))}
          </details>
          <details open>
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
