import type {
  AcquisitionSource,
  Catalog,
  ItemReward,
  QueryTarget,
} from "./types";
import { useI18n } from "./i18n";

type Check = AcquisitionSource["conditions"][number];
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
    label += ` · ${t(caption)} ${check.actual}`;
  }
  return label;
}

export function ConditionDetails({
  checks,
  conditions,
  catalog,
  onTarget,
  heading,
}: {
  checks?: Check[];
  conditions?: ItemReward["conditions"];
  catalog: Catalog;
  onTarget?: (target: QueryTarget) => void;
  heading?: string;
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
          </li>
        ))}
      </ul>
      {resource && <p className="muted">{t("conditionHoldingsHelp")}</p>}
    </div>
  );
}
