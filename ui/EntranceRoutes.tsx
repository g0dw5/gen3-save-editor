import { useState } from "react";
import { useI18n } from "./i18n";
import { ConditionDetails } from "./ConditionDetails";
import type {
  Catalog,
  EntranceSuggestion,
  GameMap,
  MapFocus,
  MapLink,
  QueryTarget,
} from "./types";

export const entranceChecks = (path: MapLink[]) =>
  path.flatMap((e) => e.script?.checks ?? []);
export function entranceState(path: MapLink[]): string | null {
  if (!path.some((e) => e.script)) return null;
  const checks = entranceChecks(path);
  return checks.some((c) => c.satisfied === false)
    ? "planEntranceBlocked"
    : checks.some((c) => c.satisfied == null) || !checks.length
      ? "planEntranceUnknown"
      : "planEntranceMet";
}

/** Approach alternatives remain separate and do not establish live accessibility. */
export function EntranceRoutes({
  entry,
  maps,
  catalog,
  onMap,
  onTarget,
}: {
  entry?: EntranceSuggestion;
  maps: GameMap[];
  catalog: Catalog;
  onMap: (id: string, focus?: MapFocus) => void;
  onTarget: (target: QueryTarget) => void;
}) {
  const { t } = useI18n();
  const [limit, setLimit] = useState(3);
  const name = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  if (!entry) return null;
  const unresolved = entry.unresolved_incoming ?? [];
  const showMap = (id: string, x?: number | null, y?: number | null) => (
    <button
      className="link-button"
      onClick={() => onMap(id, x != null && y != null ? { x, y } : undefined)}
    >
      {name(id)}
      {x != null && y != null ? ` (${x}, ${y})` : ""} ↗
    </button>
  );
  const passages = (path: MapLink[]) =>
    path
      .filter((e) => e.script)
      .map((e, i) => (
        <div className="entrance-passage" key={`${e.offset}-${i}`}>
          <p className="small muted">
            {name(e.from)} · {t("navScriptWarp")} ·{" "}
            {t(
              e.script!.source_kind === "npc"
                ? "navTalkPassage"
                : "navScriptHelp",
            )}
          </p>
          <ConditionDetails
            checks={e.script!.checks}
            catalog={catalog}
            onTarget={onTarget}
            onMap={onMap}
          />
          <p className="small warning-text">{t("navScriptAccessUnknown")}</p>
          <details>
            <summary>{t("evidence")}</summary>
            <pre>{JSON.stringify(e, null, 2)}</pre>
          </details>
        </div>
      ));
  return (
    <section className="collection-entrances small">
      <p>{t("navApproaches")}</p>
      <p className="muted">{t("planEntranceAlternatives")}</p>
      {!entry.chains.length && <p className="muted">{t("navNoApproach")}</p>}
      {entry.chains.slice(0, limit).map((path, i) => {
        const state = entranceState(path);
        const last = path.at(-1);
        return (
          <details className="entrance-path" key={i} open>
            <summary>
              {t("planEntranceNumber").replace("{n}", String(i + 1))}
              {state ? ` · ${t(state)}` : ""}
            </summary>
            <p>
              {path.map((e, j) => (
                <span key={j}>{showMap(e.from, e.x, e.y)} → </span>
              ))}
              {showMap(entry.map_id, last?.target_x, last?.target_y)}
            </p>
            {passages(path)}
          </details>
        );
      })}
      {entry.chains.length > limit && (
        <button onClick={() => setLimit((n) => n + 3)}>
          {t("planEntranceMore")}
        </button>
      )}
      {!!unresolved.length && (
        <details className="entrance-unresolved">
          <summary>
            {t("navUnresolved")} ({unresolved.length})
          </summary>
          {unresolved.map((e, i) => (
            <div key={i}>
              <p>
                {showMap(e.from, e.x, e.y)} · {t("navUnresolved")}
              </p>
              {passages([e])}
            </div>
          ))}
        </details>
      )}
      {entry.truncated && <p className="muted">{t("navBounded")}</p>}
      <p className="muted">{t("navHelp")}</p>
    </section>
  );
}
