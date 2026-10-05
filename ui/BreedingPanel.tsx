import { useEffect, useRef, useState } from "react";
import { api } from "./api";
import { useI18n } from "./i18n";
import { SearchSelect } from "./SearchSelect";
import { ConditionDetails } from "./ConditionDetails";
import { fromKey, locationKey } from "./types";
import type {
  AcquisitionSource,
  Catalog,
  GameMap,
  MapFocus,
  Pokemon,
  QueryTarget,
  Snapshot,
  SavedDaycareState,
} from "./types";

type Gender = "male" | "female" | "genderless";
type Choice = { source: string; species: number; gender: Gender; item: number };
export interface Preview {
  rom_md5: string;
  parents: Pokemon[];
  compatibility: number;
  child: Pokemon | null;
  seed: number;
  offspring_pid: number;
  rng_after: number;
  partial: boolean;
  production?: {
    bag_context: string;
    modifier_item: number | null;
    modifier_present: boolean | null;
    interval_steps: number;
    numerator: number;
    denominator: number;
    percent: number;
  } | null;
}
function genders(ratio: number): Gender[] {
  return ratio === 255
    ? ["genderless"]
    : ratio === 0
      ? ["male"]
      : ratio === 254
        ? ["female"]
        : ["female", "male"];
}
/** Executes a read-only scenario; never creates an individual in the save. */
export function BreedingPanel({
  catalog,
  save,
  species,
  maps,
  onTarget,
  onMap,
  onError,
}: {
  catalog: Catalog;
  save: Snapshot | null;
  species: number;
  maps: GameMap[];
  onTarget: (target: QueryTarget) => void;
  onMap: (id: string, focus?: MapFocus) => void;
  onError: (error: unknown) => void;
}) {
  const { t } = useI18n();
  const defaults = (): Choice[] =>
    [0, 1].map((i) => {
      const mon = catalog.species.find((s) => s.id === species)!;
      const options = genders(mon.gender_ratio);
      return {
        source: "simulated",
        species,
        gender: options[Math.min(i, options.length - 1)],
        item: 0,
      };
    });
  const [parents, setParents] = useState(defaults);
  const [open, setOpen] = useState(false);
  const [seed, setSeed] = useState(42);
  const [pid, setPid] = useState(24);
  const [productionItem, setProductionItem] = useState("current");
  const [result, setResult] = useState<Preview | null>(null);
  const [busy, setBusy] = useState(false);
  const [services, setServices] = useState<AcquisitionSource[] | null>(null);
  const [savedState, setSavedState] = useState<SavedDaycareState | null>(null);
  const [stateLoading, setStateLoading] = useState(false);
  const revision = useRef(0);
  useEffect(() => {
    revision.current++;
    setParents(defaults());
    setResult(null);
    setBusy(false);
    setSeed(42);
    setPid(24);
    setProductionItem("current");
    return () => {
      revision.current++;
    };
  }, [catalog.profile.md5, species, save]);
  useEffect(() => {
    if (!open) return;
    let active = true;
    setServices(null);
    api<AcquisitionSource[]>("daycare_sources")
      .then((s) => {
        if (active) setServices(s);
      })
      .catch((e) => {
        if (active) onError(e);
      });
    return () => {
      active = false;
    };
  }, [open, catalog.profile.md5, save, onError]);
  useEffect(() => {
    let active = true;
    setSavedState(null);
    setStateLoading(false);
    if (open && save && catalog.profile.breeding?.saved) {
      setStateLoading(true);
      api<SavedDaycareState | null>("daycare_state")
        .then((s) => {
          if (active && s?.rom_md5 === catalog.profile.md5) setSavedState(s);
        })
        .catch((e) => {
          if (active) onError(e);
        })
        .finally(() => {
          if (active) setStateLoading(false);
        });
    }
    return () => {
      active = false;
    };
  }, [open, catalog.profile.md5, save, onError]);
  const current =
    savedState?.rom_md5 === catalog.profile.md5 ? savedState : null;
  const deposited = current?.parents.filter((p) => p.pokemon && !p.issue) ?? [];
  const stored = (save?.pokemon ?? []).filter(
    (m) => m.pokemon.species > 0 && !m.pokemon.egg,
  );
  const invalidate = () => {
    revision.current++;
    setResult(null);
    setBusy(false);
  };
  const update = (i: number, patch: Partial<Choice>) => {
    invalidate();
    setParents((p) => p.map((v, j) => (j === i ? { ...v, ...patch } : v)));
  };
  const calculate = async () => {
    const ticket = ++revision.current;
    setBusy(true);
    setResult(null);
    try {
      const value = await api<Preview>("breeding_preview", {
        production_item:
          productionItem === "current" ? undefined : productionItem === "yes",
        parents: parents.map((p, i) =>
          p.source === "simulated"
            ? {
                kind: "simulated",
                species: p.species,
                gender: p.gender,
                held_item: p.item,
                trainer_id: i + 1,
              }
            : p.source.startsWith("deposited:")
              ? { kind: "deposited", slot: Number(p.source.split(":")[1]) }
              : { kind: "stored", location: fromKey(p.source) },
        ),
        seed,
        offspring_pid: pid,
      });
      if (revision.current === ticket && value.rom_md5 === catalog.profile.md5)
        setResult(value);
    } catch (e) {
      if (revision.current === ticket) onError(e);
    } finally {
      if (revision.current === ticket) setBusy(false);
    }
  };
  const speciesName = (id: number) =>
    catalog.species.find((s) => s.id === id)?.name ?? `#${id}`;
  return (
    <details
      className="breeding-panel"
      onToggle={(e) => setOpen(e.currentTarget.open)}
    >
      <summary>{t("breedTitle")}</summary>
      <p className="small muted">{t("breedHelp")}</p>
      {save && catalog.profile.breeding?.saved && (
        <section className="daycare-state small">
          <h4>{t("daycareSavedTitle")}</h4>
          <p className="muted">{t("daycareSavedHelp")}</p>
          {stateLoading ? (
            <p>{t("loading")}</p>
          ) : current ? (
            <>
              <p>{t(`daycareSaved_${current.status}`)}</p>
              {current.parents.map((parent) => (
                <p key={parent.slot}>
                  {t("breedParent")} {parent.slot + 1}:{" "}
                  {parent.pokemon ? (
                    <>
                      <button
                        className="link-button"
                        onClick={() =>
                          onTarget({
                            kind: "species",
                            id: parent.pokemon!.species,
                          })
                        }
                      >
                        {speciesName(parent.pokemon.species)} ↗
                      </button>{" "}
                      · {t("daycareStoredLevel")} {parent.pokemon.level} ·{" "}
                      {t("daycareAccumulatedSteps")} {parent.accumulated_steps}
                    </>
                  ) : (
                    t(
                      parent.present
                        ? "daycareSavedInvalidParent"
                        : "daycareSavedEmptySlot",
                    )
                  )}
                </p>
              ))}
              {current.next_check_steps != null && (
                <p>
                  {t("daycareNextCheck").replace(
                    "{steps}",
                    String(current.next_check_steps),
                  )}
                </p>
              )}
              {current.compatibility === 0 && (
                <p className="muted">{t("daycareSavedIncompatible")}</p>
              )}
              {deposited.length === 2 && (
                <button
                  onClick={() => {
                    invalidate();
                    setParents(
                      deposited.map((p) => ({
                        source: `deposited:${p.slot}`,
                        species: p.pokemon!.species,
                        gender: "female",
                        item: 0,
                      })),
                    );
                  }}
                >
                  {t("daycareUseDeposited")}
                </button>
              )}
              <details>
                <summary>{t("evidence")}</summary>
                <pre>{JSON.stringify(current, null, 2)}</pre>
              </details>
            </>
          ) : (
            <p>{t("daycareSavedUnknown")}</p>
          )}
        </section>
      )}
      <div className="breeding-parents">
        {parents.map((p, i) => (
          <fieldset key={i}>
            <legend>
              {t("breedParent")} {i + 1}
            </legend>
            <SearchSelect
              label={t("breedChooseParent")}
              value={p.source}
              onChange={(source) => update(i, { source })}
              options={[
                { value: "simulated", label: t("breedSimulated") },
                ...deposited.map((p) => ({
                  value: `deposited:${p.slot}`,
                  label: `${t("daycareDeposited")} ${p.slot + 1} · ${speciesName(p.pokemon!.species)}`,
                })),
                ...stored.map((m) => ({
                  value: locationKey(m.location),
                  label: `${m.location.kind === "party" ? t("party") : `${t("box")} ${m.location.box_index + 1}`} · ${m.location.slot + 1} · ${m.pokemon.nickname || speciesName(m.pokemon.species)}`,
                })),
              ]}
            />
            {p.source === "simulated" ? (
              <>
                <SearchSelect
                  label={t("species")}
                  value={p.species}
                  options={catalog.species
                    .filter((s) => s.id > 0)
                    .map((s) => ({ value: s.id, label: s.name }))}
                  onChange={(v) => {
                    const id = Number(v);
                    const mon = catalog.species.find((s) => s.id === id)!;
                    update(i, {
                      species: id,
                      gender: genders(mon.gender_ratio)[0],
                    });
                  }}
                />
                <label className="field">
                  <span>{t("gender")}</span>
                  <select
                    value={p.gender}
                    onChange={(e) =>
                      update(i, { gender: e.target.value as Gender })
                    }
                  >
                    {genders(
                      catalog.species.find((s) => s.id === p.species)!
                        .gender_ratio,
                    ).map((g) => (
                      <option value={g} key={g}>
                        {t(g)}
                      </option>
                    ))}
                  </select>
                </label>
                <SearchSelect
                  label={t("held_item")}
                  value={p.item}
                  onChange={(v) => update(i, { item: Number(v) })}
                  options={catalog.items.map((item) => ({
                    value: item.id,
                    label: item.id === 0 ? t("emptyMove") : item.name,
                  }))}
                />
              </>
            ) : (
              <p className="small">{t("breedStoredHelp")}</p>
            )}
          </fieldset>
        ))}
      </div>
      <details>
        <summary>{t("breedScenario")}</summary>
        <p className="small muted">{t("breedScenarioHelp")}</p>
        <label className="field">
          <span>{t("breedSeed")}</span>
          <input
            type="number"
            min={0}
            max={4294967295}
            step={1}
            value={seed}
            onChange={(e) => {
              invalidate();
              setSeed(Number(e.target.value));
            }}
          />
        </label>
        <label className="field">
          <span>{t("breedPid")}</span>
          <input
            type="number"
            min={1}
            max={
              catalog.profile.breeding?.pending_width === 2 ? 65535 : 4294967295
            }
            step={1}
            value={pid}
            onChange={(e) => {
              invalidate();
              setPid(Number(e.target.value));
            }}
          />
        </label>
      </details>
      {catalog.profile.breeding?.production?.modifier != null && (
        <label className="field">
          <span>{t("breedProductionItem")}</span>
          <select
            aria-label={t("breedProductionItem")}
            value={productionItem}
            onChange={(e) => {
              invalidate();
              setProductionItem(e.target.value);
            }}
          >
            <option value="current">
              {t(save ? "breedProductionSaved" : "breedProductionEmpty")}
            </option>
            <option value="yes">{t("breedProductionWith")}</option>
            <option value="no">{t("breedProductionWithout")}</option>
          </select>
        </label>
      )}
      <button
        disabled={
          busy ||
          !Number.isInteger(seed) ||
          seed < 0 ||
          seed > 4294967295 ||
          !Number.isInteger(pid) ||
          pid < 1 ||
          pid >
            (catalog.profile.breeding?.pending_width === 2 ? 65535 : 4294967295)
        }
        onClick={calculate}
      >
        {t(busy ? "loading" : "breedCalculate")}
      </button>
      {result && (
        <section className="breeding-result" aria-live="polite">
          <p>
            {t(
              result.compatibility === 0
                ? "breedIncompatible"
                : "breedCompatible",
            )}
          </p>
          <p className="small muted">{t("breedNotRate")}</p>
          {result.production && (
            <article className="encounter-card">
              <strong>{t("breedProductionTitle")}</strong>
              <p>
                {t("breedProductionChance")} ·{" "}
                {result.production.percent.toFixed(2)}%{" · "}
                {t("breedProductionInterval")} ·{" "}
                {result.production.interval_steps} {t("breedProductionSteps")}
              </p>
              <p className="small muted">{t("breedProductionScope")}</p>
              <p className="small">
                {t(
                  result.production.bag_context === "ordinary_save_projection"
                    ? "breedProductionSaved"
                    : result.production.bag_context === "empty_bag_simulation"
                      ? "breedProductionEmpty"
                      : "breedProductionOverride",
                )}
              </p>
              {result.production.modifier_item !== null && (
                <p>
                  <button
                    className="link-button"
                    onClick={() =>
                      onTarget({
                        kind: "item",
                        id: result.production!.modifier_item!,
                      })
                    }
                  >
                    {catalog.items.find(
                      (i) => i.id === result.production!.modifier_item,
                    )?.name ?? "?"}{" "}
                    ↗
                  </button>
                  {" · "}
                  {t(
                    result.production.modifier_present
                      ? "breedProductionPresent"
                      : "breedProductionMissing",
                  )}
                </p>
              )}
              {result.production.modifier_item === null && (
                <p className="small muted">{t("breedProductionNoModifier")}</p>
              )}
              {result.compatibility === 0 &&
                result.production.numerator > 0 && (
                  <p className="small">{t("breedProductionIncompatible")}</p>
                )}
            </article>
          )}
          {result.child && (
            <>
              <p>
                <strong>{t("breedChild")} · </strong>
                <button
                  className="link-button"
                  onClick={() =>
                    onTarget({ kind: "species", id: result.child!.species })
                  }
                >
                  {speciesName(result.child.species)} ↗
                </button>{" "}
                · Lv. {result.child.level} · {t(result.child.gender)}
              </p>
              <p>
                {t(
                  result.child.species === species
                    ? "breedMatches"
                    : "breedDifferent",
                )}
              </p>
              <p>
                {t("nature")} ·{" "}
                {catalog.natures.find(
                  (n) =>
                    n.id ===
                    (result.child!.effective_nature ?? result.child!.nature),
                )?.name ?? "?"}{" "}
                · {t("ability")} ·{" "}
                {catalog.abilities.find(
                  (a) => a.id === result.child!.ability_id,
                )?.name ?? "?"}
              </p>
              <p>
                {t("moves")} ·{" "}
                {result.child.moves.map((m, slot) => (
                  <button
                    className="link-button"
                    key={slot}
                    disabled={m === 0}
                    onClick={() => onTarget({ kind: "move", id: m })}
                  >
                    {catalog.moves.find((v) => v.id === m)?.name ??
                      t("emptyMove")}{" "}
                  </button>
                ))}
              </p>
              <table className="compact-table">
                <thead>
                  <tr>
                    <th>{t("stat")}</th>
                    <th>{t("ivs")}</th>
                  </tr>
                </thead>
                <tbody>
                  {[
                    "hp",
                    "attack",
                    "defense",
                    "speed",
                    "sp_attack",
                    "sp_defense",
                  ].map((key, i) => (
                    <tr key={key}>
                      <th>{t(key)}</th>
                      <td>{result.child!.ivs[i]}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
          <details>
            <summary>{t("breedEvidence")}</summary>
            <pre>{JSON.stringify(result, null, 2)}</pre>
          </details>
        </section>
      )}
      <h4>{t("breedServices")}</h4>
      <p className="small muted">{t("breedServicesHelp")}</p>
      {!services ? (
        <p>{t("loading")}</p>
      ) : !services.length ? (
        <p>{t("breedNoService")}</p>
      ) : (
        services.map((s, i) => (
          <article
            className="encounter-card"
            key={`${s.map_id}-${s.offset}-${i}`}
          >
            <button
              className="link-button"
              onClick={() =>
                onMap(
                  s.map_id!,
                  s.x !== null && s.y !== null ? { x: s.x, y: s.y } : undefined,
                )
              }
            >
              {maps.find((m) => m.id === s.map_id)?.name ?? s.map_id} ·{" "}
              {s.x === null ? t("acqNoTile") : `(${s.x}, ${s.y})`} ↗
            </button>
            <ConditionDetails
              checks={s.conditions}
              catalog={catalog}
              onTarget={onTarget}
            />
          </article>
        ))
      )}
    </details>
  );
}
