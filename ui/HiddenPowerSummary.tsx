import { Types } from "./components";
import { hiddenPower } from "./hiddenPower";
import { useI18n } from "./i18n";
import type { Catalog } from "./types";

export function HiddenPowerSummary({
  catalog,
  ivs,
}: {
  catalog: Catalog;
  ivs: readonly number[];
}) {
  const { t } = useI18n();
  const rules = catalog.profile.hidden_power;
  if (!rules) return null;
  const value = hiddenPower(rules, ivs);
  return (
    <div className="hidden-power-summary" role="status">
      <div>
        <strong>
          {catalog.moves[rules.move_id]?.name ?? t("hiddenPower")}
        </strong>
        {value ? (
          <>
            <Types values={[value.type]} />
            <span>
              {t("power")} · {value.power}
            </span>
          </>
        ) : (
          <span>{t("hiddenPowerUnknown")}</span>
        )}
      </div>
      <p className="small muted">
        {t(
          rules.formula === "gen6_fixed60"
            ? "hiddenPowerFixedHelp"
            : "hiddenPowerHelp",
        )}
      </p>
    </div>
  );
}
