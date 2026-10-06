import type {
  AcquisitionSource,
  Catalog,
  ItemReward,
  QueryTarget,
  EventDependencyReport,
  MapFocus,
} from "./types";
import { useI18n } from "./i18n";
import { createContext, useContext, useEffect, useRef, useState } from "react";
import { api } from "./api";

export const ConditionQueryRevision = createContext(0);

type Check = AcquisitionSource["conditions"][number];
export const conditionKey = (c: Check["condition"]) =>
  `${c.kind}:${c.id}:${c.comparison}:${c.value}:${c.taken}`;

export function effectLabel(
  effect: EventDependencyReport["writers"][number]["effect"],
  t: (key: string) => string,
) {
  return t(
    effect.kind === "flag"
      ? effect.value === 1
        ? "dependencySetEvent"
        : "dependencyClearEvent"
      : effect.value == null
        ? "dependencyUnknownStage"
        : "dependencyChangeStage",
  );
}

function Dependencies({
  check,
  catalog,
  onTarget,
  onMap,
  depth,
  trail,
}: {
  check: Check;
  catalog: Catalog;
  onTarget?: (t: QueryTarget) => void;
  onMap?: (id: string, focus?: MapFocus) => void;
  depth: number;
  trail: string[];
}) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [refresh, setRefresh] = useState(0);
  const [report, setReport] = useState<EventDependencyReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const generation = useRef(0);
  const c = check.condition;
  const revision = useContext(ConditionQueryRevision);
  const signature = JSON.stringify(check);
  const md5 = catalog.profile.md5;
  const limited = depth >= 4 || trail.includes(conditionKey(c));
  useEffect(() => {
    const ticket = ++generation.current;
    setReport(null);
    setError(null);
    setBusy(false);
    if (!open || limited) return;
    setBusy(true);
    api<EventDependencyReport>("event_dependencies", {
      ...c,
      expected_rom_md5: md5,
    })
      .then((r) => {
        if (ticket === generation.current && r.rom_md5 === md5) setReport(r);
      })
      .catch((e) => {
        if (ticket === generation.current) setError(e);
      })
      .finally(() => {
        if (ticket === generation.current) setBusy(false);
      });
    return () => {
      generation.current++;
    };
  }, [open, limited, signature, md5, refresh, revision]);
  if (!["flag", "variable"].includes(c.kind)) return null;
  const more = async () => {
    if (!report || report.next_offset == null || busy) return;
    const ticket = generation.current;
    setBusy(true);
    setError(null);
    try {
      const r = await api<EventDependencyReport>("event_dependencies", {
        ...c,
        expected_rom_md5: md5,
        offset: report.next_offset,
      });
      if (ticket === generation.current && r.rom_md5 === md5)
        setReport({ ...r, writers: [...report.writers, ...r.writers] });
    } catch (e) {
      if (ticket === generation.current) setError(e);
    } finally {
      if (ticket === generation.current) setBusy(false);
    }
  };
  return (
    <details
      className="event-dependencies"
      onToggle={(e) => {
        if (e.target === e.currentTarget) setOpen(e.currentTarget.open);
      }}
    >
      <summary>{t("dependencyTrace")}</summary>
      {limited ? (
        <p className="muted">{t("dependencyLimit")}</p>
      ) : (
        <>
          <p className="muted">{t("dependencyHelp")}</p>
          {busy && <p role="status">{t("loading")}</p>}
          {!!error && (
            <details>
              <summary>{t("dependencyError")}</summary>
              <pre>{JSON.stringify(error, null, 2)}</pre>
            </details>
          )}
          {report && (
            <>
              <p>
                {t("dependencySnapshot")} ·{" "}
                {t(
                  report.condition.satisfied === true
                    ? "planConditionYes"
                    : report.condition.satisfied === false
                      ? "planConditionNo"
                      : "acqStatus_unknown",
                )}{" "}
                · {conditionLabel(report.condition, catalog, t)}
              </p>
              <button
                className="link-button"
                disabled={busy}
                onClick={() => setRefresh((v) => v + 1)}
              >
                {t("dependencyRefresh")}
              </button>
              {!report.writers.length && <p>{t("dependencyNone")}</p>}
              {report.writers.map((w, i) => (
                <article key={`${w.reference.offset}-${w.effect.offset}-${i}`}>
                  <strong>{effectLabel(w.effect, t)}</strong> ·{" "}
                  {t(`dependency_${w.reference.kind}`)}
                  <p>
                    {onMap ? (
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
                    ) : (
                      w.reference.map_name
                    )}
                  </p>
                  {w.reference.x == null && (
                    <p className="muted">{t("acqNoTile")}</p>
                  )}
                  {!!w.text.length && (
                    <details className="dependency-text">
                      <summary>{t("dependencyText")}</summary>
                      <p className="muted">{t("dependencyTextHelp")}</p>
                      {w.text.map((r) => (
                        <blockquote key={r.offset}>{r.text}</blockquote>
                      ))}
                    </details>
                  )}
                  <ConditionDetails
                    checks={w.conditions}
                    catalog={catalog}
                    onTarget={onTarget}
                    onMap={onMap}
                    dependencyDepth={depth + 1}
                    dependencyTrail={[...trail, conditionKey(c)]}
                  />
                  <p className="muted">{t("dependencyAccessUnknown")}</p>
                  {!w.path_complete && (
                    <p className="muted">{t("dependencyPathPartial")}</p>
                  )}
                  <details>
                    <summary>{t("evidence")}</summary>
                    <pre>{JSON.stringify(w, null, 2)}</pre>
                  </details>
                </article>
              ))}
              {report.next_offset != null && (
                <button disabled={busy} onClick={() => void more()}>
                  {t("dependencyMore")}
                </button>
              )}
              <p className="muted">
                {t("dependencyCoverage")
                  .replace("{checked}", String(report.coverage.checked_scripts))
                  .replace("{total}", String(report.coverage.total_scripts))}
              </p>
              <details>
                <summary>{t("evidence")}</summary>
                <pre>{JSON.stringify(report.coverage, null, 2)}</pre>
              </details>
            </>
          )}
        </>
      )}
    </details>
  );
}
export function conditionLabel(
  check: Check,
  catalog: Catalog,
  t: (key: string) => string,
): string {
  const c = check.condition;
  const op = c.taken ? c.comparison : ([4, 5, 3, 2, 0, 1][c.comparison] ?? -1);
  const symbol = ["<", "=", ">", "≤", "≥", "≠"][op] ?? "?";
  const isBag = ["bag_item", "bag_item_runtime"].includes(c.kind);
  let label: string;
  if (isBag) {
    const name = catalog.items.find((i) => i.id === c.id)?.name ?? `#${c.id}`;
    label =
      c.value === 0
        ? `${t(c.taken ? "conditionItemPresent" : "conditionItemAbsent")} ${name}`
        : `${name} ${symbol} ${c.value}`;
  } else if (["money", "money_runtime"].includes(c.kind)) {
    label = `${t("conditionMoney")} ${symbol} ${c.value}`;
  } else if (c.kind === "player_gender") {
    const gender = ["conditionPlayerMale", "conditionPlayerFemale"][c.value];
    label = `${t("conditionPlayerGender")} ${symbol} ${gender ? t(gender) : c.value}`;
  } else if (c.kind === "flag" && c.value === 1) {
    label = t(
      op === 2
        ? "conditionImpossible"
        : op === 3
          ? "conditionAlways"
          : [1, 4].includes(op)
            ? "conditionEventSet"
            : "conditionEventUnset",
    );
  } else if (c.kind === "variable") {
    label = `${t("conditionUnnamed")} ${symbol} ${c.value}`;
  } else {
    label = t("conditionUnknown");
  }
  if (check.actual != null) {
    const caption =
      check.unresolved === "alternate_bag_unresolved"
        ? "conditionOrdinaryBag"
        : "conditionActual";
    const actual =
      c.kind === "flag" && [0, 1].includes(check.actual)
        ? t(check.actual === 1 ? "conditionFlagSet" : "conditionFlagUnset")
        : c.kind === "player_gender" && [0, 1].includes(check.actual)
          ? t(
              check.actual === 0
                ? "conditionPlayerMale"
                : "conditionPlayerFemale",
            )
          : String(check.actual);
    label += ` · ${t(caption)} ${actual}`;
  }
  return label;
}

