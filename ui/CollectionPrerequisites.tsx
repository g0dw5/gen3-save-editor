import { EntranceRoutes } from "./EntranceRoutes";
import { useEffect, useRef, useState } from "react";
import {
  ConditionDetails,
  conditionLabel,
  effectLabel,
} from "./ConditionDetails";
import { acquisitionTargetName } from "./acquisitionLabels";
import { useI18n } from "./i18n";
import type {
  Catalog,
  CollectionPlan,
  GameMap,
  MapFocus,
  QueryTarget,
} from "./types";

type Bundle = NonNullable<CollectionPlan["prerequisites"]>;
type Route = NonNullable<Bundle["routes"]>[number];
type Props = {
  plan: CollectionPlan;
  catalog: Catalog;
  maps: GameMap[];
  onTarget: (target: QueryTarget) => void;
  onMap: (id: string, focus?: MapFocus) => void;
};

/** Alternatives stay separate. Parsed guards and static entrances never prove access. */
export function CollectionPrerequisites(props: Props) {
  const { t } = useI18n();
  const [selected, setSelected] = useState<{
    index: number;
    revision: number;
  } | null>(null);
  const bundle = props.plan.prerequisites;
  if (!bundle) return null;
  const routes = bundle.routes ?? [];
  return (
    <details className="collection-prerequisites" open>
      <summary>{t("planPrerequisites")}</summary>
      <p>{t("planPrerequisiteHelp")}</p>
      {(bundle.truncated || bundle.skipped_conditions > 0) && (
        <p>{t("dependencyAppendixLimit")}</p>
      )}
      {!routes.length && <p>{t("planPrerequisiteNone")}</p>}
      {routes.map((route) => (
        <PrerequisiteRoute
          {...props}
          key={route.report_index}
          bundle={bundle}
          route={route}
          selected={selected}
          onCondition={(index) =>
            setSelected((previous) => ({
              index,
              revision: (previous?.revision ?? 0) + 1,
            }))
          }
        />
      ))}
    </details>
  );
}

