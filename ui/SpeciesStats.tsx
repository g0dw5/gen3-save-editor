import { configuredReference } from "./configuredReferences";
import {
  officialStats,
  referenceSource,
  suggestedReference,
} from "./officialStats";
import { statKeys, useI18n } from "./i18n";
import type { Catalog, SpeciesDetail } from "./types";

export function SpeciesStats({
  detail,
  catalog,
}: {
  detail: SpeciesDetail;
  catalog: Catalog;
}) {
  const { t } = useI18n();
  const configured = configuredReference(
    catalog.profile.md5,
    detail.species.id,
  );
  const suggestion = configured
    ? officialStats.find(
        (row) => row.key === configured.target && configured.status !== "none",
      )
    : suggestedReference(detail, catalog);
  const reference = suggestion;
  const stats = detail.species.stats;
  const total = stats.reduce((sum, value) => sum + value, 0);
  const referenceTotal = reference?.stats.reduce(
    (sum, value) => sum + value,
    0,
  );
  const difference = (value: number, original: number | undefined) =>
    original === undefined
      ? "—"
      : value === original
        ? "="
        : `${value > original ? "+" : ""}${value - original}`;
  return (
    <section className="species-stats" aria-label={t("baseStats")}>
      <h3>{t("baseStats")}</h3>
      <table className="base-stats-comparison">
        <thead>
          <tr>
            <th>{t("stat")}</th>
            <th>{t("officialReference")}</th>
            <th>{t("currentRom")}</th>
            <th>{t("statDifference")}</th>
          </tr>
        </thead>
        <tbody>
          {[0, 1, 2, 4, 5, 3].map((index) => (
            <tr key={index}>
              <th scope="row">{t(statKeys[index])}</th>
              <td>{reference?.stats[index] ?? "—"}</td>
              <td>
                <strong>{stats[index]}</strong>
                <span className="stat-track">
                  <span style={{ width: `${(stats[index] / 255) * 100}%` }} />
                </span>
              </td>
              <td
                className={
                  reference && stats[index] > reference.stats[index]
                    ? "stat-increase"
                    : reference && stats[index] < reference.stats[index]
                      ? "stat-decrease"
                      : "muted"
                }
              >
                {difference(stats[index], reference?.stats[index])}
              </td>
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr>
            <th>{t("baseStatTotal")}</th>
            <td>{referenceTotal ?? "—"}</td>
            <td>{total}</td>
            <td>{difference(total, referenceTotal)}</td>
          </tr>
        </tfoot>
      </table>
      <p className="small muted">
        {reference ? (
          <>
            <a
              href={referenceSource(reference.generation)}
              target="_blank"
              rel="noreferrer"
            >
              {t("referenceSource")} ·{" "}
              {t("generation").replace("{n}", String(reference.generation))}
            </a>{" "}
            · {reference.name}
            {reference.form ? ` · ${reference.form}` : ""}
            {!configured ? ` · ${t("referenceMatched")}` : ""}
            {configured?.status === "comparison"
              ? ` · ${t("referenceComparisonOnly")}`
              : ""}
          </>
        ) : (
          t(
            configured?.status === "none"
              ? "referenceConfiguredNone"
              : "referenceUnmatched",
          )
        )}
      </p>
    </section>
  );
}
