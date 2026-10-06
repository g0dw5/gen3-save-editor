import { useContext, useEffect, useRef, useState } from "react";
import { api } from "./api";
import { ConditionQueryRevision } from "./ConditionDetails";
import { TrainingIndividualSelect } from "./TrainingIndividualSelect";
import { SearchSelect } from "./SearchSelect";
import { useI18n } from "./i18n";
import { fromKey, locationKey } from "./types";
import type { Catalog, Pokemon, Snapshot } from "./types";
import type { TrainingService } from "./TrainingServices";

type Preview = {
  rom_md5: string;
  service_root: number;
  choice_index: number;
  before: Pokemon;
  after: Pokemon;
  party_stats_before: number[];
  party_stats_after: number[];
  native_level: number;
  minimum_level: number;
  level_satisfied: boolean;
  known_requirements_met: boolean | null;
  changed: boolean;
  stat_refresh: string;
  scenario: string;
  party_state: string;
  partial: boolean;
};
const stats = ["hp", "attack", "defense", "speed", "spAttack", "spDefense"];
export function TrainingServicePreview({
  catalog,
  save,
  service,
  choice,
  onError,
}: {
  catalog: Catalog;
  save: Snapshot | null;
  service: TrainingService;
  choice: TrainingService["choices"][number];
  onError: (error: unknown) => void;
}) {
  const { t } = useI18n();
  const revision = useContext(ConditionQueryRevision);
  const [individual, setIndividual] = useState("simulated");
  const [species, setSpecies] = useState(catalog.species[0]?.id ?? 1);
  const [level, setLevel] = useState(service.minimum_level);
  const [ivs, setIvs] = useState([0, 0, 0, 0, 0, 0]);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [busy, setBusy] = useState(false);
  const ticket = useRef(0);
  const md5 = catalog.profile.md5;
  const root = service.evidence.root;
  const reset = () => {
    ticket.current++;
    setPreview(null);
    setBusy(false);
  };
  useEffect(() => {
    reset();
    if (!save?.pokemon.some((p) => locationKey(p.location) === individual))
      setIndividual("simulated");
    return () => {
      ticket.current++;
    };
  }, [revision, save, md5, root, choice.menu_index]);
  const invalid =
    individual === "simulated"
      ? !Number.isInteger(level) ||
        level < 1 ||
        level > (catalog.profile.max_level ?? 100) ||
        ivs.some((v) => !Number.isInteger(v) || v < 0 || v > 31)
      : !save?.pokemon.some((p) => locationKey(p.location) === individual);
  const run = async () => {
    const current = ++ticket.current;
    setBusy(true);
    setPreview(null);
    try {
      const result = await api<Preview>("training_service_preview", {
        expected_rom_md5: md5,
        service_root: root,
        choice_index: choice.menu_index,
        individual:
          individual === "simulated"
            ? {
                kind: "simulated",
                species,
                level,
                ivs,
                evs: [0, 0, 0, 0, 0, 0],
                friendship: 70,
              }
            : { kind: "stored", location: fromKey(individual) },
      });
      if (
        current === ticket.current &&
        result.rom_md5 === md5 &&
        result.service_root === root &&
        result.choice_index === choice.menu_index
      )
        setPreview(result);
    } catch (error) {
      if (current === ticket.current) onError(error);
    } finally {
      if (current === ticket.current) setBusy(false);
    }
  };
  return (
    <details className="training-service-preview">
      <summary>{t("trainingServicePreview")}</summary>
      <p className="small muted">{t("trainingServicePreviewScope")}</p>
      <TrainingIndividualSelect
        catalog={catalog}
        save={save}
        label={t("trainingServiceIndividual")}
        value={individual}
        onChange={(value) => {
          setIndividual(value);
          reset();
        }}
      />
      {individual === "simulated" && (
        <fieldset>
          <legend>{t("trainingSimulated")}</legend>
          <p className="small muted">{t("trainingServiceSimulated")}</p>
          <SearchSelect
            label={t("trainingServiceSpecies")}
            value={species}
            options={catalog.species.map((s) => ({
              value: s.id,
              label: s.name,
            }))}
            onChange={(value) => {
              setSpecies(+value);
              reset();
            }}
          />
          <label>
            {t("trainingServiceLevel")}
            <input
              aria-label={t("trainingServiceLevel")}
              type="number"
              min={1}
              max={catalog.profile.max_level ?? 100}
              value={level}
              onChange={(e) => {
                setLevel(+e.target.value);
                reset();
              }}
            />
          </label>
          <div className="clock-offsets">
            {stats.map((stat, i) => (
              <label key={stat}>
                {t(stat)} {t("ivs")}
                <input
                  aria-label={`${t("trainingServiceIv")} · ${t(stat)}`}
                  type="number"
                  min={0}
                  max={31}
                  value={ivs[i]}
                  onChange={(e) => {
                    setIvs((values) =>
                      values.map((v, j) => (j === i ? +e.target.value : v)),
                    );
                    reset();
                  }}
                />
              </label>
            ))}
          </div>
        </fieldset>
      )}
      <button disabled={busy || invalid} onClick={run}>
        {t(busy ? "loading" : "trainingServiceRun")}
      </button>
      {preview && (
        <section className="training-service-result">
          <h4>{t("trainingServiceResult")}</h4>
          <p>
            {catalog.species.find((s) => s.id === preview.before.species)?.name}{" "}
            · {t("level")} {preview.native_level}
          </p>
          <p className="small">
            {t(
              preview.party_state === "stored_party"
                ? "trainingServiceStoredParty"
                : "trainingServiceProjectedParty",
            )}
          </p>
          <p>
            {t("trainingServiceKnownRequirements")} ·{" "}
            {t(
              preview.known_requirements_met === true
                ? "planConditionYes"
                : preview.known_requirements_met === false
                  ? "planConditionNo"
                  : "acqStatus_unknown",
            )}
          </p>
          {preview.level_satisfied &&
            preview.known_requirements_met !== true && (
              <p className="small warning-text">
                {t("trainingServiceHypothetical")}
              </p>
            )}
          {!preview.level_satisfied && (
            <p className="small warning-text">
              {t("trainingServiceBelowLevel")} {preview.minimum_level}
            </p>
          )}
          {preview.level_satisfied && (
            <p className="small muted">
              {t(
                preview.stat_refresh === "deferred"
                  ? "trainingServiceDeferred"
                  : "trainingServiceImmediate",
              )}
            </p>
          )}
          <div className="table-scroll">
            <table>
              <thead>
                <tr>
                  <th>{t("stats")}</th>
                  <th>{t("trainingServiceBaseIvs")}</th>
                  {service.kind === "hyper_training_flags" && (
                    <th>{t("trainingServiceFlags")}</th>
                  )}
                  <th>{t("trainingServicePartyStats")}</th>
                </tr>
              </thead>
              <tbody>
                {stats.map((stat, i) => (
                  <tr key={stat}>
                    <th>{t(stat)}</th>
                    <td>
                      {preview.before.ivs[i]} → {preview.after.ivs[i]}
                    </td>
                    {service.kind === "hyper_training_flags" && (
                      <td>
                        {t(
                          preview.before.hyper_trained?.[i]
                            ? "trainingServiceFlagYes"
                            : "trainingServiceFlagNo",
                        )}{" "}
                        →{" "}
                        {t(
                          preview.after.hyper_trained?.[i]
                            ? "trainingServiceFlagYes"
                            : "trainingServiceFlagNo",
                        )}
                      </td>
                    )}
                    <td>
                      {preview.party_stats_before[i]} →{" "}
                      {preview.party_stats_after[i]}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="small">
            {t("trainingServiceHp")} {preview.before.current_hp ?? "—"} →{" "}
            {preview.after.current_hp ?? "—"}
          </p>
          {!preview.changed && (
            <p className="small">{t("trainingServiceUnchanged")}</p>
          )}
          <details>
            <summary>{t("evidence")}</summary>
            <pre>{JSON.stringify(preview, null, 2)}</pre>
          </details>
        </section>
      )}
    </details>
  );
}