function PrerequisiteRoute({
  plan,
  catalog,
  maps,
  onTarget,
  onMap,
  bundle,
  route,
  selected,
  onCondition,
}: Props & {
  bundle: Bundle;
  route: Route;
  selected: { index: number; revision: number } | null;
  onCondition: (index: number) => void;
}) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [limit, setLimit] = useState(8);
  const [goalLimit, setGoalLimit] = useState(8);
  const element = useRef<HTMLDetailsElement>(null);
  useEffect(() => {
    if (selected?.index === route.report_index) {
      setOpen(true);
      element.current?.scrollIntoView({ block: "nearest" });
    }
  }, [selected, route.report_index]);
  const report = bundle.reports[route.report_index];
  if (!report) return null;
  const regions = new Map<number, typeof route.candidates>();
  for (const candidate of route.candidates.slice(0, limit)) {
    const writer = report.writers[candidate.writer_index];
    if (!writer) continue;
    const list = regions.get(writer.reference.region) ?? [];
    list.push(candidate);
    regions.set(writer.reference.region, list);
  }
  return (
    <details
      ref={element}
      className="prerequisite-route encounter-card"
      open={open}
      onToggle={(event) => {
        if (event.target === event.currentTarget)
          setOpen(event.currentTarget.open);
      }}
    >
      <summary>
        {t("planPrerequisiteNumber").replace(
          "{n}",
          String(route.report_index + 1),
        )}{" "}
        · {conditionLabel(report.condition, catalog, t)} ·{" "}
        {t(
          report.condition.satisfied === true
            ? "planConditionYes"
            : report.condition.satisfied === false
              ? "planConditionNo"
              : "acqStatus_unknown",
        )}
      </summary>
      {open && (
        <>
          <p className="small">
            {t("planAffectedGoals")}:{" "}
            {route.goals.slice(0, goalLimit).map(([region, task]) => {
              const goal = plan.regions[region]?.tasks[task];
              return (
                goal && (
                  <button
                    className="link-button"
                    key={`${region}-${task}`}
                    onClick={() => onTarget(goal.target)}
                  >
                    {acquisitionTargetName(goal.target, catalog)} ↗
                  </button>
                )
              );
            })}
          </p>
          {route.goals.length > goalLimit && (
            <button onClick={() => setGoalLimit((n) => n + 8)}>
              {t("planMoreGoals")}
            </button>
          )}
          {!route.candidates.length && <p>{t("dependencyNone")}</p>}
          {report.condition.satisfied !== true &&
            [...regions].map(([region, candidates]) => (
              <section key={region}>
                <h4>
                  {catalog.met_locations.find((r) => r.id === region)?.name ??
                    `#${region}`}
                </h4>
                {candidates.map((candidate) => {
                  const w = report.writers[candidate.writer_index];
                  const entrance = bundle.entrances.find(
                    (e) => e.map_id === w.reference.map_id,
                  );
                  const state = w.conditions.some((g) => g.satisfied === false)
                    ? "blocked"
                    : w.conditions.some((g) => g.satisfied == null)
                      ? "unknown"
                      : "available";
                  return (
                    <div
                      className="prerequisite-candidate"
                      key={candidate.writer_index}
                    >
                      <strong>
                        {t("planCandidateEvent")} · {effectLabel(w.effect, t)}
                      </strong>
                      <p>
                        {t(`acqStatus_${state}`)} ·{" "}
                        {t("dependencyAccessUnknown")}
                      </p>
                      <button
                        className="link-button"
                        onClick={() =>
                          onMap(
                            w.reference.map_id,
                            w.reference.x != null && w.reference.y != null
                              ? { x: w.reference.x, y: w.reference.y }
                              : undefined,
                          )
                        }
                      >
                        {w.reference.map_name}
                        {w.reference.x != null
                          ? ` (${w.reference.x}, ${w.reference.y})`
                          : ""}{" "}
                        ↗
                      </button>
                      {!!w.text.length && (
                        <details>
                          <summary>{t("dependencyText")}</summary>
                          <p className="small muted">
                            {t("dependencyTextHelp")}
                          </p>
                          {w.text.map((text) => (
                            <blockquote key={text.offset}>
                              {text.text}
                            </blockquote>
                          ))}
                        </details>
                      )}
                      {!!candidate.requires.length && (
                        <p className="small">
                          {t("planCandidateRequires")}:{" "}
                          {candidate.requires.map((i) => (
                            <button
                              className="link-button"
                              key={i}
                              onClick={() => onCondition(i)}
                            >
                              {t("planPrerequisiteNumber").replace(
                                "{n}",
                                String(i + 1),
                              )}{" "}
                              ·{" "}
                              {conditionLabel(
                                bundle.reports[i].condition,
                                catalog,
                                t,
                              )}{" "}
                              ↗
                            </button>
                          ))}
                        </p>
                      )}
                      {candidate.recursive && <p>{t("planRecursiveClue")}</p>}
                      <ConditionDetails
                        checks={w.conditions}
                        catalog={catalog}
                        onTarget={onTarget}
                        onMap={onMap}
                      />
                      {(candidate.untraced_conditions.length > 0 ||
                        (candidate.entry_untraced_conditions?.length ?? 0) >
                          0) && (
                        <p className="small muted">{t("planUntracedGuards")}</p>
                      )}
                      <EntranceRoutes
                        entry={entrance}
                        maps={maps}
                        catalog={catalog}
                        onMap={onMap}
                        onTarget={onTarget}
                      />
                      {!w.path_complete && <p>{t("dependencyPathPartial")}</p>}
                      <details>
                        <summary>{t("evidence")}</summary>
                        <pre>
                          {JSON.stringify({ candidate, writer: w }, null, 2)}
                        </pre>
                      </details>
                    </div>
                  );
                })}
              </section>
            ))}
          {report.condition.satisfied !== true &&
            route.candidates.length > limit && (
              <button onClick={() => setLimit((n) => n + 8)}>
                {t("dependencyMore")}
              </button>
            )}
        </>
      )}
    </details>
  );
}
