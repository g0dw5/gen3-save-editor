import { useContext, useEffect, useRef, useState } from "react";
import { api } from "./api";
import { ConditionQueryRevision } from "./ConditionDetails";
import { SearchSelect } from "./SearchSelect";
import { useI18n } from "./i18n";
import { fromKey, locationKey } from "./types";
import type { Catalog, Pokemon, QueryTarget, Snapshot } from "./types";
type Offer = {
  item: number;
  stat: number;
  direction: string;
  handler: number;
  native_category: number;
  partial: boolean;
};
type TrainingCatalog = { rom_md5: string; offers: Offer[]; partial: boolean };
type Preview = {
  rom_md5: string;
  item: number;
  before: Pokemon;
  after: Pokemon;
  native_no_effect: boolean;
  changed: boolean;
  scenario: string;
  context: string;
  party_state: string;
  partial: boolean;
};
const stats = ["hp", "attack", "defense", "speed", "spAttack", "spDefense"];
export function TrainingPanel({
  catalog,
  save,
  onTarget,
  onError,
}: {
  catalog: Catalog;
  save: Snapshot | null;
  onTarget: (target: QueryTarget) => void;
  onError: (error: unknown) => void;
}) {
  const { t } = useI18n();
  const revision = useContext(ConditionQueryRevision);
  const [report, setReport] = useState<TrainingCatalog | null>(null);
  const [item, setItem] = useState(0);
  const [individual, setIndividual] = useState("simulated");
  const [species, setSpecies] = useState(catalog.species[0]?.id ?? 1);
  const [level, setLevel] = useState(5);
  const [evs, setEvs] = useState([0, 0, 0, 0, 0, 0]);
  const [friendship, setFriendship] = useState(70);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [busy, setBusy] = useState(false);
  const ticket = useRef(0);
  const md5 = catalog.profile.md5;
  const reset = () => {
    ticket.current++;
    setPreview(null);
    setBusy(false);
  };
  useEffect(() => {
    let cancelled = false;
    setReport(null);
    api<TrainingCatalog>("training_catalog", { expected_rom_md5: md5 })
      .then((r) => {
        if (!cancelled && r.rom_md5 === md5) {
          setReport(r);
          setItem(r.offers[0]?.item ?? 0);
        }
      })
      .catch((e) => {
        if (!cancelled) onError(e);
      });
    return () => {
      cancelled = true;
      ticket.current++;
    };
  }, [md5]);
  useEffect(() => {
    reset();
    if (
      !save ||
      !save.pokemon.some((p) => locationKey(p.location) === individual)
    )
      setIndividual("simulated");
  }, [revision, save]);
  const run = async () => {
    const current = ++ticket.current;
    setBusy(true);
    setPreview(null);
    try {
      const r = await api<Preview>("training_preview", {
        expected_rom_md5: md5,
        item,
        individual:
          individual === "simulated"
            ? { kind: "simulated", species, level, evs, friendship }
            : { kind: "stored", location: fromKey(individual) },
      });
      if (current === ticket.current && r.rom_md5 === md5 && r.item === item)
        setPreview(r);
    } catch (e) {
      if (current === ticket.current) onError(e);
    } finally {
      if (current === ticket.current) setBusy(false);
    }
  };
  const offer = report?.offers.find((o) => o.item === item);
  return (
    <section className="training-panel">
      <h2>{t("trainingPage")}</h2>
      <p className="small muted">{t("trainingScope")}</p>
      {!report ? (
        <p>{t("loading")}</p>
      ) : !report.offers.length ? (
        <p>{t("trainingNone")}</p>
      ) : (
        <>
          <SearchSelect
            label={t("trainingItem")}
            value={item}
            options={report.offers.map((o) => ({
              value: o.item,
              label: `${catalog.items.find((i) => i.id === o.item)?.name ?? o.item} · ${t(stats[o.stat])} · ${t(o.direction === "increase" ? "trainingIncrease" : "trainingDecrease")}`,
            }))}
            onChange={(v) => {
              setItem(+v);
              reset();
            }}
          />
          <button onClick={() => onTarget({ kind: "item", id: item })}>
            {t("trainingGetItem")}
          </button>
          <label>
            {t("trainingIndividual")}
            <select
              aria-label={t("trainingIndividual")}
              value={individual}
              onChange={(e) => {
                setIndividual(e.target.value);
                reset();
              }}
            >
              <option value="simulated">{t("trainingSimulated")}</option>
              {save?.pokemon
                .filter((p) => !p.pokemon.egg && p.pokemon.species)
                .map((p) => (
                  <option
                    key={locationKey(p.location)}
                    value={locationKey(p.location)}
                  >
                    {p.pokemon.nickname} ·{" "}
                    {
                      catalog.species.find((s) => s.id === p.pokemon.species)
                        ?.name
                    }{" "}
                    ·{" "}
                    {p.location.kind === "party"
                      ? `${t("party")} ${p.location.slot + 1}`
                      : `${save.boxes[p.location.box_index]?.name} ${p.location.slot + 1}`}
                  </option>
                ))}
            </select>
          </label>
          {individual === "simulated" && (
            <fieldset>
              <legend>{t("trainingSimulated")}</legend>
              <SearchSelect
                label={t("species")}
                value={species}
                options={catalog.species.map((s) => ({
                  value: s.id,
                  label: s.name,
                }))}
                onChange={(v) => {
                  setSpecies(+v);
                  reset();
                }}
              />
              <label>
                {t("level")}
                <input
                  aria-label={t("level")}
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
              <label>
                {t("friendship")}
                <input
                  aria-label={t("friendship")}
                  type="number"
                  min={0}
                  max={255}
                  value={friendship}
                  onChange={(e) => {
                    setFriendship(+e.target.value);
                    reset();
                  }}
                />
              </label>
              <div className="clock-offsets">
                {stats.map((stat, i) => (
                  <label key={stat}>
                    {t(stat)} {t("evs")}
                    <input
                      aria-label={`${t(stat)} ${t("evs")}`}
                      type="number"
                      min={0}
                      max={255}
                      value={evs[i]}
                      onChange={(e) => {
                        setEvs(
                          evs.map((v, j) => (i === j ? +e.target.value : v)),
                        );
                        reset();
                      }}
                    />
                  </label>
                ))}
              </div>
            </fieldset>
          )}
          <button
            disabled={
              busy ||
              !item ||
              (individual === "simulated" &&
                (!Number.isInteger(level) ||
                  level < 1 ||
                  level > (catalog.profile.max_level ?? 100) ||
                  evs.some((v) => !Number.isInteger(v) || v < 0 || v > 255) ||
                  !Number.isInteger(friendship) ||
                  friendship < 0 ||
                  friendship > 255))
            }
            onClick={run}
          >
            {busy ? t("loading") : t("trainingPreview")}
          </button>
          <p className="small muted">{t("trainingPreviewScope")}</p>
          {preview && (
            <article className="training-result">
              <h3>{t("trainingResult")}</h3>
              <p>
                {t(
                  preview.native_no_effect
                    ? "trainingNoEffect"
                    : "trainingAccepted",
                )}
              </p>
              <table>
                <thead>
                  <tr>
                    <th>{t("stat")}</th>
                    <th>{t("trainingBefore")}</th>
                    <th>{t("trainingAfter")}</th>
                  </tr>
                </thead>
                <tbody>
                  {stats.map((s, i) => (
                    <tr key={s}>
                      <td>
                        {t(s)} {t("evs")}
                      </td>
                      <td>{preview.before.evs[i]}</td>
                      <td>{preview.after.evs[i]}</td>
                    </tr>
                  ))}
                  <tr>
                    <td>{t("friendship")}</td>
                    <td>{preview.before.friendship}</td>
                    <td>{preview.after.friendship}</td>
                  </tr>
                </tbody>
              </table>
              {preview.before.evs.every(
                (v, i) => preview.after.evs[i] === v,
              ) && <p className="small muted">{t("trainingUnchangedEV")}</p>}
              {preview.party_state === "boxed_full_hp_scenario" && (
                <p className="small muted">{t("trainingBoxScenario")}</p>
              )}
              <p className="small muted">
                {t(
                  preview.context === "save_blocks"
                    ? "trainingSaveContext"
                    : "trainingZeroContext",
                )}
              </p>
              <details>
                <summary>{t("evidence")}</summary>
                <pre>{JSON.stringify({ offer, preview }, null, 2)}</pre>
              </details>
            </article>
          )}
        </>
      )}
    </section>
  );
}
