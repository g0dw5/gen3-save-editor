import { useEffect, useRef, useState } from "react";
import { api } from "./api";
import { useI18n } from "./i18n";
import type { Preview } from "./BreedingPanel";
import type {
  Catalog,
  CollectionBreedingRoute,
  CollectionPlan,
  QueryTarget,
} from "./types";

export function breedingCoverageSummary(
  coverage: CollectionPlan["breeding_coverage"],
  t: (key: string) => string,
): string[] {
  if (!coverage) return [];
  return [
    t("planBreedCoverage")
      .replace("{checked}", String(coverage.checked_pairs))
      .replace("{total}", String(coverage.total_pairs)),
    ...(coverage.truncated ? [t("planBreedBound")] : []),
    ...(coverage.failed_pairs ? [t("planBreedFailed")] : []),
  ];
}

export function breedingRouteSummary(
  route: CollectionBreedingRoute,
  catalog: Catalog,
  t: (key: string) => string,
): string[] {
  const speciesName = (id: number) =>
    catalog.species.find((s) => s.id === id)?.name ?? `#${id}`;
  return [
    t("planBreedTitle"),
    ...route.parents.map((p) => {
      const loc =
        p.location.kind === "party"
          ? `${t("party")} ${p.location.slot + 1}`
          : `${t("box")} ${p.location.box_index + 1} / ${p.location.slot + 1}`;
      const item = p.held_item
        ? (catalog.items.find((i) => i.id === p.held_item)?.name ??
          `#${p.held_item}`)
        : t("none");
      return `${loc}: ${p.nickname} · ${speciesName(p.species)} · ${t(p.gender)} · ${t("held_item")}: ${item}`;
    }),
    t("planBreedHelp"),
  ];
}

/** A full native receipt scenario for the exact suggested records, still read-only. */
export function CollectionBreeding({
  route,
  origin,
  catalog,
  onTarget,
  onError,
}: {
  route: CollectionBreedingRoute;
  origin: number;
  catalog: Catalog;
  onTarget: (target: QueryTarget) => void;
  onError: (error: unknown) => void;
}) {
  const { t } = useI18n();
  const [result, setResult] = useState<Preview | null>(null);
  const [busy, setBusy] = useState(false);
  const revision = useRef(0);
  useEffect(() => {
    revision.current++;
    setResult(null);
    setBusy(false);
    return () => {
      revision.current++;
    };
  }, [route, catalog.profile.md5]);
  const run = async () => {
    const ticket = ++revision.current;
    setBusy(true);
    setResult(null);
    try {
      const value = await api<Preview>("breeding_preview", {
        parents: route.parents.map((p) => ({
          kind: "stored",
          location: p.location,
        })),
        seed: route.seed,
        offspring_pid: route.offspring_pid,
      });
      if (ticket === revision.current && value.rom_md5 === catalog.profile.md5)
        setResult(value);
    } catch (e) {
      if (ticket === revision.current) onError(e);
    } finally {
      if (ticket === revision.current) setBusy(false);
    }
  };
  return (
    <div className="collection-breeding">
      <strong>{t("planBreedTitle")}</strong>
      {route.parents.map((p, i) => (
        <p key={i}>
          {p.location.kind === "party"
            ? `${t("party")} ${p.location.slot + 1}`
            : `${t("box")} ${p.location.box_index + 1} / ${p.location.slot + 1}`}{" "}
          · {p.nickname}{" "}
          <button
            className="link-button"
            onClick={() => onTarget({ kind: "species", id: p.species })}
          >
            {catalog.species.find((s) => s.id === p.species)?.name ??
              `#${p.species}`}{" "}
            ↗
          </button>{" "}
          · {t(p.gender)}
          {p.held_item > 0 && (
            <>
              {" "}
              · {t("held_item")}:{" "}
              <button
                className="link-button"
                onClick={() => onTarget({ kind: "item", id: p.held_item })}
              >
                {catalog.items.find((s) => s.id === p.held_item)?.name ??
                  `#${p.held_item}`}{" "}
                ↗
              </button>
            </>
          )}
        </p>
      ))}
      <p className="muted">{t("planBreedHelp")}</p>
      <button disabled={busy} onClick={run}>
        {busy ? t("loading") : t("planBreedVerify")}
      </button>
      {result && (
        <p>
          {t(
            result.child?.species === origin
              ? "planBreedMatches"
              : "planBreedChanged",
          )}
          {result.child && (
            <>
              {" "}
              ·{" "}
              <button
                className="link-button"
                onClick={() =>
                  onTarget({ kind: "species", id: result.child!.species })
                }
              >
                {catalog.species.find((s) => s.id === result.child!.species)
                  ?.name ?? `#${result.child.species}`}{" "}
                ↗
              </button>
            </>
          )}
        </p>
      )}
      <details>
        <summary>{t("evidence")}</summary>
        <pre>{JSON.stringify(result ?? route, null, 2)}</pre>
      </details>
    </div>
  );
}
