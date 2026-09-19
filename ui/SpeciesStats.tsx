import { configuredReference } from "./configuredReferences";
import { SelectField } from "./components";
import {
  officialStats,
  referenceSource,
  suggestedReference,
} from "./officialStats";
import { statKeys, useI18n } from "./i18n";
import type { Catalog, SpeciesDetail } from "./types";
import { useState } from "react";

export function SpeciesStats({
  detail,
  catalog,
}: {
  detail: SpeciesDetail;
  catalog: Catalog;
}) {
  const { t } = useI18n();
  const [choices, setChoices] = useState<Record<string, string>>(() => {
    try {
      const saved = JSON.parse(
        localStorage.getItem("gen3.statReferences") ?? "{}",
      );
      return saved && typeof saved === "object" && !Array.isArray(saved)
        ? saved
        : {};
    } catch {
      return {};
    }
  });
  const identity = `${catalog.profile.md5}:${detail.species.id}`;
  const configured = configuredReference(
    catalog.profile.md5,
    detail.species.id,
  );
  const suggestion = configured
    ? officialStats.find(
        (row) => row.key === configured.target && configured.status !== "none",
      )
    : suggestedReference(detail, catalog);
  const chosen = choices[identity];
  const reference =
    chosen === undefined
      ? suggestion
      : officialStats.find((row) => row.key === chosen);
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
      <SelectField
        searchable
        label={t("referenceEntry")}
        value={reference?.key ?? ""}
        onChange={(key) => {
          const next = { ...choices, [identity]: String(key) };
          setChoices(next);
          try {
            localStorage.setItem("gen3.statReferences", JSON.stringify(next));
          } catch {
            /* Browsing still works without persistent preferences. */
          }
        }}
        options={[
          { value: "", label: t("noOfficialReference") },
          ...officialStats.map((row) => ({
            value: row.key,
            label: `#${row.dex} ${row.form || row.name}${row.form && !row.form.includes(row.name) ? ` · ${row.name}` : ""}`,
          })),
        ]}
      />
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
            ·{" "}
            {chosen === undefined
              ? t(
                  configured?.status === "direct"
                    ? "referenceConfiguredDirect"
                    : configured?.status === "comparison"
                      ? "referenceConfiguredComparison"
                      : "referenceMatched",
                )
              : t("referenceChosen")}
          </>
        ) : (
          t(
            configured?.status === "none" && chosen === undefined
              ? "referenceConfiguredNone"
              : "referenceUnmatched",
          )
        )}
      </p>
      <p className="small muted">{t("referenceHelp")}</p>
    </section>
  );
}
