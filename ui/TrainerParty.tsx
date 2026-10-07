import { useEffect, useMemo, useState } from "react";
import { api } from "./api";
import { Sprite, Types } from "./components";
import { statKeys, useI18n } from "./i18n";
import { hiddenPower } from "./hiddenPower";
import { speciesDisplayName } from "./speciesDisplay";
import type {
  Catalog,
  Opponent,
  Snapshot,
  TrainerDifficulty,
  TrainerBattlePreview,
  NativeTrainerPreview,
  TrainerEvPreview,
} from "./types";

type BattleMon = {
  species: number;
  held_item: number;
  ability_slot: number;
  nature: number;
  speed: number;
  current_hp: number;
};
export function TrainerParty({
  trainer,
  catalog,
  save,
  difficulty,
  onSpecies,
  onAbility,
}: {
  trainer: Opponent;
  catalog: Catalog;
  save: Snapshot | null;
  difficulty: TrainerDifficulty | null;
  onSpecies: (id: number) => void;
  onAbility: (id: number) => void;
}) {
  const { t } = useI18n();
  const nativeSeed = 0; // Stable sample; random native values are explicitly labelled.
  const [nativePreview, setNativePreview] =
    useState<NativeTrainerPreview | null>(null);
  const [nativeError, setNativeError] = useState("");
  const hasNativeConstructor = !!catalog.profile.native_trainers;
  useEffect(() => {
    setNativePreview(null);
    setNativeError("");
    if (!hasNativeConstructor) return;
    let alive = true;
    const timer = window.setTimeout(() => {
      api<NativeTrainerPreview>("trainer_native_preview", {
        trainer_id: trainer.id,
        seed: nativeSeed,
      })
        .then((result) => {
          if (alive) setNativePreview(result);
        })
        .catch((error) => {
          if (alive)
            setNativeError(
              `${error?.code ?? "error"}: ${error?.detail ?? error}`,
            );
        });
    }, 120);
    return () => {
      alive = false;
      window.clearTimeout(timer);
    };
  }, [hasNativeConstructor, catalog.profile.md5, trainer.id, nativeSeed]);
  const party = save?.pokemon.filter((p) => p.location.kind === "party") ?? [];
  const [battlePreview, setBattlePreview] =
    useState<TrainerBattlePreview | null>(null);
  const [battleError, setBattleError] = useState("");
  const [preview, setPreview] = useState<TrainerEvPreview | null>(null);
  const [previewError, setPreviewError] = useState("");
  const [loading, setLoading] = useState(false);
  const hasTemplate = trainer.party.some(
    (p) => p.generation?.context === "ultimate_template",
  );
  const savedParty = useMemo<BattleMon[]>(
    () =>
      (save?.pokemon ?? [])
        .filter((p) => p.location.kind === "party" && !p.pokemon.egg)
        .sort((a, b) => a.location.slot - b.location.slot)
        .map(({ pokemon: p }) => ({
          species: p.species,
          held_item: p.held_item,
          ability_slot: p.ability_slot,
          nature: p.effective_nature ?? p.nature,
          speed: p.stats[3],
          current_hp: p.current_hp ?? 0,
        })),
    [save],
  );
  const useSave = savedParty.length > 0;
  const saveMaxLevel = useMemo(() => {
    const levels = (save?.pokemon ?? [])
      .filter((p) => p.location.kind === "party" && !p.pokemon.egg)
      .map((p) => p.pokemon.level);
    return levels.length ? Math.max(...levels) : null;
  }, [save]);
  const playerMaxLevel = useSave ? saveMaxLevel : null;
  const needsMaxLevel = trainer.party.some(
    (p) =>
      p.level === 0 ||
      (difficulty === 4 &&
        p.level > 1 &&
        p.level < (catalog.profile.max_level ?? 100)),
  );
  const battleKey = JSON.stringify({
    trainer_id: trainer.id,
    difficulty,
    player_max_level: playerMaxLevel,
  });
  useEffect(() => {
    if (difficulty === null) {
      setBattlePreview(null);
      setBattleError("");
      return;
    }
    let alive = true;
    setBattlePreview(null);
    setBattleError("");
    const timer = window.setTimeout(() => {
      api<TrainerBattlePreview>("trainer_battle_preview", JSON.parse(battleKey))
        .then((result) => {
          if (alive) setBattlePreview(result);
        })
        .catch((error) => {
          if (alive)
            setBattleError(
              `${error?.code ?? "error"}: ${error?.detail ?? error}`,
            );
        });
    }, 120);
    return () => {
      alive = false;
      window.clearTimeout(timer);
    };
  }, [difficulty, battleKey]);
  const currentBattle =
    battlePreview?.trainer_id === trainer.id &&
    battlePreview.difficulty === difficulty &&
    battlePreview.player_max_level === playerMaxLevel
      ? battlePreview
      : null;
  const playerParty = savedParty;
  const scenario =
    playerParty.length > 0 &&
    playerParty.every(
      (p) => p.species > 0 && p.speed > 0 && p.nature >= 0 && p.nature < 25,
    );
  const scenarioKey = JSON.stringify({
    trainer_id: trainer.id,
    difficulty,
    player_party: playerParty,
    opponent_levels: trainer.party.map(
      (p, i) => currentBattle?.mons[i]?.level || p.level,
    ),
  });
  useEffect(() => {
    if (
      difficulty === null ||
      !hasTemplate ||
      !scenario ||
      !currentBattle ||
      trainer.party.some(
        (p, i) =>
          p.generation?.context === "ultimate_template" &&
          !currentBattle.mons[i]?.level,
      )
    ) {
      setPreview(null);
      setLoading(false);
      setPreviewError("");
      return;
    }
    let alive = true;
    setLoading(true);
    setPreview(null);
    setPreviewError("");
    const timer = window.setTimeout(() => {
      api<TrainerEvPreview>("trainer_ev_preview", JSON.parse(scenarioKey))
        .then((result) => {
          if (alive) setPreview(result);
        })
        .catch((error) => {
          if (alive) {
            setPreview(null);
            setPreviewError(
              `${error?.code ?? "error"}: ${error?.detail ?? error}`,
            );
          }
        })
        .finally(() => {
          if (alive) setLoading(false);
        });
    }, 180);
    return () => {
      alive = false;
      window.clearTimeout(timer);
    };
  }, [
    difficulty,
    hasTemplate,
    scenario,
    scenarioKey,
    currentBattle,
    trainer.party,
  ]);
  const highestLevel = party.length
    ? Math.max(...party.map((p) => p.pokemon.level))
    : null;
  return (
    <div className="trainer-party">
      {difficulty !== null && (
        <details className="trainer-mode-note">
          <summary>
            {t("trainerPartyInMode").replace(
              "{mode}",
              t(`trainerDifficulty_${difficulty}`),
            )}
          </summary>
          <p className="small">{t("trainerDifficultySimple")}</p>
          <p className="small muted">
            {hasTemplate
              ? t(useSave ? "trainerAutomaticParty" : "trainerAutomaticNoSave")
              : t("trainerRomOnlyPreview")}
          </p>
          {needsMaxLevel && playerMaxLevel === null && (
            <p className="small muted">{t("trainerMaxLevelNeeded")}</p>
          )}
          {loading && <p role="status">{t("trainerEvCalculating")}</p>}
          {(battleError || previewError) && (
            <p role="alert">{t("trainerPreviewError")}</p>
          )}
        </details>
      )}
      {hasNativeConstructor && (
        <details className="trainer-mode-note">
          <summary>{t("nativeTrainerTitle")}</summary>
          <p className="small">{t("nativeTrainerScope")}</p>
          <p className="small muted">{t("nativeTrainerSample")}</p>
          {nativeError && <p role="alert">{t("trainerPreviewError")}</p>}
          {!nativePreview && !nativeError && <p>{t("trainerEvCalculating")}</p>}
        </details>
      )}
      <table className="trainer-party-table">
        <colgroup>
          <col className="trainer-pokemon-column" />
          <col className="trainer-loadout-column" />
          <col />
        </colgroup>
        <thead>
          <tr>
            <th scope="col">{t("species")}</th>
            <th scope="col">{t("trainerLoadout")}</th>
            <th scope="col">{t("moves")}</th>
          </tr>
        </thead>
        {trainer.party.map((p, i) => {
          const native =
            nativePreview?.rom_md5 === catalog.profile.md5 &&
            nativePreview.trainer_id === trainer.id &&
            nativePreview.seed === nativeSeed
              ? nativePreview.mons[i]
              : null;
          const renderedSpecies = native?.species ?? p.species;
          const species = catalog.species.find((s) => s.id === renderedSpecies);
          const g = native
            ? {
                context: "native_scenario",
                gender: native.gender,
                nature: native.effective_nature ?? native.nature,
                ability_id: native.ability_id,
                ability_options: [native.ability_id],
                ivs: native.ivs,
                evs: native.evs,
                personality_parameter: 0,
                ev_increment: null,
              }
            : p.generation;
          const template = g?.context === "ultimate_template";
          const simulated =
            template &&
            preview?.trainer_id === trainer.id &&
            preview.difficulty === difficulty
              ? preview.mons[i]
              : null;
          const battleMon = currentBattle?.mons[i];
          const resolvedIvs = template
            ? simulated?.ivs
            : (battleMon?.ivs ?? g?.ivs);
          const resolvedEvs = template
            ? simulated?.evs
            : (battleMon?.evs ?? g?.evs);
          const hpRules = catalog.profile.hidden_power;
          const hp = hiddenPower(hpRules, resolvedIvs);
          const hpAlternate = hiddenPower(hpRules, simulated?.alternate_ivs);
          const dynamic = p.level_rule === "party_max";
          const level = native
            ? native.level
            : battleMon
              ? battleMon.level
              : dynamic
                ? highestLevel
                : p.level;
          const moveIds = native?.moves ?? battleMon?.moves ?? p.moves;
          const heldItem = native?.held_item ?? p.held_item;
          const gender = g?.gender;
          const abilities = [
            ...new Set(g?.ability_options ?? (g ? [g.ability_id] : [])),
          ];
          return (
            <tbody
              className="trainer-party-entry"
              key={i}
              data-species={renderedSpecies}
            >
              <tr>
                <td>
                  <div className="trainer-mon-heading">
                    <Sprite
                      catalog={catalog}
                      species={renderedSpecies}
                      pid={native?.pid}
                    />
                    <div>
                      <div className="trainer-mon-title">
                        <button
                          className="link-button"
                          onClick={() => onSpecies(renderedSpecies)}
                        >
                          {speciesDisplayName(catalog, renderedSpecies)}
                        </button>
                        <strong>
                          {battleMon && level === null
                            ? t(
                                battleMon.level_source === "needs_player_max"
                                  ? "trainerMaxLevelNeeded"
                                  : "dynamicLevel",
                              )
                            : level === null
                              ? t("dynamicLevel")
                              : p.level_rule === "difficulty"
                                ? battleMon
                                  ? `Lv. ${level}`
                                  : `${t("trainerBaseLevel")} ${level}`
                                : `Lv. ${level}`}
                        </strong>
                      </div>
                      <span className={`gender-badge ${gender ?? ""}`}>
                        {t("gender")} · {gender ? t(gender) : t("unresolved")}
                      </span>
                      {species && (
                        <Types catalog={catalog} values={species.types} />
                      )}
                      {p.level_rule === "difficulty" && (
                        <p className="small muted">
                          {battleMon
                            ? t(`trainerLevelSource_${battleMon.level_source}`)
                            : t("ultimateTrainerLevel")}
                        </p>
                      )}
                      {dynamic && (
                        <div className="small muted">
                          {t(
                            highestLevel === null
                              ? "partyMaxNoSave"
                              : "partyMaxWithSave",
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                </td>
                <td>
                  <dl className="trainer-mon-facts">
                    <div>
                      <dt>{t("nature")}</dt>
                      <dd>
                        {(g && catalog.natures[g.nature]?.name) ||
                          t("unresolved")}
                      </dd>
                    </div>
                    <div>
                      <dt>{t("ability")}</dt>
                      <dd>
                        {g
                          ? abilities.map((id) => (
                              <button
                                key={id}
                                className="link-button"
                                onClick={() => onAbility(id)}
                                title={catalog.abilities[id]?.description}
                              >
                                {catalog.abilities[id]?.name ?? `#${id}`} ↗
                              </button>
                            ))
                          : t("unresolved")}
                        {g && (
                          <span className="small muted">
                            {t(
                              native
                                ? "nativeTrainerScenarioValue"
                                : abilities.length > 1
                                  ? "abilityChoice"
                                  : "abilityFixed",
                            )}
                          </span>
                        )}
                      </dd>
                    </div>
                    <div>
                      <dt>{t("held_item")}</dt>
                      <dd>
                        {heldItem
                          ? (catalog.items[heldItem]?.name ?? heldItem)
                          : t("noHeldItem")}
                      </dd>
                    </div>
                  </dl>
                </td>
                <td className="trainer-moves">
                  <span className="small muted">
                    {t(p.moves_explicit ? "explicitMoves" : "levelSource")}
                  </span>
                  <div>
                    {((dynamic && !native) ||
                      (battleMon && battleMon.moves === null)) &&
                    !p.moves_explicit
                      ? t("dynamicMoves")
                      : moveIds.filter(Boolean).map((id) => (
                          <span key={id}>
                            {catalog.moves[id]?.name ?? id}
                            {id === hpRules?.move_id && (
                              <>
                                {" "}
                                ·{" "}
                                {hp
                                  ? `${catalog.type_names[hp.type]} · ${t("power")} ${hp.power}${hpAlternate && (hpAlternate.type !== hp.type || hpAlternate.power !== hp.power) ? ` / ${catalog.type_names[hpAlternate.type]} · ${t("power")} ${hpAlternate.power}` : ""}`
                                  : t("hiddenPowerUnknown")}
                              </>
                            )}
                          </span>
                        ))}
                  </div>
                </td>
              </tr>
              <tr className="trainer-values-row">
                <td colSpan={3}>
                  <table className="trainer-stat-table">
                    <thead>
                      <tr>
                        <th scope="col" aria-label={t("stat")}></th>
                        {statKeys.map((key) => (
                          <th key={key} scope="col">
                            {t(key)}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      <tr>
                        <th scope="row" title={t("ivs")}>
                          IV
                        </th>
                        {statKeys.map((key, s) => (
                          <td key={key}>
                            {resolvedIvs?.[s] ?? "?"}
                            {simulated?.alternate_ivs &&
                            simulated.alternate_ivs[s] !== simulated.ivs?.[s]
                              ? ` / ${simulated.alternate_ivs[s]}`
                              : ""}
                          </td>
                        ))}
                      </tr>
                      <tr>
                        <th scope="row" title={t("evs")}>
                          EV
                        </th>
                        {statKeys.map((key, s) => (
                          <td key={key}>
                            {resolvedEvs?.[s] ?? "?"}
                            {simulated?.alternate_evs &&
                            simulated.alternate_evs[s] !== simulated.evs?.[s]
                              ? ` / ${simulated.alternate_evs[s]}`
                              : ""}
                          </td>
                        ))}
                      </tr>
                    </tbody>
                  </table>
                  {resolvedEvs && (
                    <p className="small muted">
                      {t("trainerEvTotal")}:{" "}
                      {resolvedEvs.reduce((a, b) => a + b, 0)}
                      {simulated?.alternate_evs &&
                        simulated.alternate_evs.reduce((a, b) => a + b, 0) !==
                          resolvedEvs.reduce((a, b) => a + b, 0) &&
                        ` / ${simulated.alternate_evs.reduce((a, b) => a + b, 0)}`}
                      {(simulated?.alternate_evs || simulated?.alternate_ivs) &&
                        ` · ${t("trainerEvAlternate")}`}
                    </p>
                  )}
                  {g && !resolvedIvs && !template && (
                    <p className="small muted">{t("randomIVs")}</p>
                  )}
                </td>
              </tr>
            </tbody>
          );
        })}
      </table>
    </div>
  );
}
