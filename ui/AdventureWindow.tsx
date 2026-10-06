import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "./api";
import { Floating, Sprite } from "./components";
import { ConditionDetails } from "./ConditionDetails";
import { useI18n } from "./i18n";
import { useRomCharacterImage } from "./romCharacterImage";
import type {
  AdventureGuide,
  AdventureTask,
  Catalog,
  MapFocus,
  QueryTarget,
  Snapshot,
  World,
} from "./types";

export function AdventureWindow({
  catalog,
  save,
  world,
  loadWorld,
  onClose,
  onMap,
  onTarget,
}: {
  catalog: Catalog;
  save: Snapshot | null;
  world: World | null;
  loadWorld: () => void;
  onClose: () => void;
  onMap: (id: string, focus?: MapFocus) => void;
  onTarget: (target: QueryTarget) => void;
}) {
  const { t } = useI18n();
  const [report, setReport] = useState<AdventureGuide | null>(null);
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("all");
  const [status, setStatus] = useState("all");
  const [selected, setSelected] = useState("");
  const [history, setHistory] = useState<string[]>([]);
  const [expanded, setExpanded] = useState<string[]>([]);
  const [limit, setLimit] = useState(80);
  const [showUnplaced, setShowUnplaced] = useState(false);
  const [retry, setRetry] = useState(0);
  const [failed, setFailed] = useState(false);
  const choseTask = useRef(false);
  useEffect(loadWorld, [loadWorld]);
  useEffect(() => {
    let active = true;
    setReport(null);
    setFailed(false);
    api<AdventureGuide>("adventure_guide", {
      expected_rom_md5: catalog.profile.md5,
    })
      .then((r) => {
        if (active && r.rom_md5 === catalog.profile.md5) {
          setReport(r);
          setSelected((old) =>
            r.tasks.some((n) => n.id === old)
              ? old
              : (r.tasks.find((n) => n.next_candidate)?.id ??
                r.tasks[0]?.id ??
                ""),
          );
        }
      })
      .catch(() => {
        if (active) setFailed(true);
      });
    return () => {
      active = false;
    };
  }, [catalog.profile.md5, save, retry]);
  const name = (goal: QueryTarget) =>
    (goal.kind === "item"
      ? catalog.items
      : goal.kind === "species"
        ? catalog.species
        : catalog.moves
    ).find((r) => r.id === goal.id)?.name ?? `#${goal.id}`;
  const mapName = (id: string) =>
    world?.maps.find((m) => m.id === id)?.name ?? id;
  const title = (n: AdventureTask) =>
    n.kind === "main"
      ? t("guideStage").replace("{n}", String(n.stage))
      : n.goals.length
        ? `${t("guideReward")} · ${n.goals.map(name).join(" / ")}`
        : `${t("guideClue")} · ${mapName(n.map_id)}`;
  const nodes = useMemo(
    () => new Map(report?.tasks.map((n) => [n.id, n])),
    [report],
  );
  useEffect(() => {
    if (!report || !world || choseTask.current || showUnplaced) return;
    const hidden = new Set(world.reference_visibility?.maps.map((m) => m.id));
    const current = report.tasks.find((n) => n.id === selected);
    if (current?.next_candidate || (current && !hidden.has(current.map_id)))
      return;
    setSelected(
      report.tasks.find((n) => n.next_candidate || !hidden.has(n.map_id))?.id ??
        "",
    );
  }, [report, world, selected, showUnplaced]);
  const filtered = (report?.tasks ?? []).filter(
    (n) =>
      (showUnplaced ||
        n.next_candidate ||
        !world?.reference_visibility?.maps.some((m) => m.id === n.map_id)) &&
      (kind === "all" || n.kind === kind) &&
      (status === "all" || n.status === status) &&
      `${title(n)} ${mapName(n.map_id)} ${n.text.join(" ")}`
        .toLocaleLowerCase()
        .includes(query.toLocaleLowerCase()),
  );
  const task = nodes.get(selected);
  const next = report?.tasks.filter((n) => n.next_candidate) ?? [];
  const choose = (id: string) => {
    choseTask.current = true;
    if (id !== selected) setHistory((h) => [...h.slice(-29), selected]);
    setSelected(id);
  };
  const jump = (n: AdventureTask) =>
    onMap(
      n.map_id,
      n.x != null && n.y != null ? { x: n.x, y: n.y } : undefined,
    );
  const tree = (n: AdventureTask, trail: string[] = []): React.ReactNode => {
    if (trail.includes(n.id) || trail.length >= 4)
      return <p className="small muted">{t("guideDependencyLimit")}</p>;
    return (
      <ul className="quest-dependency-tree">
        {n.prerequisites.map((alternatives, i) => (
          <li key={i}>
            <small className="muted">{t("guideAlternatives")}</small>
            {alternatives.map((id) => {
              const p = nodes.get(id);
              return (
                p && (
                  <details
                    key={id}
                    onToggle={(e) => {
                      if (e.target !== e.currentTarget) return;
                      const key = [...trail, n.id, id].join("/");
                      const open = e.currentTarget.open;
                      setExpanded((old) =>
                        open
                          ? [...new Set([...old, key])]
                          : old.filter((v) => v !== key),
                      );
                    }}
                  >
                    <summary>
                      <button
                        className="link-button"
                        onClick={(e) => {
                          e.preventDefault();
                          choose(id);
                        }}
                      >
                        {title(p)}
                      </button>{" "}
                      · {mapName(p.map_id)}
                    </summary>
                    <p className="small">{p.text[0] || t("guideNoDialogue")}</p>
                    {expanded.includes([...trail, n.id, id].join("/")) &&
                      tree(p, [...trail, n.id])}
                  </details>
                )
              );
            })}
          </li>
        ))}
      </ul>
    );
  };
  return (
    <Floating title={t("guide")} onClose={onClose} initial={-2} wide>
      <details className="reference-help guide-scope">
        <summary>{t("guideReadingHelp")}</summary>
        <p className="small muted">{t("guideScope")}</p>
        <p className="small muted">{t("guideStageHelp")}</p>
      </details>
      {report?.story_supported && (
        <section className="guide-next">
          <strong>
            {report.current_stage == null
              ? t("guideOpenSave")
              : t("guideCurrentStage").replace(
                  "{n}",
                  String(report.current_stage),
                )}
          </strong>
          {report.current_stage != null && (
            <>
              <h4>{t("guideNext")}</h4>
              {next.length ? (
                next.slice(0, 8).map((n) => (
                  <button
                    key={n.id}
                    className="guide-next-action"
                    onClick={() => choose(n.id)}
                  >
                    {title(n)} · {mapName(n.map_id)} ↗
                  </button>
                ))
              ) : (
                <p className="small muted">{t("guideNextUnknown")}</p>
              )}
            </>
          )}
        </section>
      )}
      {report && !report.story_supported && (
        <p className="small muted">{t("guideStoryUnknown")}</p>
      )}
      <div className="reference-filters guide-filters">
        <label>
          <input
            type="checkbox"
            checked={showUnplaced}
            onChange={(e) => setShowUnplaced(e.target.checked)}
          />
          {t("guideShowUnconfirmedEntrances")}
        </label>
        <input
          aria-label={t("guideSearch")}
          placeholder={t("guideSearch")}
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setLimit(80);
          }}
        />
        <label>
          {t("guideSection")}{" "}
          <select
            value={kind}
            onChange={(e) => {
              setKind(e.target.value);
              setLimit(80);
            }}
          >
            <option value="all">{t("filterAll")}</option>
            {["main", "side", "prerequisite"].map((k) => (
              <option key={k} value={k}>
                {t(`guideKind_${k}`)}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("guideStatus")}{" "}
          <select
            value={status}
            onChange={(e) => {
              setStatus(e.target.value);
              setLimit(80);
            }}
          >
            <option value="all">{t("filterAll")}</option>
            {["completed", "ready", "blocked", "unknown"].map((k) => (
              <option key={k} value={k}>
                {t(`guideStatus_${k}`)}
              </option>
            ))}
          </select>
        </label>
      </div>
      {!report ? (
        <p role="status">
          {t(failed ? "guideLoadError" : "loading")}
          {failed && (
            <button onClick={() => setRetry((n) => n + 1)}>
              {t("guideRetry")}
            </button>
          )}
        </p>
      ) : (
        <div className="reference-layout adventure-layout">
          <div className="reference-list">
            <div className="reference-rows">
              {filtered.slice(0, limit).map((n) => (
                <button
                  key={n.id}
                  className={n.id === selected ? "selected" : ""}
                  onClick={() => choose(n.id)}
                >
                  <span>
                    <strong>{title(n)}</strong>
                    <small className="guide-task-meta">
                      {mapName(n.map_id)} · {t(`guideStatus_${n.status}`)}
                    </small>
                    {n.text[0] && (
                      <small className="guide-task-excerpt">
                        {n.text[0].slice(0, 60)}
                      </small>
                    )}
                  </span>
                </button>
              ))}
            </div>
            {!filtered.length && <p>{t("noResults")}</p>}
            {filtered.length > limit && (
              <button onClick={() => setLimit((n) => n + 80)}>
                {t("acqMore")} ({filtered.length - limit})
              </button>
            )}
          </div>
          <div className="reference-detail">
            <button
              className="link-button"
              disabled={!history.length}
              onClick={() => {
                setSelected(history.at(-1)!);
                setHistory((h) => h.slice(0, -1));
              }}
            >
              ← {t("guideBack")}
            </button>
            {task && (
              <>
                <h2>{title(task)}</h2>
                <p className={`source-status status-${task.status}`}>
                  {t(`guideStatus_${task.status}`)}
                </p>
                <p className="small muted">
                  {t(
                    task.status === "completed"
                      ? "guideReceipt"
                      : "guideStatusHelp",
                  )}
                </p>
                <button className="link-button" onClick={() => jump(task)}>
                  {mapName(task.map_id)}
                  {task.x != null ? ` · (${task.x}, ${task.y})` : ""} ↗
                </button>
                <TaskMap
                  task={task}
                  catalog={catalog}
                  world={world}
                  onClick={() => jump(task)}
                />
                <div className="source-related">
                  {task.goals.map((g) => (
                    <button
                      className="link-button"
                      key={`${g.kind}:${g.id}`}
                      onClick={() => onTarget(g)}
                    >
                      {g.kind === "species" && (
                        <Sprite catalog={catalog} species={g.id} />
                      )}
                      {name(g)} ↗
                    </button>
                  ))}
                </div>
                <h3>{t("guideDialogue")}</h3>
                <p className="small muted">{t("guideDialogueHelp")}</p>
                {task.text.map((text, i) => (
                  <blockquote key={i}>{text}</blockquote>
                ))}
                {!task.text.length && (
                  <p className="muted">{t("guideNoDialogue")}</p>
                )}
                <h3>{t("guideDependencies")}</h3>
                <p className="small muted">{t("guideDependencyHelp")}</p>
                {tree(task)}
                {!task.prerequisites.length && (
                  <p className="muted">{t("guideNoDependency")}</p>
                )}
                <ConditionDetails
                  checks={task.checks}
                  catalog={catalog}
                  onMap={onMap}
                  onTarget={onTarget}
                />
              </>
            )}
          </div>
        </div>
      )}
    </Floating>
  );
}
function TaskMap({
  task,
  catalog,
  world,
  onClick,
}: {
  task: AdventureTask;
  catalog: Catalog;
  world: World | null;
  onClick: () => void;
}) {
  const { t } = useI18n();
  const [url, setUrl] = useState("");
  const map = world?.maps.find((m) => m.id === task.map_id);
  const actor = useRomCharacterImage(
    catalog.profile.md5,
    "object_sprite",
    task.actor,
  );
  useEffect(() => {
    let active = true;
    setUrl("");
    api<{ url: string }>("map_image", { id: task.map_id })
      .then((r) => {
        if (active) setUrl(r.url);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [catalog.profile.md5, task.map_id]);
  return url && map ? (
    <button
      className="guide-map"
      onClick={onClick}
      aria-label={t("guideOpenMap")}
    >
      <img src={url} alt={map.name} />
      {task.x != null &&
        task.y != null &&
        task.x >= 0 &&
        task.y >= 0 &&
        task.x < map.width &&
        task.y < map.height && (
          <span
            className="guide-map-pin"
            style={{
              left: `${((task.x + 0.5) / map.width) * 100}%`,
              top: `${((task.y + 0.5) / map.height) * 100}%`,
            }}
          >
            {actor ? <img src={actor.url} alt={t("mapNpcs")} /> : "📍"}
          </span>
        )}
    </button>
  ) : null;
}
