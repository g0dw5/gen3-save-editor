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
} from "./types";

type Gender = "male" | "female" | "genderless";
type Choice = { source: string; species: number; gender: Gender; item: number };
interface Preview {
  rom_md5: string;
  parents: Pokemon[];
  compatibility: number;
  child: Pokemon | null;
  seed: number;
  offspring_pid: number;
  rng_after: number;
  partial: boolean;
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
  const [result, setResult] = useState<Preview | null>(null);
  const [busy, setBusy] = useState(false);
  const [services, setServices] = useState<AcquisitionSource[] | null>(null);
  const revision = useRef(0);
  useEffect(() => {
    revision.current++;
    setParents(defaults());
    setResult(null);
    setBusy(false);
    setSeed(42);
    setPid(24);
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
        parents: parents.map((p, i) =>
          p.source === "simulated"
            ? {
                kind: "simulated",
                species: p.species,
                gender: p.gender,
                held_item: p.item,
                trainer_id: i + 1,
              }
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
              <details>
                <summary>{t("breedEvidence")}</summary>
                <pre>{JSON.stringify(result, null, 2)}</pre>
              </details>
            </>
          )}
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
