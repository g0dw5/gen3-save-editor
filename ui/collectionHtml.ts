import {
  acquisitionSourceSummary,
  acquisitionTargetName,
} from "./acquisitionLabels";
import { heldSummary } from "./WildHeldDetails";
import { conditionLabel, conditionKey, effectLabel } from "./ConditionDetails";
import type {
  Catalog,
  CollectionPlan,
  EntranceSuggestion,
  GameMap,
  MapLink,
  QueryTarget,
} from "./types";
import { entranceState } from "./EntranceRoutes";
import { tradeSummary } from "./TradeDetails";
import { clockSummary } from "./ClockDetails";
import { evolutionLabel } from "./referenceLabels";
import { preparationSummary } from "./CollectionPreparation";
import { breedingCoverageSummary } from "./CollectionBreeding";

/** Standalone, escaped, non-executable report; never embeds ROM artwork or save bytes. */
export function collectionHtml(
  plan: CollectionPlan,
  catalog: Catalog,
  maps: GameMap[],
  t: (key: string) => string,
): string {
  const esc = (v: unknown) =>
    String(v).replace(
      /[&<>"']/g,
      (c) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          '"': "&quot;",
          "'": "&#39;",
        })[c]!,
    );
  const name = (v: QueryTarget) => acquisitionTargetName(v, catalog);
  const anchor = (region: number, task: number) => `task-${region}-${task}`;
  const targets = new Map<string, string>();
  plan.regions.forEach((region, i) =>
    region.tasks.forEach((task, j) => {
      const key = `${task.target.kind}:${task.target.id}`;
      if (!targets.has(key)) targets.set(key, anchor(i, j));
    }),
  );
  const relatedHtml = (target: QueryTarget) => {
    const destination = targets.get(`${target.kind}:${target.id}`);
    return destination
      ? `<a href="#${esc(destination)}">${esc(name(target))}</a>`
      : esc(name(target));
  };
  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  const prerequisiteId = (c: Parameters<typeof conditionKey>[0]) =>
    `prerequisite-${encodeURIComponent(conditionKey(c))}`;
  const checkHtml = (c: Parameters<typeof conditionLabel>[0]) => {
    const label = `${t(c.satisfied === true ? "planConditionYes" : c.satisfied === false ? "planConditionNo" : "acqStatus_unknown")}: ${conditionLabel(c, catalog, t)}${c.unresolved ? ` · ${t(c.unresolved)}` : ""}`;
    return plan.prerequisites?.reports.some(
      (r) => conditionKey(r.condition.condition) === conditionKey(c.condition),
    )
      ? `<a href="#${esc(prerequisiteId(c.condition))}">${esc(label)}</a>`
      : esc(label);
  };
  const passageHtml = (path: MapLink[]) =>
    path
      .filter((e) => e.script)
      .map((e) => {
        const script = e.script!;
        return `<p>${esc(mapName(e.from))} · ${esc(t("navScriptWarp"))}</p><p>${esc(t(script.source_kind === "npc" ? "navTalkPassage" : "navScriptHelp"))}</p>${script.checks.length ? `<p>${esc(t("acqConditions"))}: ${script.checks.map(checkHtml).join("; ")}</p>` : ""}<p>${esc(t("navScriptAccessUnknown"))}</p><details><summary>${esc(t("evidence"))}</summary><pre>${esc(JSON.stringify(e, null, 2))}</pre></details>`;
      })
      .join("");
  const entranceHtml = (entry?: EntranceSuggestion) => {
    if (!entry) return "";
    const tile = (id: string, x: number | null, y: number | null) =>
      `${mapName(id)}${x != null && y != null ? ` (${x}, ${y})` : ""}`;
    const paths = entry.chains
      .map((path, i) => {
        const last = path.at(-1);
        const label = path
          .map((e) => tile(e.from, e.x, e.y))
          .concat(
            tile(entry.map_id, last?.target_x ?? null, last?.target_y ?? null),
          )
          .join(" → ");
        const state = entranceState(path);
        return `<details open class="entrance-path"><summary>${esc(t("planEntranceNumber").replace("{n}", String(i + 1)))}${state ? ` · ${esc(t(state))}` : ""}</summary><p>${esc(label)}</p>${passageHtml(path)}</details>`;
      })
      .join("");
    const unresolved = entry.unresolved_incoming ?? [];
    return `<div class="collection-entrances"><p>${esc(t("navApproaches"))}</p><p>${esc(t("planEntranceAlternatives"))}</p>${paths || `<p>${esc(t("navNoApproach"))}</p>`}${unresolved.length ? `<details class="entrance-unresolved"><summary>${esc(t("navUnresolved"))} (${unresolved.length})</summary>${unresolved.map((e) => `<p>${esc(tile(e.from, e.x, e.y))}</p>${passageHtml([e])}`).join("")}</details>` : ""}${entry.truncated ? `<p>${esc(t("navBounded"))}</p>` : ""}<p>${esc(t("navHelp"))}</p></div>`;
  };
  const routeHtml = (index: number) => {
    const route = plan.prerequisites?.routes?.find(
      (r) => r.report_index === index,
    );
    if (!route?.goals.length) return "";
    return `<p>${esc(t("planAffectedGoals"))}: ${route.goals
      .map(([region, task]) => {
        const goal = plan.regions[region]?.tasks[task];
        return goal
          ? `<a href="#${anchor(region, task)}">${esc(name(goal.target))}</a>`
          : "";
      })
      .join(" / ")}</p>`;
  };
  const candidateHtml = (reportIndex: number, writerIndex: number) => {
    const route = plan.prerequisites?.routes?.find(
      (r) => r.report_index === reportIndex,
    );
    const candidate = route?.candidates.find(
      (c) => c.writer_index === writerIndex,
    );
    if (!candidate) return "";
    return `${candidate.recursive ? `<p>${esc(t("planRecursiveClue"))}</p>` : ""}${candidate.untraced_conditions.length || candidate.entry_untraced_conditions?.length ? `<p>${esc(t("planUntracedGuards"))}</p>` : ""}`;
  };
  const appendix = plan.prerequisites
    ? `<section id="prerequisites"><h2>${esc(t("dependencyTrace"))}</h2><p>${esc(t("dependencyHelp"))}</p><p>${esc(t("planPrerequisiteHelp"))}</p>${plan.prerequisites.truncated || plan.prerequisites.skipped_conditions ? `<p>${esc(t("dependencyAppendixLimit"))}</p>` : ""}${plan.prerequisites.reports
        .map(
          (report, reportIndex) =>
            `<article id="${esc(prerequisiteId(report.condition.condition))}"><h3>${esc(t("planPrerequisiteNumber").replace("{n}", String(reportIndex + 1)))} · ${esc(conditionLabel(report.condition, catalog, t))}</h3>${routeHtml(reportIndex)}<p>${esc(t(report.condition.satisfied === true ? "planConditionYes" : report.condition.satisfied === false ? "planConditionNo" : "acqStatus_unknown"))}</p>${!report.writers.length ? `<p>${esc(t("dependencyNone"))}</p>` : ""}${report.writers
              .map((w, writerIndex) => {
                if (report.condition.satisfied === true) return "";
                const entry = plan.prerequisites?.entrances.find(
                  (e) => e.map_id === w.reference.map_id,
                );
                return `<div class="task">${candidateHtml(reportIndex, writerIndex)}<strong>${esc(effectLabel(w.effect, t))} · ${esc(t(`dependency_${w.reference.kind}`))}</strong><p>${esc(w.reference.map_name)}${w.reference.x != null ? ` (${w.reference.x}, ${w.reference.y})` : ` · ${esc(t("acqNoTile"))}`}</p><p>${esc(t("dependencyAccessUnknown"))}</p>${w.text.length ? `<details><summary>${esc(t("dependencyText"))}</summary><p>${esc(t("dependencyTextHelp"))}</p>${w.text.map((r) => `<blockquote>${esc(r.text)}</blockquote>`).join("")}</details>` : ""}${w.conditions.length ? `<p>${esc(t("acqConditions"))}: ${w.conditions.map(checkHtml).join("; ")}</p>` : ""}${entranceHtml(
                  entry,
                )}${!w.path_complete ? `<p>${esc(t("dependencyPathPartial"))}</p>` : ""}<details><summary>${esc(t("evidence"))}</summary><pre>${esc(JSON.stringify(w, null, 2))}</pre></details></div>`;
              })
              .join(
                "",
              )}<details><summary>${esc(t("evidence"))}</summary><pre>${esc(JSON.stringify(report.coverage, null, 2))}</pre></details></article>`,
        )
        .join("")}<p>${esc(t("navHelp"))}</p></section>`
    : "";
  const title = t("collection");
  return `<!doctype html><html lang="${document.documentElement.lang || "zh"}"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src 'none'; base-uri 'none'; form-action 'none'"><title>${esc(title)}</title><style>body{font:16px/1.7 system-ui,sans-serif;background:#f4f7fa;color:#182532;max-width:980px;margin:auto;padding:22px}section{background:white;padding:20px;margin:18px 0;border-radius:12px}blockquote{white-space:pre-wrap}table{border-collapse:collapse;width:100%;font-size:14px}td,th{text-align:left;border-bottom:1px solid #dce3e9;padding:10px;vertical-align:top}small{color:#576775}details{margin-top:8px}pre{white-space:pre-wrap;overflow-wrap:anywhere}.task{margin-bottom:16px}h1{font-size:27px}h2{font-size:21px}input{width:20px;height:20px;vertical-align:middle}@media(max-width:600px){body{padding:12px}section{padding:12px}td,th{padding:5px;font-size:12px}}</style><h1>${esc(title)}</h1><p>${esc(catalog.profile.label)} · ${esc(t("planGeneratedAt"))}: ${esc(new Date().toLocaleString())}</p><p>${esc(t("planHelp"))}</p><p>${esc(t("planFamilyHelp"))}</p><p>${esc(t("acqSaveOverlay"))}</p><p>${esc(t("planBasis"))}: ${esc(t(plan.basis === "dex" ? "planDex" : "planIndividuals"))} · ${esc(t("planMissing"))}: ${plan.missing_count}</p>${plan.clock ? `<p>${esc(clockSummary(plan.clock, t))}</p>${plan.clock.saved ? `<p><small>${esc(t("clockSnapshotHelp"))}</small></p>` : ""}${plan.clock.forced_night ? `<p>${esc(t("clockForcedNight"))}</p>` : ""}` : ""}<small>MD5 ${esc(plan.rom_md5)}</small>${plan.entrance_coverage ? `<details><summary>${esc(t("evidence"))} · ${esc(t("navApproaches"))}</summary><pre>${esc(JSON.stringify({ coverage: plan.entrance_coverage, diagnostics: plan.entrance_diagnostics }, null, 2))}</pre></details>` : ""}${plan.entrance_coverage?.truncated || plan.entrance_coverage?.failed_scripts ? `<p>${esc(t("dependencyAppendixLimit"))}</p>` : ""}${breedingCoverageSummary(
    plan.breeding_coverage,
    t,
  )
    .map((line) => `<p>${esc(line)}</p>`)
    .join("")}${plan.regions
    .map(
      (region, regionIndex) =>
        `<section><h2>${esc(region.region === null ? t("planNoRegion") : (catalog.met_locations.find((r) => r.id === region.region)?.name ?? `#${region.region}`))}</h2>${region.tasks
          .map((task, taskIndex) => {
            const s = task.source;
            const preparation = task.preparation;
            const prepEntrance = plan.entrances.find(
              (e) => e.map_id === preparation?.source?.map_id,
            );
            const preparationHtml = preparation
              ? `<div class="collection-preparation">${preparationSummary(
                  preparation,
                  catalog,
                  maps,
                  t,
                )
                  .map((line) => `<p>${esc(line)}</p>`)
                  .join("")}${entranceHtml(prepEntrance)}</div>`
              : "";
            const entrance = plan.entrances.find((e) => e.map_id === s?.map_id);
            const conditions = s?.conditions.map(checkHtml).join("; ");
            return `<div class="task" id="${anchor(regionIndex, taskIndex)}"><h3><input type="checkbox" aria-label="${esc(t("planCheck"))}"> ${esc(name(task.target))}</h3>${task.family.length ? `<small>${esc(t("planFamily"))}: ${esc(task.family.map((id) => name({ kind: "species", id })).join(" / "))}</small>` : ""}<p>${esc(s ? t(`acqStatus_${s.status}`) : t("acqNoSource"))} · ${esc(s?.map_id ? mapName(s.map_id) : t("planNoRegion"))}${s?.x !== null && s?.x !== undefined ? ` (${s.x}, ${s.y})` : ""}</p>${
              s
                ? acquisitionSourceSummary(s, catalog, t)
                    .map((line) => `<p>${esc(line)}</p>`)
                    .join("")
                : ""
            }${s?.related.length ? `<p>${esc(t("acqRelatedTargets"))}: ${s.related.map(relatedHtml).join(" / ")}</p>` : ""}${s?.script_source ? `<p>${esc(t("acqScriptSourceHelp"))}</p>` : ""}${
              s?.script_source?.trade
                ? tradeSummary(s.script_source, s.trade_context, catalog, t)
                    .map((line) => `<p>${esc(line)}</p>`)
                    .join("")
                : ""
            }${
              s
                ? heldSummary(s, task.target.id, catalog, t)
                    .map((line) => `<p>${esc(line)}</p>`)
                    .join("")
                : ""
            }${s?.receipt ? `<p>${esc(t("acqGiftReceiptHelp"))}</p>` : ""}${s && ["gift", "pc"].includes(s.kind) && s.receipt_flag == null ? `<p>${esc(t("acqReceiptUnknown"))}</p>` : ""}${s?.underfoot ? `<p>${esc(t("mapHiddenUnderfoot"))}</p>` : ""}${s?.evolution ? `<p>${esc(evolutionLabel(s.evolution, catalog, catalog.type_names, t))}</p>` : ""}${s?.in_scenario !== null && s?.in_scenario !== undefined ? `<p>${esc(t(s.in_scenario ? "clockInsideScenario" : "clockOutsideScenario"))}</p>` : ""}${conditions ? `<p>${esc(t("acqConditions"))}: ${conditions}</p>${s?.conditions.some((c) => ["money", "money_runtime", "bag_item", "bag_item_runtime"].includes(c.condition.kind)) ? `<p><small>${esc(t("conditionHoldingsHelp"))}</small></p>` : ""}` : ""}${entranceHtml(
              entrance,
            )}${preparationHtml}<details><summary>${esc(t("evidence"))}</summary><pre>${esc(JSON.stringify(task, null, 2))}</pre></details></div>`;
          })
          .join("")}</section>`,
    )
    .join("")}${appendix}</html>`;
}
