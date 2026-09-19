import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Search, ArrowUpRight, FileCode2 } from "lucide-react";
import { api, download, fromBase64, native, outputPath } from "./api";
import {
  Floating,
  NumberField,
  SelectField,
  Sprite,
  Types,
} from "./components";
import { MapExplorer } from "./MapExplorer";
import { TrainerArt } from "./TrainerArt";
import { TrainerParty } from "./TrainerParty";
import {
  emptyTrainerFilters,
  indexTrainers,
  searchTrainers,
  trainerFacetOptions,
  type TrainerFacet,
  type TrainerFilters,
} from "./trainerSearch";
import { useI18n, statKeys } from "./i18n";
import type {
  Ability,
  Catalog,
  Encounter,
  GameMap,
  Item,
  Move,
  Opponent,
  RefTab,
  RefWindow,
  SpeciesDetail,
  Template,
  World,
  Snapshot,
  FishingReport,
} from "./types";

interface Props {
  window: RefWindow;
  catalog: Catalog;
  world: World | null;
  save: Snapshot | null;
  loadWorld: () => void;
  onClose: () => void;
  onTemplate: (template: Template) => void;
  onError: (error: unknown) => void;
}

// Gen III item identities; display names come from the loaded ROM.
const fishingRodItems: Record<string, number> = {
  old_rod: 262,
  good_rod: 263,
  super_rod: 264,
};