export function ConditionDetails({
  checks,
  conditions,
  catalog,
  onTarget,
  heading,
  onMap,
  dependencyDepth = 0,
  dependencyTrail = [],
}: {
  checks?: Check[];
  conditions?: ItemReward["conditions"];
  catalog: Catalog;
  onTarget?: (target: QueryTarget) => void;
  heading?: string;
  onMap?: (id: string, focus?: MapFocus) => void;
  dependencyDepth?: number;
  dependencyTrail?: string[];
}) {
  const { t } = useI18n();
  const rows: Check[] | undefined =
    checks ??
    conditions?.map((condition) => ({
      condition,
      satisfied: null,
      actual: null,
    }));
  if (!rows?.length) return null;
  const resource = rows.some((r) =>
    ["money", "money_runtime", "bag_item", "bag_item_runtime"].includes(
      r.condition.kind,
    ),
  );
  return (
    <div className="condition-details small">
      <strong>{heading ?? t("acqConditions")}</strong>
      <ul>
        {rows.map((check, index) => (
          <li key={index}>
            <span>
              {t(
                check.satisfied === true
                  ? "planConditionYes"
                  : check.satisfied === false
                    ? "planConditionNo"
                    : "acqStatus_unknown",
              )}{" "}
              ·{" "}
            </span>
            {onTarget &&
            ["bag_item", "bag_item_runtime"].includes(check.condition.kind) ? (
              <button
                className="link-button"
                onClick={() =>
                  onTarget({ kind: "item", id: check.condition.id })
                }
              >
                {conditionLabel(check, catalog, t)} ↗
              </button>
            ) : (
              <span>{conditionLabel(check, catalog, t)}</span>
            )}
            {check.unresolved && <p className="muted">{t(check.unresolved)}</p>}
            <Dependencies
              check={check}
              catalog={catalog}
              onTarget={onTarget}
              onMap={onMap}
              depth={dependencyDepth}
              trail={dependencyTrail}
            />
          </li>
        ))}
      </ul>
      {resource && <p className="muted">{t("conditionHoldingsHelp")}</p>}
    </div>
  );
}
