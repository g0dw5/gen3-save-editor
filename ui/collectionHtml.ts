import type { Catalog, CollectionPlan, GameMap, QueryTarget } from "./types";
import { tradeSummary } from "./TradeDetails";
import { clockSummary } from "./ClockDetails";
import { evolutionLabel } from "./referenceLabels";

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
  const name = (v: QueryTarget) =>
    (v.kind === "species" ? catalog.species : catalog.items).find(
      (x) => x.id === v.id,
    )?.name ?? `#${v.id}`;
  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  const title = t("collection");
  return `<!doctype html><html lang="${document.documentElement.lang || "zh"}"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src 'none'; base-uri 'none'; form-action 'none'"><title>${esc(title)}</title><style>body{font:16px/1.7 system-ui,sans-serif;background:#f4f7fa;color:#182532;max-width:980px;margin:auto;padding:22px}section{background:white;padding:20px;margin:18px 0;border-radius:12px}table{border-collapse:collapse;width:100%;font-size:14px}td,th{text-align:left;border-bottom:1px solid #dce3e9;padding:10px;vertical-align:top}small{color:#576775}details{margin-top:8px}pre{white-space:pre-wrap;overflow-wrap:anywhere}.task{margin-bottom:16px}h1{font-size:27px}h2{font-size:21px}input{width:20px;height:20px;vertical-align:middle}@media(max-width:600px){body{padding:12px}section{padding:12px}td,th{padding:5px;font-size:12px}}</style><h1>${esc(title)}</h1><p>${esc(catalog.profile.label)} · ${esc(t("planGeneratedAt"))}: ${esc(new Date().toLocaleString())}</p><p>${esc(t("planHelp"))}</p><p>${esc(t("planFamilyHelp"))}</p><p>${esc(t("acqSaveOverlay"))}</p><p>${esc(t("planBasis"))}: ${esc(t(plan.basis === "dex" ? "planDex" : "planIndividuals"))} · ${esc(t("planMissing"))}: ${plan.missing_count}</p>${plan.clock ? `<p>${esc(clockSummary(plan.clock, t))}</p>${plan.clock.saved ? `<p><small>${esc(t("clockSnapshotHelp"))}</small></p>` : ""}${plan.clock.forced_night ? `<p>${esc(t("clockForcedNight"))}</p>` : ""}` : ""}<small>MD5 ${esc(plan.rom_md5)}</small>${plan.regions
    .map(
      (region) =>
        `<section><h2>${esc(region.region === null ? t("planNoRegion") : (catalog.met_locations.find((r) => r.id === region.region)?.name ?? `#${region.region}`))}</h2>${region.tasks
          .map((task) => {
            const s = task.source;
            const entrance = plan.entrances.find((e) => e.map_id === s?.map_id);
            const conditions = s?.conditions
              .map(
                (c) =>
                  `${t(c.satisfied === true ? "planConditionYes" : c.satisfied === false ? "planConditionNo" : "acqStatus_unknown")}: ${c.condition.kind} ${c.condition.id} ${["<", "=", ">", "≤", "≥", "≠"][c.condition.comparison] ?? "?"} ${c.condition.value}${c.condition.taken ? "" : " (false)"}`,
              )
              .join("; ");
            return `<div class="task"><h3><input type="checkbox" aria-label="${esc(t("planCheck"))}"> ${esc(name(task.target))}</h3>${task.family.length ? `<small>${esc(t("planFamily"))}: ${esc(task.family.map((id) => name({ kind: "species", id })).join(" / "))}</small>` : ""}<p>${esc(s ? t(`acqStatus_${s.status}`) : t("acqNoSource"))} · ${esc(s?.map_id ? mapName(s.map_id) : t("planNoRegion"))}${s?.x !== null && s?.x !== undefined ? ` (${s.x}, ${s.y})` : ""}</p>${s?.min_level !== null && s?.min_level !== undefined ? `<p>Lv. ${s.min_level}–${s.max_level ?? s.min_level}${s.encounter_percent !== null ? ` · ${esc(t("acqEncounterChance"))} ${s.encounter_percent}%` : ""}</p>` : ""}${s?.script_source ? `<p>${esc(t("acqScriptSourceHelp"))}</p>` : ""}${
              s?.script_source?.trade
                ? tradeSummary(s.script_source, s.trade_context, catalog, t)
                    .map((line) => `<p>${esc(line)}</p>`)
                    .join("")
                : ""
            }${s?.receipt ? `<p>${esc(t("acqGiftReceiptHelp"))}</p>` : ""}${s && ["gift", "pc"].includes(s.kind) && s.receipt_flag == null ? `<p>${esc(t("acqReceiptUnknown"))}</p>` : ""}${s?.underfoot ? `<p>${esc(t("mapHiddenUnderfoot"))}</p>` : ""}${s?.evolution ? `<p>${esc(evolutionLabel(s.evolution, catalog, catalog.type_names, t))}</p>` : ""}${s?.periods.length ? `<p>${esc(t("planPeriods"))}: ${esc(s.periods.map((p) => t(({ base: "encounterBase", morning: "encounterMorning", day: "encounterDay", dusk: "encounterDusk", night: "encounterNight" } as Record<string, string>)[p] ?? p)).join(" / "))}</p>` : ""}${s?.in_scenario !== null && s?.in_scenario !== undefined ? `<p>${esc(t(s.in_scenario ? "clockInsideScenario" : "clockOutsideScenario"))}</p>` : ""}${conditions ? `<p>${esc(t("acqConditions"))}: ${esc(conditions)}</p>` : ""}${
              entrance?.chains.length
                ? `<p>${esc(t("navApproaches"))}:</p><ul>${entrance.chains
                    .slice(0, 3)
                    .map(
                      (chain) =>
                        `<li>${esc(
                          chain
                            .map(
                              (e) =>
                                `${mapName(e.from)}${e.x !== null ? ` (${e.x}, ${e.y})` : ""}`,
                            )
                            .concat(mapName(entrance.map_id))
                            .join(" → "),
                        )}</li>`,
                    )
                    .join("")}</ul>`
                : `<p><small>${esc(t("navNoApproach"))}</small></p>`
            }<details><summary>${esc(t("evidence"))}</summary><pre>${esc(JSON.stringify(task, null, 2))}</pre></details></div>`;
          })
          .join("")}</section>`,
    )
    .join("")}</html>`;
}