export function ReferenceWindow({
  window: info,
  catalog,
  world,
  save,
  loadWorld,
  onClose,
  onTemplate,
  onError,
}: Props) {
  const { t } = useI18n();
  const encounterMethod = (method: string) =>
    catalog.items.find((item) => item.id === fishingRodItems[method])?.name ||
    t(method);
  const [tab, setTab] = useState<RefTab>(info.tab);
  const [selected, setSelected] = useState<number | string>(info.selected ?? 1);
  const [search, setSearch] = useState("");
  const [trainerFilters, setTrainerFilters] =
    useState<TrainerFilters>(emptyTrainerFilters);
  const trainerEntries = useMemo(
    () => indexTrainers(world, catalog, t),
    [world, catalog, t],
  );
  const trainerEntryById = useMemo(
    () => new Map(trainerEntries.map((entry) => [entry.trainer.id, entry])),
    [trainerEntries],
  );
  const [detail, setDetail] = useState<SpeciesDetail | null>(null);
  const [mapImage, setMapImage] = useState("");
  const [fishingState, setFishingState] = useState<{
    save: Snapshot | null;
    md5: string;
    report: FishingReport | null;
  } | null>(null);
  const fishing =
    fishingState?.save === save && fishingState?.md5 === catalog.profile.md5
      ? fishingState.report
      : null;
  useEffect(() => {
    let active = true;
    setFishingState(null);
    if (catalog.profile.feebas)
      api<FishingReport | null>("fishing_spots")
        .then((report) => {
          if (active)
            setFishingState({ save, md5: catalog.profile.md5, report });
        })
        .catch((error) => {
          if (active) onError(error);
        });
    return () => {
      active = false;
    };
  }, [save, catalog.profile.md5, catalog.profile.feebas, onError]);
  const [romEdit, setRomEdit] = useState(false);
  const detailPane = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    detailPane.current?.scrollTo({ top: 0 });
  }, [tab, selected]);
  useEffect(() => {
    if (tab === "maps" || tab === "trainers") loadWorld();
  }, [tab, loadWorld]);
  const rows = useMemo(
    () =>
      (tab === "maps"
        ? (world?.maps ?? [])
        : tab === "trainers"
          ? (world?.trainers ?? [])
          : catalog[tab]
      ).filter((row) => row.id !== 0),
    [tab, world, catalog],
  );
  useEffect(() => {
    if (rows.length && !rows.some((r) => r.id === selected))
      setSelected(rows[0].id);
  }, [rows, selected]);
  const current = rows.find((row) => row.id === selected);
  // A tab change can briefly retain a map ID before selection is reset.
  const speciesId =
    tab === "species"
      ? catalog.species.find((species) => species.id === selected)?.id
      : undefined;
  const filtered =
    tab === "trainers"
      ? searchTrainers(trainerEntries, search, trainerFilters).map(
          (entry) => entry.trainer,
        )
      : rows.filter((row) =>
          `${row.id} ${row.name}`
            .toLocaleLowerCase()
            .includes(search.toLocaleLowerCase()),
        );
  useLayoutEffect(() => {
    if (
      tab === "trainers" &&
      filtered.length &&
      !filtered.some((r) => r.id === selected)
    )
      setSelected(filtered[0].id);
  }, [tab, search, trainerFilters, world, selected]);
  useEffect(() => {
    let active = true;
    setDetail(null);
    if (speciesId !== undefined)
      api<SpeciesDetail>("species", { id: speciesId })
        .then((d) => {
          if (active) setDetail(d);
        })
        .catch((error) => {
          if (active) onError(error);
        });
    return () => {
      active = false;
    };
  }, [speciesId, onError]);
  useEffect(() => {
    let active = true;
    setMapImage("");
    if (tab === "maps" && typeof selected === "string")
      api<{ url: string }>("map_image", { id: selected })
        .then((r) => {
          if (active) setMapImage(r.url);
        })
        .catch(onError);
    return () => {
      active = false;
    };
  }, [tab, selected, onError]);
  const goSpecies = (id: number) => {
    setTab("species");
    setSelected(id);
    setSearch("");
  };
  const template = (enc?: Encounter): Template => ({
    species: +selected,
    level: enc?.min_level ?? 5,
    ...(enc ? { met_location: enc.region, egg: enc.method === "egg" } : {}),
  });
  const drag = (e: React.DragEvent, value: Template) => {
    e.dataTransfer.setData(
      "application/x-gen3",
      JSON.stringify({
        kind: "template",
        profile: catalog.profile.md5,
        template: value,
      }),
    );
    e.dataTransfer.effectAllowed = "copy";
  };
  const renderTrainerFilter = (facet: TrainerFacet) => (
    <label key={facet}>
      <span>{t(`trainerFacet_${facet}`)}</span>
      <select
        aria-label={t(`trainerFacet_${facet}`)}
        value={trainerFilters[facet]}
        onChange={(e) =>
          setTrainerFilters((filters) => ({
            ...filters,
            [facet]: e.target.value,
          }))
        }
      >
        <option value="">{t("trainerFilterAll")}</option>
        {trainerFacetOptions(trainerEntries, search, trainerFilters, facet).map(
          (option) => (
            <option
              key={option.value}
              value={option.value}
              disabled={!option.count && trainerFilters[facet] !== option.value}
            >
              {option.label} · {option.count}
            </option>
          ),
        )}
      </select>
    </label>
  );
  return (
    <Floating title={t("references")} onClose={onClose} initial={info.id} wide>
      <div className="reference-tabs">
        {(
          [
            "species",
            "moves",
            "items",
            "abilities",
            "maps",
            "trainers",
          ] as RefTab[]
        )
          .filter(
            (key) =>
              catalog.profile.capabilities?.world !== false ||
              !["maps", "trainers"].includes(key),
          )
          .map((key) => (
            <button
              type="button"
              key={key}
              className={tab === key ? "active" : ""}
              onClick={() => {
                setTab(key);
                setSearch("");
                setRomEdit(false);
              }}
            >
              {t(key)}
            </button>
          ))}
      </div>
      <div className="reference-layout">
        <div className="reference-list">
          <label className="search-field">
            <Search size={16} />
            <input
              aria-label={t(
                tab === "trainers" ? "trainerSearchHint" : "search",
              )}
              placeholder={t(
                tab === "trainers" ? "trainerSearchHint" : "search",
              )}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </label>
          {tab === "trainers" && (
            <div className="trainer-filters">
              {(["role", "location"] as TrainerFacet[]).map(
                renderTrainerFilter,
              )}
              <details className="trainer-extra-filters">
                <summary>
                  {t("trainerMoreFilters")}
                  {trainerFilters.battle || trainerFilters.level ? " •" : ""}
                </summary>
                {(["battle", "level"] as TrainerFacet[]).map(
                  renderTrainerFilter,
                )}
              </details>
              <div className="trainer-filter-status">
                <span>
                  {filtered.length} {t("trainerResults")}
                </span>
                {(search || Object.values(trainerFilters).some(Boolean)) && (
                  <button
                    className="link-button"
                    onClick={() => {
                      setTrainerFilters(emptyTrainerFilters);
                      setSearch("");
                    }}
                  >
                    {t("trainerResetFilters")}
                  </button>
                )}
              </div>
            </div>
          )}
          <div className="reference-rows">
            {filtered.map((row) => (
              <button
                type="button"
                key={row.id}
                className={selected === row.id ? "selected" : ""}
                onClick={() => {
                  setSelected(row.id);
                  setRomEdit(false);
                }}
              >
                <span className="id">{row.id}</span>
                <span>
                  {row.name}
                  {tab === "trainers" && (
                    <small className="trainer-row-context">
                      <span className="trainer-row-tags">
                        {trainerEntryById
                          .get(+row.id)
                          ?.tags.filter(
                            (tag) =>
                              tag.facet === "role" || tag.facet === "location",
                          )
                          .map((tag) => (
                            <span
                              className={`trainer-tag ${tag.facet}`}
                              key={`${tag.facet}:${tag.value}`}
                            >
                              {tag.label}
                            </span>
                          ))}
                      </span>
                      <br />
                      {(row as Opponent).party.some(
                        (p) => p.level_rule === "party_max",
                      )
                        ? t("dynamicLevel")
                        : `Lv. ${Math.min(...(row as Opponent).party.map((p) => p.level))}–${Math.max(...(row as Opponent).party.map((p) => p.level))}`}
                    </small>
                  )}
                </span>
              </button>
            ))}
            {!filtered.length && (
              <p className="muted">
                {t(
                  (tab === "maps" || tab === "trainers") && !world
                    ? "loading"
                    : "noResults",
                )}
              </p>
            )}
          </div>
        </div>
        <div className="reference-detail" ref={detailPane}>
          <div className="reference-label">
            <span className="eyebrow">
              {t("readOnly")} · {catalog.profile.label}
            </span>
            {catalog.profile.capabilities?.rom_edit !== false &&
              ["species", "moves", "items"].includes(tab) && (
                <button
                  className="icon-button"
                  title={t("romEdit")}
                  aria-label={t("romEdit")}
                  onClick={() => setRomEdit((v) => !v)}
                >
                  <FileCode2 size={17} />
                </button>
              )}
          </div>
          {current && (tab !== "trainers" || filtered.length > 0) && (
            <h2>
              {current.name} <small>#{current.id}</small>
            </h2>
          )}
          {romEdit && (
            <RomPatch
              catalog={catalog}
              table={tab}
              id={+selected}
              onError={onError}
            />
          )}
          {tab === "species" &&
            (detail && detail.species.id === speciesId ? (
              <>
                <div
                  className="dex-hero"
                  draggable={catalog.profile.capabilities?.save_edit !== false}
                  onDragStart={(e) => drag(e, template())}
                >
                  <Sprite catalog={catalog} species={+selected} large />
                  <div>
                    <Types values={detail.species.types} />
                    <p className="muted small">
                      {t(
                        catalog.profile.capabilities?.save_edit === false
                          ? "representativeSprite"
                          : "dragTemplate",
                      )}
                    </p>
                    <button
                      type="button"
                      className="link-button"
                      disabled={
                        catalog.profile.capabilities?.save_edit === false
                      }
                      onClick={() => onTemplate(template())}
                    >
                      {t("pickDestination")}
                      <ArrowUpRight size={14} />
                    </button>
                  </div>
                </div>
                <div className="base-stats">
                  {statKeys.map((key, i) => (
                    <div key={key}>
                      <span>{t(key)}</span>
                      <strong>{detail.species.stats[i]}</strong>
                      <div className="stat-track">
                        <div
                          style={{
                            width: `${(detail.species.stats[i] / 255) * 100}%`,
                          }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
                <div className="detail-pairs">
                  <span>{t("ability")}</span>
                  <span>
                    {[...new Set(detail.species.abilities.filter(Boolean))]
                      .map((id) => catalog.abilities[id]?.name ?? id)
                      .join(" / ")}
                  </span>
                </div>
                {fishing?.species === speciesId && (
                  <div className="fishing-info">
                    <strong>{t("fishingSpots")}</strong>
                    <p className="small muted">
                      {t(
                        fishing.seed === null
                          ? "fishingNeedsSave"
                          : "fishingSource",
                      )}
                    </p>
                    <button
                      onClick={() => {
                        setTab("maps");
                        setSelected(fishing.map_id);
                        setSearch("");
                      }}
                    >
                      {t("fishingOpenMap")} · {fishing.map_id}
                    </button>
                  </div>
                )}
                <h3>{t("evolution")}</h3>
                {detail.evolutions.map((e) => (
                  <div className="reference-line" key={e.offset}>
                    <button
                      className="link-button"
                      onClick={() => goSpecies(e.target)}
                    >
                      {catalog.species.find((s) => s.id === e.target)?.name ??
                        e.target}
                    </button>
                    <span className="muted small">
                      {t("method")} {e.method} · {t("parameter")} {e.parameter}
                    </span>
                  </div>
                ))}
                {!!detail.battle_forms?.length && (
                  <section className="battle-form-reference">
                    <h3>{t("battleForms")}</h3>
                    <p className="small muted">{t("battleFormsHelp")}</p>
                    {detail.battle_forms.map((form) => (
                      <div className="reference-line" key={form.offset}>
                        <button
                          className="link-button"
                          onClick={() =>
                            goSpecies(
                              form.source === speciesId
                                ? form.target
                                : form.source,
                            )
                          }
                        >
                          {
                            catalog.species.find((s) => s.id === form.source)
                              ?.name
                          }{" "}
                          →{" "}
                          {
                            catalog.species.find((s) => s.id === form.target)
                              ?.name
                          }
                        </button>
                        <span>
                          {t(form.kind)} ·{" "}
                          {t(
                            form.trigger.kind === "held_item"
                              ? "held_item"
                              : "move",
                          )}{" "}
                          ·{" "}
                          {(form.trigger.kind === "held_item"
                            ? catalog.items
                            : catalog.moves
                          ).find((entry) => entry.id === form.trigger.id)
                            ?.name ?? form.trigger.id}
                        </span>
                      </div>
                    ))}
                  </section>
                )}
                <h3>{t("learnset")}</h3>
                {detail.teaching_list_present === false && (
                  <p className="small muted">{t("missingTeachingList")}</p>
                )}
                {catalog.profile.teaching?.shared_lists != null && (
                  <p className="small muted">{t("sharedTeachingHelp")}</p>
                )}
                {catalog.profile.capabilities?.complete_learnsets === false && (
                  <p className="small muted">{t("partialLearnsetHelp")}</p>
                )}
                <div className="learnset-table">
                  <table>
                    <thead>
                      <tr>
                        <th>{t("move")}</th>
                        <th>{t("source")}</th>
                        <th>{t("level")}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {detail.learnset.map((s, i) => (
                        <tr key={`${s.offset}:${i}`}>
                          <td>
                            <button
                              className="link-button"
                              onClick={() => {
                                setTab("moves");
                                setSelected(s.move_id);
                              }}
                            >
                              {catalog.moves[s.move_id]?.name ?? s.move_id}
                            </button>
                          </td>
                          <td>
                            {t(
                              s.source === "level"
                                ? "levelSource"
                                : s.source === "egg"
                                  ? "eggSource"
                                  : s.source,
                            )}
                            {s.species !== +selected && (
                              <small>
                                {" "}
                                ·{" "}
                                {
                                  catalog.species.find(
                                    (p) => p.id === s.species,
                                  )?.name
                                }
                              </small>
                            )}
                          </td>
                          <td>{s.level ?? (s.index ? `#${s.index}` : "—")}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <h3>{t("encounter")}</h3>
                {!detail.encounters.length && (
                  <p className="muted">
                    {t(
                      detail.encounters_verified === false
                        ? "unverifiedEncounters"
                        : "noEncounters",
                    )}
                  </p>
                )}
                {detail.encounters.map((e, i) => (
                  <div
                    className="encounter-card"
                    draggable={
                      catalog.profile.capabilities?.save_edit !== false
                    }
                    onDragStart={(event) => drag(event, template(e))}
                    key={`${e.map_id}:${e.offset}:${i}`}
                  >
                    <div>
                      <button
                        className="link-button"
                        onClick={() => {
                          setTab("maps");
                          setSelected(e.map_id);
                        }}
                      >
                        {e.map_name}
                      </button>
                      <span>
                        Lv. {e.min_level}
                        {e.max_level !== e.min_level ? `–${e.max_level}` : ""}
                      </span>
                    </div>
                    <div className="muted small">
                      {encounterMethod(e.method)}
                      {e.weight !== null
                        ? ` · ${t("weight")} ${e.weight}%`
                        : ""}
                      {e.encounter_rate !== null
                        ? ` · ${t("triggerRate")} ${e.encounter_rate}`
                        : ""}
                    </div>
                    {e.selector && (
                      <div className="warning-text small">
                        {t("scriptVariable")} 0x
                        {e.selector.variable.toString(16).toUpperCase()} ={" "}
                        {e.selector.value}
                        {e.selector.fallback
                          ? ` · ${t("fallbackVariant")}`
                          : ""}
                      </div>
                    )}
                    {e.conditional && !e.selector && (
                      <div className="warning-text small">
                        {t("conditional")}
                      </div>
                    )}
                    <button
                      className="link-button small"
                      onClick={() => onTemplate(template(e))}
                    >
                      {t("pickDestination")} ↗
                    </button>
                  </div>
                ))}
                <details>
                  <summary>{t("evidence")}</summary>
                  <pre>
                    {JSON.stringify(
                      {
                        offset: detail.species.offset,
                        evolutions: detail.evolutions,
                        learnset: detail.learnset,
                      },
                      null,
                      2,
                    )}
                  </pre>
                </details>
              </>
            ) : (
              <p className="muted">{t("loading")}</p>
            ))}
          {tab === "moves" && current && (
            <>
              <p>{(current as Move).description}</p>
              <div className="metric-grid">
                {(
                  [
                    "power",
                    "accuracy",
                    "pp",
                    "priority",
                    "effect",
                    "chance",
                  ] as const
                ).map((k) => (
                  <div key={k}>
                    <span>{t(k)}</span>
                    <strong>
                      {k === "power" &&
                      current.id === catalog.profile.hidden_power?.move_id
                        ? "30–70"
                        : (current as Move)[k]}
                    </strong>
                  </div>
                ))}
              </div>
              {current.id === catalog.profile.hidden_power?.move_id ? (
                <p className="small muted">
                  {t(
                    catalog.profile.hidden_power?.formula === "gen6_fixed60"
                      ? "hiddenPowerFixedHelp"
                      : "hiddenPowerReferenceHelp",
                  )}
                </p>
              ) : (
                <Types values={[(current as Move).move_type]} />
              )}
            </>
          )}
          {tab === "items" && current && (
            <>
              <p>{(current as Item).description}</p>
              <div className="detail-pairs">
                <span>{t("price")}</span>
                <span>{(current as Item).price}</span>
                <span>{t("pocket")}</span>
                <span>{(current as Item).pocket}</span>
              </div>
              {(current as Item).tm_move && (
                <button
                  className="link-button"
                  onClick={() => {
                    setTab("moves");
                    setSelected((current as Item).tm_move!);
                  }}
                >
                  {catalog.moves[(current as Item).tm_move!]?.name}
                </button>
              )}
            </>
          )}
          {tab === "abilities" && current && (
            <p>{(current as Ability).description}</p>
          )}
          {tab === "maps" && current && (
            <>
              <div className="muted">
                {(current as GameMap).width} × {(current as GameMap).height}
              </div>
              {mapImage ? (
                <MapExplorer
                  map={current as GameMap}
                  image={mapImage}
                  report={world?.map_events.find((m) => m.map_id === selected)}
                  catalog={catalog}
                  fishing={fishing?.map_id === selected ? fishing : null}
                />
              ) : (
                <p className="muted">{t("loading")}</p>
              )}
              <h3>{t("mapTrainers")}</h3>
              <p className="small muted">{t("trainerMapsHelp")}</p>
              {world?.trainer_locations.locations
                .filter((l) => l.map_id === selected)
                .map((l) => (
                  <div className="reference-line" key={l.trainer_id}>
                    <button
                      className="link-button"
                      onClick={() => {
                        setTab("trainers");
                        setTrainerFilters(emptyTrainerFilters);
                        setSelected(l.trainer_id);
                        setSearch("");
                      }}
                    >
                      {world.trainers.find((t) => t.id === l.trainer_id)
                        ?.name ?? `#${l.trainer_id}`}{" "}
                      · #{l.trainer_id} ↗
                    </button>
                  </div>
                ))}
              <h3>{t("encounter")}</h3>
              {world?.encounters
                .filter((e) => e.map_id === selected)
                .map((e, i) => (
                  <div
                    className="encounter-card"
                    key={`${e.offset}:${i}`}
                    draggable={
                      catalog.profile.capabilities?.save_edit !== false
                    }
                    onDragStart={(event) =>
                      drag(event, {
                        species: e.species,
                        level: e.min_level,
                        met_location: e.region,
                        egg: e.method === "egg",
                      })
                    }
                  >
                    <div>
                      <button
                        className="link-button"
                        onClick={() => goSpecies(e.species)}
                      >
                        {catalog.species.find((s) => s.id === e.species)?.name}
                      </button>
                      <span>
                        Lv. {e.min_level}–{e.max_level}
                      </span>
                    </div>
                    <span className="muted small">
                      {encounterMethod(e.method)}
                      {e.weight !== null ? ` · ${e.weight}%` : ""}
                      {e.selector &&
                        ` · ${t("scriptVariable")} 0x${e.selector.variable.toString(16).toUpperCase()} = ${e.selector.value}${e.selector.fallback ? ` · ${t("fallbackVariant")}` : ""}`}
                    </span>
                    <button
                      className="link-button small"
                      onClick={() =>
                        onTemplate({
                          species: e.species,
                          level: e.min_level,
                          met_location: e.region,
                          egg: e.method === "egg",
                        })
                      }
                    >
                      {t("pickDestination")} ↗
                    </button>
                  </div>
                ))}
            </>
          )}
          {tab === "trainers" && current && filtered.length > 0 && (
            <>
              <TrainerArt
                trainer={current as Opponent}
                world={world}
                catalog={catalog}
              />
              <div
                className="trainer-context-tags"
                aria-label={t("trainerTags")}
              >
                {trainerEntryById.get(+selected)?.tags.map((tag) => (
                  <button
                    key={`${tag.facet}:${tag.value}`}
                    className={`trainer-tag ${tag.facet}`}
                    title={`${t("trainerFilterBy")} ${tag.label}`}
                    onClick={() =>
                      setTrainerFilters((filters) => ({
                        ...filters,
                        [tag.facet]: tag.value,
                      }))
                    }
                  >
                    {tag.label}
                  </button>
                ))}
              </div>
              <p className="small muted">{t("trainerTagsHelp")}</p>
              <h3>{t("trainerMaps")}</h3>
              <p className="small muted">{t("trainerMapsHelp")}</p>
              {world?.trainer_locations.locations
                .filter((l) => l.trainer_id === +selected)
                .map((l) => (
                  <div className="reference-line" key={l.map_id}>
                    <button
                      className="link-button"
                      onClick={() => {
                        setTab("maps");
                        setSelected(l.map_id);
                        setSearch("");
                      }}
                    >
                      {l.map_name} ↗
                    </button>
                    <details>
                      <summary>{t("evidence")}</summary>
                      <code>
                        {l.battle_offsets
                          .map((o) => `0x${o.toString(16).toUpperCase()}`)
                          .join(", ")}
                      </code>
                    </details>
                  </div>
                ))}
              {world &&
                !world.trainer_locations.locations.some(
                  (l) => l.trainer_id === +selected,
                ) && <p className="muted">{t("trainerMapsUnknown")}</p>}
              {!world && <p className="muted">{t("loading")}</p>}
              {!!(current as Opponent).diagnostics.length && (
                <div className="warning-text">
                  {t("trainerDiagnostics")}
                  <pre>{(current as Opponent).diagnostics.join("\n")}</pre>
                </div>
              )}
              <div className="muted small">
                {(current as Opponent).party.length} Pokémon · AI{" "}
                {(current as Opponent).ai} · {t("items")}{" "}
                {(current as Opponent).items
                  .filter(Boolean)
                  .map((id) => catalog.items[id]?.name ?? id)
                  .join(" / ") || "—"}
              </div>
              <TrainerParty
                trainer={current as Opponent}
                catalog={catalog}
                save={save}
                onSpecies={goSpecies}
                onAbility={(id) => {
                  setTab("abilities");
                  setSelected(id);
                  setSearch("");
                }}
              />
            </>
          )}
        </div>
      </div>
    </Floating>
  );
}

function RomPatch({
  catalog,
  table,
  id,
  onError,
}: {
  catalog: Catalog;
  table: string;
  id: number;
  onError: (e: unknown) => void;
}) {
  const { t } = useI18n();
  const fields =
    table === "species"
      ? [
          "hp",
          "attack",
          "defense",
          "speed",
          "sp_attack",
          "sp_defense",
          "catch_rate",
          "friendship",
          "growth",
          "ability1",
          "ability2",
        ]
      : table === "moves"
        ? ["power", "accuracy", "pp", "chance"]
        : ["price"];
  const [field, setField] = useState(fields[0]);
  const species = catalog.species.find((s) => s.id === id);
  const stats = ["hp", "attack", "defense", "speed", "sp_attack", "sp_defense"];
  const original =
    table === "species" && species
      ? stats.includes(field)
        ? species.stats[stats.indexOf(field)]
        : field === "ability1"
          ? species.abilities[0]
          : field === "ability2"
            ? species.abilities[1]
            : Number(
                (species as unknown as Record<string, unknown>)[field] ?? 0,
              )
      : table === "moves"
        ? Number(
            (catalog.moves[id] as unknown as Record<string, unknown>)?.[
              field
            ] ?? 0,
          )
        : (catalog.items[id]?.price ?? 0);
  const [value, setValue] = useState(original);
  useEffect(() => setValue(original), [original, field, id, table]);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    setField(fields[0]);
  }, [table, id]);
  return (
    <form
      className="rom-patch"
      onSubmit={async (e) => {
        e.preventDefault();
        setBusy(true);
        try {
          const path = native
            ? await outputPath(`${catalog.profile.id}-edited.gba`, "gba")
            : undefined;
          if (native && !path) return;
          const result = await api<{ bytes?: string; manifest: unknown }>(
            "patch_rom",
            { edits: [{ table, id, field, value }], ...(path ? { path } : {}) },
          );
          if (result.bytes)
            download(
              `${catalog.profile.id}-edited.gba`,
              fromBase64(result.bytes),
            );
          if (!native)
            download(
              `${catalog.profile.id}.patch.json`,
              JSON.stringify(result.manifest, null, 2),
              "application/json",
            );
        } catch (e) {
          onError(e);
        } finally {
          setBusy(false);
        }
      }}
    >
      <h3>{t("romEdit")}</h3>
      <p className="small muted">{t("romEditHelp")}</p>
      <div className="field-grid">
        <NumberField
          label={t("before")}
          value={original}
          onChange={() => {}}
          disabled
          max={65535}
        />
        <SelectField
          label={t("field")}
          value={field}
          onChange={setField}
          options={fields.map((f) => ({ value: f, label: t(f) }))}
        />
        <NumberField
          label={t("value")}
          value={value}
          max={65535}
          onChange={setValue}
        />
      </div>
      <button disabled={busy} type="submit">
        {t(busy ? "working" : "exportRom")}
      </button>
    </form>
  );
}
