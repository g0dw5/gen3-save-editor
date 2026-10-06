import { useContext, useEffect, useRef, useState } from "react";
import { api } from "./api";
import { ConditionQueryRevision } from "./ConditionDetails";
import { SearchSelect } from "./SearchSelect";
import { TrainingServices } from "./TrainingServices";
import { TrainingIndividualSelect } from "./TrainingIndividualSelect";
import { useI18n } from "./i18n";
import { fromKey, locationKey } from "./types";
import type {
  Catalog,
  MapFocus,
  Pokemon,
  QueryTarget,
  Snapshot,
} from "./types";
type Offer = {
  item: number;
  stat: number;
  direction: string;
  handler: number;
  native_category: number;
  partial: boolean;
};
type NatureOffer = {
  item: number;
  nature: number;
  handler: number;
  partial: boolean;
};
type AbilityOffer = {
  item: number;
  handler: number;
  mechanism: string;
  random_pid: boolean;
  requires_rng_seed?: boolean;
  partial: boolean;
};
type TrainingCatalog = {
  rom_md5: string;
  offers: Offer[];
  nature_items?: NatureOffer[];
  ability_items?: AbilityOffer[];
  partial: boolean;
};
type Preview = {
  rom_md5: string;
  item: number;
  before: Pokemon;
  after: Pokemon;
  party_stats_before?: number[];
  party_stats_after?: number[];
  native_no_effect: boolean;
  changed: boolean;
  scenario: string;
  context: string;
  party_state: string;
  effect_scope?: string;
  ability_target?: number | null;
  rng_seed?: number | null;
  rng_after?: number | null;
  partial: boolean;
};
const stats = ["hp", "attack", "defense", "speed", "spAttack", "spDefense"];
export function TrainingPanel({
  catalog,
  save,
  onTarget,
  onMap,
  onError,
}: {
  catalog: Catalog;
  save: Snapshot | null;
  onTarget: (target: QueryTarget) => void;
  onMap: (id: string, focus?: MapFocus) => void;
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
  const [nature, setNature] = useState(26);
  const [ability, setAbility] = useState(0);
  const [seed, setSeed] = useState("42");
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
          setItem(
            r.offers[0]?.item ??
              r.nature_items?.[0]?.item ??
              r.ability_items?.[0]?.item ??
              0,
          );
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
        ...(abilityRequiresSeed ? { rng_seed: Number(seed) } : {}),
        individual:
          individual === "simulated"
            ? {
                kind: "simulated",
                species,
                level,
                evs,
                friendship,
                ...(report?.ability_items?.length
                  ? { ability_slot: ability }
                  : {}),
                ...(report?.nature_items?.length
                  ? { nature_override: nature }
                  : {}),
              }
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
  const natureOffer = report?.nature_items?.find((o) => o.item === item);
  const abilityOffer = report?.ability_items?.find((o) => o.item === item);
  const abilityRequiresSeed =
    abilityOffer?.requires_rng_seed ?? abilityOffer?.random_pid;
  const abilityName = (id: number) =>
    catalog.abilities.find((a) => a.id === id)?.name ?? String(id);
  const abilitySlotName = (slot: number) =>
    t(
      slot === 2
        ? "trainingHiddenAbility"
        : slot === 1
          ? "trainingNormalAbility2"
          : "trainingNormalAbility1",
    );
  const initialAbilities =
    catalog.species.find((s) => s.id === species)?.abilities ?? [];
  const invalidSeed =
    abilityRequiresSeed &&
    (!seed.trim() ||
      !Number.isInteger(Number(seed)) ||
      Number(seed) < 0 ||
      Number(seed) > 4294967295);
  const natureName = (id: number) =>
    catalog.natures.find((n) => n.id === id)?.name ?? String(id);
  return (
    <section className="training-panel">
      <h2>{t("trainingPage")}</h2>
      <p className="small muted">{t("trainingScope")}</p>
      <TrainingServices
        catalog={catalog}
        save={save}
        onMap={onMap}
        onTarget={onTarget}
        onError={onError}
      />
      {!report ? (
        <p>{t("loading")}</p>
      ) : !report.offers.length &&
        !report.nature_items?.length &&
        !report.ability_items?.length ? (
        <p>{t("trainingNone")}</p>
      ) : (
        <>
          <SearchSelect
            label={t("trainingItem")}
            value={item}
            options={[
              ...report.offers.map((o) => ({
                value: o.item,
                label: `${catalog.items.find((i) => i.id === o.item)?.name ?? o.item} · ${t(stats[o.stat])} · ${t(o.direction === "increase" ? "trainingIncrease" : "trainingDecrease")}`,
              })),
              ...(report.nature_items ?? []).map((o) => ({
                value: o.item,
                label: `${catalog.items.find((i) => i.id === o.item)?.name ?? o.item} · ${t("trainingNature")} · ${natureName(o.nature)}`,
              })),
              ...(report.ability_items ?? []).map((o) => ({
                value: o.item,
                label: `${catalog.items.find((i) => i.id === o.item)?.name ?? o.item} · ${t(o.mechanism === "hidden_toggle" ? "trainingHiddenChange" : "trainingNormalChange")}`,
              })),
            ]}
            onChange={(v) => {
              setItem(+v);
              reset();
            }}
          />
          <button onClick={() => onTarget({ kind: "item", id: item })}>
            {t("trainingGetItem")}
          </button>
          <TrainingIndividualSelect
            catalog={catalog}
            save={save}
            label={t("trainingIndividual")}
            value={individual}
            onChange={(value) => {
              setIndividual(value);
              reset();
            }}
          />
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
                  setAbility(0);
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
              {!!report.nature_items?.length && (
                <label>
                  {t("trainingInitialNature")}
                  <select
                    aria-label={t("trainingInitialNature")}
                    value={nature}
                    onChange={(e) => {
                      setNature(+e.target.value);
                      reset();
                    }}
                  >
                    <option value={26}>{t("trainingPIDNature")}</option>
                    {catalog.natures.map((n) => (
                      <option key={n.id} value={n.id}>
                        {n.name}
                      </option>
                    ))}
                  </select>
                </label>
              )}
              {!!report.ability_items?.length && (
                <label>
                  {t("trainingInitialAbility")}
                  <select
                    aria-label={t("trainingInitialAbility")}
                    value={ability}
                    onChange={(e) => {
                      setAbility(+e.target.value);
                      reset();
                    }}
                  >
                    {initialAbilities.map((id, slot) => (
                      <option key={slot} value={slot} disabled={!id}>
                        {abilitySlotName(slot)} ·{" "}
                        {id ? abilityName(id) : t("none")}
                      </option>
                    ))}
                  </select>
                </label>
              )}
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
          {abilityRequiresSeed && (
            <label>
              {t("trainingAbilitySeed")}
              <input
                aria-label={t("trainingAbilitySeed")}
                type="number"
                min={0}
                max={4294967295}
                step={1}
                value={seed}
                onChange={(e) => {
                  setSeed(e.target.value);
                  reset();
                }}
              />
              <span className="small muted">
                {t("trainingAbilitySeedScope")}
              </span>
            </label>
          )}
          <button
            disabled={
              busy ||
              !!invalidSeed ||
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
              {preview.effect_scope === "nature_persistent_stage" && (
                <p className="small muted">{t("trainingNatureScope")}</p>
              )}
              {preview.effect_scope === "ability_persistent_stage" && (
                <p className="small muted">
                  {t(
                    abilityOffer?.random_pid
                      ? "trainingAbilityPidScope"
                      : "trainingAbilitySlotScope",
                  )}
                </p>
              )}
              {abilityRequiresSeed && !abilityOffer?.random_pid && (
                <p className="small muted">
                  {t("trainingAbilityRandomSlotScope")}
                </p>
              )}
              {abilityOffer &&
                !preview.native_no_effect &&
                !preview.changed && (
                  <p className="small muted">{t("trainingAbilityUnchanged")}</p>
                )}
              <p>
                {t(
                  abilityOffer
                    ? preview.native_no_effect
                      ? "trainingAbilityRejected"
                      : "trainingAbilityAccepted"
                    : preview.native_no_effect
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
                  {(natureOffer || abilityOffer) && (
                    <tr>
                      <td>{t("trainingNature")}</td>
                      <td>
                        {natureName(
                          preview.before.effective_nature ??
                            preview.before.nature,
                        )}
                      </td>
                      <td>
                        {natureName(
                          preview.after.effective_nature ??
                            preview.after.nature,
                        )}
                      </td>
                    </tr>
                  )}
                  {abilityOffer && (
                    <>
                      <tr>
                        <td>{t("ability")}</td>
                        <td>
                          {abilityName(preview.before.ability_id)} ·{" "}
                          {abilitySlotName(preview.before.ability_slot)}
                        </td>
                        <td>
                          {abilityName(preview.after.ability_id)} ·{" "}
                          {abilitySlotName(preview.after.ability_slot)}
                        </td>
                      </tr>
                      <tr>
                        <td>PID</td>
                        <td>{preview.before.pid}</td>
                        <td>{preview.after.pid}</td>
                      </tr>
                    </>
                  )}
                  {(natureOffer || abilityOffer) &&
                    stats.map((s, i) => (
                      <tr key={`ability-${s}`}>
                        <td>{t(s)}</td>
                        <td>
                          {preview.party_stats_before?.[i] ??
                            preview.before.stats[i]}
                        </td>
                        <td>
                          {preview.party_stats_after?.[i] ??
                            preview.after.stats[i]}
                        </td>
                      </tr>
                    ))}
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
              {abilityOffer &&
                preview.ability_target != null &&
                preview.after.ability_id !== preview.ability_target && (
                  <p className="small muted">
                    {t("trainingAbilityTargetDifference")}{" "}
                    {abilityName(preview.ability_target)}
                  </p>
                )}
              {!natureOffer &&
                !abilityOffer &&
                preview.before.evs.every(
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
                <pre>
                  {JSON.stringify(
                    { offer: abilityOffer ?? natureOffer ?? offer, preview },
                    null,
                    2,
                  )}
                </pre>
              </details>
            </article>
          )}
        </>
      )}
    </section>
  );
}
