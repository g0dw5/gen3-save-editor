import { useContext, useEffect, useRef, useState } from "react";
import { api } from "./api";
import { ConditionDetails, ConditionQueryRevision } from "./ConditionDetails";
import { useI18n } from "./i18n";
import type {
  Catalog,
  MapFocus,
  QueryTarget,
  TrainerReferenceReport,
} from "./types";

export function TrainerLocationsPanel({
  catalog,
  trainerId,
  onMap,
  onTarget,
}: {
  catalog: Catalog;
  trainerId: number;
  onMap: (id: string, focus?: MapFocus) => void;
  onTarget: (target: QueryTarget) => void;
}) {
  const { t } = useI18n();
  const revision = useContext(ConditionQueryRevision);
  const generation = useRef(0);
  const [report, setReport] = useState<TrainerReferenceReport | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [refresh, setRefresh] = useState(0);
  const md5 = catalog.profile.md5;
  useEffect(() => {
    const ticket = ++generation.current;
    setReport(null);
    setError(null);
    setBusy(true);
    api<TrainerReferenceReport>("trainer_references", {
      expected_rom_md5: md5,
      trainer_id: trainerId,
    })
      .then((r) => {
        if (
          generation.current === ticket &&
          r.rom_md5 === md5 &&
          r.trainer_id === trainerId
        )
          setReport(r);
      })
      .catch((e) => {
        if (generation.current === ticket) setError(e);
      })
      .finally(() => {
        if (generation.current === ticket) setBusy(false);
      });
    return () => {
      generation.current++;
    };
  }, [md5, trainerId, revision, refresh]);
  const more = async () => {
    if (!report || report.next_offset == null || busy) return;
    const ticket = generation.current;
    setBusy(true);
    try {
      const r = await api<TrainerReferenceReport>("trainer_references", {
        expected_rom_md5: md5,
        trainer_id: trainerId,
        offset: report.next_offset,
      });
      if (
        generation.current === ticket &&
        r.rom_md5 === md5 &&
        r.trainer_id === trainerId
      )
        setReport({
          ...r,
          references: [...report.references, ...r.references],
        });
    } catch (e) {
      if (generation.current === ticket) setError(e);
    } finally {
      if (generation.current === ticket) setBusy(false);
    }
  };
  return (
    <details
      className="trainer-reference-panel reference-section"
      key={`${md5}:${trainerId}`}
    >
      <summary>
        {t("trainerReferenceTitle")}
        {report ? ` · ${report.total_matches}` : ""}
      </summary>
      <button
        className="link-button"
        onClick={() => setRefresh((v) => v + 1)}
        disabled={busy}
      >
        {t("dependencyRefresh")}
      </button>
      {busy && <p role="status">{t("loading")}</p>}
      {!!error && (
        <details open>
          <summary>{t("trainerReferenceError")}</summary>
          <pre>{JSON.stringify(error, null, 2)}</pre>
        </details>
      )}
      {report && (
        <>
          {!report.references.length && (
            <p className="muted">{t("trainerReferenceNone")}</p>
          )}
          {report.references.map((row, i) => (
            <article
              className="trainer-reference-entry"
              key={`${row.clue_id}:${row.battle.offset}:${i}`}
            >
              <p>
                {t(`trainerReferenceRole_${row.battle.role}`)} ·{" "}
                {t(`dependency_${row.reference.kind}`)}
              </p>
              <button
                className="link-button"
                onClick={() =>
                  onMap(
                    row.reference.map_id,
                    row.reference.x != null && row.reference.y != null
                      ? { x: row.reference.x, y: row.reference.y }
                      : undefined,
                  )
                }
              >
                {row.reference.map_name}
                {row.reference.x != null && row.reference.y != null
                  ? ` (${row.reference.x}, ${row.reference.y})`
                  : ""}{" "}
                ↗
              </button>
              {row.reference.x == null && (
                <p className="small muted">{t("acqNoTile")}</p>
              )}
              <ConditionDetails
                checks={row.conditions}
                catalog={catalog}
                onTarget={onTarget}
                onMap={onMap}
              />
              {!!row.visibility.length && (
                <ConditionDetails
                  checks={row.visibility}
                  heading={t("eventCluesVisibility")}
                  catalog={catalog}
                  onTarget={onTarget}
                  onMap={onMap}
                />
              )}
              <details>
                <summary>{t("evidence")}</summary>
                <pre>{JSON.stringify(row, null, 2)}</pre>
              </details>
            </article>
          ))}
          {report.next_offset != null && (
            <button onClick={more} disabled={busy}>
              {t("eventCluesNext")}
            </button>
          )}
          <details>
            <summary>{t("evidence")}</summary>
            <pre>
              {JSON.stringify(
                {
                  total_matches: report.total_matches,
                  coverage: report.coverage,
                  partial: report.partial,
                },
                null,
                2,
              )}
            </pre>
          </details>
        </>
      )}
    </details>
  );
}
