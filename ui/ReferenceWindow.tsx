import { LearnsetPanel } from "./LearnsetPanel";
import { MapEncounters } from "./MapEncounters";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Search, ArrowUpRight } from "lucide-react";
import { api } from "./api";
import { Floating, Sprite, Types } from "./components";
import { AcquisitionPanel } from "./AcquisitionPanel";
import { MapNavigationPanel } from "./MapNavigationPanel";
import { MapExplorer } from "./MapExplorer";
import { TrainerArt } from "./TrainerArt";
import { TrainerParty } from "./TrainerParty";
import { SpeciesStats } from "./SpeciesStats";
import { EvolutionTree } from "./EvolutionTree";
import { TrainerLocationsPanel } from "./TrainerLocationsPanel";
import {
  emptyTrainerFilters,
  indexTrainers,
  searchTrainers,
  trainerFacetOptions,
  type TrainerFacet,
  type TrainerFilters,
} from "./trainerSearch";
import { useI18n, moveCategoryNames } from "./i18n";
import { itemPocketLabel } from "./referenceLabels";
import type {
  QueryTarget,
  MapNavigation,
  MapFocus,
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
  TrainerDifficulty,
  World,
  Snapshot,
  FishingReport,
} from "./types";

interface Props {
  window: RefWindow;
  catalog: Catalog;
  world: World | null;
  save: Snapshot | null;
  trainerDifficulty: TrainerDifficulty;
  onTrainerDifficulty: (value: TrainerDifficulty) => void;
  loadWorld: () => void;
  onClose: () => void;
  onTemplate: (template: Template) => void;
  onError: (error: unknown) => void;
}

export function ReferenceWindow({
  window: info,
  catalog,
  world,
  save,
  trainerDifficulty,
  onTrainerDifficulty,
  loadWorld,
  onClose,
  onTemplate,
  onError,
}: Props) {
  const { t, locale } = useI18n();
  const [tab, setTab] = useState<RefTab>(info.tab);
  const [selected, setSelected] = useState<number | string>(info.selected ?? 1);
  const [mapFocus, setMapFocus] = useState<MapFocus | undefined>(info.focus);
  const [search, setSearch] = useState("");
  const [showUnused, setShowUnused] = useState(false);
  const [navigation, setNavigation] = useState<MapNavigation | null>(null);
  const [history, setHistory] = useState<
    {
      tab: RefTab;
      selected: number | string;
      focus?: MapFocus;
      search: string;
    }[]
  >([]);
  const lastPage = useRef({
    tab,
    selected,
    focus: mapFocus,
    search,
  });
  const returning = useRef(false);
  useEffect(() => {
    const last = lastPage.current;
    if (last.tab !== tab || last.selected !== selected) {
      if (!returning.current) setHistory((h) => [...h.slice(-39), last]);
      returning.current = false;
    }
    lastPage.current = {
      tab,
      selected,
      focus: mapFocus,
      search,
    };
  }, [tab, selected, mapFocus, search]);
  const goMap = (id: string, focus?: MapFocus) => {
    setMapFocus(focus);
    setTab("maps");
    setSelected(id);
    setSearch("");
  };
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
  const [mapImageWarnings, setMapImageWarnings] = useState<string[]>([]);
  const [mapImageError, setMapImageError] = useState(false);
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
  const detailPane = useRef<HTMLDivElement>(null);
  const treeScroll = useRef<number | null>(null);
  useLayoutEffect(() => {
    if (detail?.species.id === selected && treeScroll.current !== null) {
      detailPane.current?.scrollTo({ top: treeScroll.current });
      treeScroll.current = null;
    }
  }, [detail, selected]);
  useLayoutEffect(() => {
    detailPane.current?.scrollTo({ top: 0 });
  }, [tab, selected]);
  useEffect(() => {
    loadWorld();
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
  const hidden =
    tab === "maps"
      ? world?.reference_visibility?.maps
      : tab === "items"
        ? world?.reference_visibility?.items
        : undefined;
  const hiddenIds = new Set(hidden?.map((row) => row.id));
  const visibleRows = rows.filter(
    (row) =>
      showUnused ||
      !hiddenIds.has(row.id as never) ||
      (tab === "items" && save?.bag.some((entry) => entry.item === row.id)),
  );
  const filtered =
    tab === "trainers"
      ? searchTrainers(trainerEntries, search, trainerFilters).map(
          (entry) => entry.trainer,
        )
      : visibleRows.filter((row) =>
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
    setMapImageWarnings([]);
    setMapImageError(false);
    if (tab === "maps" && typeof selected === "string")
      api<{ url: string; warnings?: string[] }>("map_image", { id: selected })
        .then((r) => {
          if (active) {
            setMapImage(r.url);
            setMapImageWarnings(r.warnings ?? []);
          }
        })
        .catch((error) => {
          if (active) {
            setMapImageError(true);
            onError(error);
          }
        });
    return () => {
      active = false;
    };
  }, [tab, selected, catalog.profile.md5, onError]);
  useEffect(() => {
    let active = true;
    setNavigation(null);
    if (tab === "maps" && typeof selected === "string")
      api<MapNavigation>("map_navigation", { id: selected })
        .then((value) => {
          if (active) setNavigation(value);
        })
        .catch((error) => {
          if (active) onError(error);
        });
    return () => {
      active = false;
    };
  }, [tab, selected, catalog.profile.md5, save, onError]);
  const goSpecies = (id: number) => {
    setTab("species");
    setSelected(id);
    setSearch("");
  };
  const goTarget = (target: QueryTarget) => {
    setTab(
      target.kind === "species"
        ? "species"
        : target.kind === "item"
          ? "items"
          : "moves",
    );
    setSelected(target.id);
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
              }}
            >
              {t(key)}
            </button>
          ))}
      </div>
      <button
        type="button"
        className="link-button reference-back"
        disabled={!history.length}
        onClick={() => {
          const previous = history.at(-1);
          if (!previous) return;
          returning.current = true;
          setHistory((h) => h.slice(0, -1));
          setTab(previous.tab);
          setSelected(previous.selected);
          setMapFocus(previous.focus);
          setSearch(previous.search);

          setTrainerFilters(emptyTrainerFilters);
        }}
      >
        ← {t("navBack")}
      </button>
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
          {(tab === "items" || tab === "maps") && !!hidden?.length && (
            <label className="reference-optional">
              <input
                type="checkbox"
                checked={showUnused}
                onChange={(e) => setShowUnused(e.target.checked)}
              />
              {t("showUnusedReferences")} <small>({hidden.length})</small>
            </label>
          )}
          {tab === "trainers" && (
            <div className="trainer-filters">
              {catalog.profile.id === "ultimate-emerald-55" && (
                <label className="trainer-filter">
                  <span>{t("trainerDifficulty")}</span>
                  <select
                    value={trainerDifficulty}
                    onChange={(e) =>
                      onTrainerDifficulty(
                        Number(e.target.value) as TrainerDifficulty,
                      )
                    }
                  >
                    {([1, 2, 3, 4] as TrainerDifficulty[]).map((mode) => (
                      <option value={mode} key={mode}>
                        {t(`trainerDifficulty_${mode}`)}
                      </option>
                    ))}
                  </select>
                </label>
              )}
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
          {!!hidden?.length && (tab === "items" || tab === "maps") && (
            <details className="reference-help">
              <summary>{t("referenceSourceHelp")}</summary>
              <p className="small muted">{t("referenceFilteredHelp")}</p>
            </details>
          )}
          <div className="reference-rows">
            {filtered.map((row) => (
              <button
                type="button"
                key={row.id}
                className={selected === row.id ? "selected" : ""}
                onClick={() => {
                  setSelected(row.id);
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
                        : `${catalog.profile.id === "ultimate-emerald-55" ? `${t("trainerBaseLevel")} ` : "Lv. "}${Math.min(...(row as Opponent).party.map((p) => p.level))}–${Math.max(...(row as Opponent).party.map((p) => p.level))}`}
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
          </div>
          {current && (tab !== "trainers" || filtered.length > 0) && (
            <h2>
              {current.name} <small>#{current.id}</small>
            </h2>
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
                    <Types catalog={catalog} values={detail.species.types} />
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
                <SpeciesStats detail={detail} catalog={catalog} />
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
                <EvolutionTree
                  detail={detail}
                  catalog={catalog}
                  maps={world?.maps ?? []}
                  onTarget={goTarget}
                  onMap={goMap}
                  onNavigate={(id) => {
                    if (id === speciesId) return;
                    treeScroll.current = detailPane.current?.scrollTop ?? 0;
                    goSpecies(id);
                  }}
                />
                <LearnsetPanel
                  detail={detail}
                  catalog={catalog}
                  onMove={(id) => goTarget({ kind: "move", id })}
                />
                <AcquisitionPanel
                  target={{ kind: "species", id: +selected }}
                  catalog={catalog}
                  maps={world?.maps ?? []}
                  save={save}
                  onTarget={goTarget}
                  onMap={goMap}
                  onError={onError}
                  onTemplate={onTemplate}
                />
              </>
            ) : (
              <p className="muted">{t("loading")}</p>
            ))}
          {tab === "moves" && current && (
            <>
              <h3>{t("moveEffectText")}</h3>
              <p>{(current as Move).description || t("unresolved")}</p>
              <div className="metric-grid">
                {(
                  ["power", "accuracy", "pp", "priority", "chance"] as const
                ).map((k) => (
                  <div key={k}>
                    <span>{t(k)}</span>
                    <strong>
                      {k === "power" &&
                      current.id === catalog.profile.hidden_power?.move_id
                        ? catalog.profile.hidden_power?.formula ===
                          "gen6_fixed60"
                          ? "60"
                          : "30–70"
                        : k === "accuracy" || k === "chance"
                          ? (current as Move)[k]
                            ? `${(current as Move)[k]}%`
                            : "—"
                          : k === "power" && !(current as Move).power
                            ? "—"
                            : (current as Move)[k]}
                    </strong>
                  </div>
                ))}
              </div>
              <p>
                {t("moveCategory")} ·{" "}
                {moveCategoryNames[locale][(current as Move).category] ??
                  t("unresolved")}
              </p>
              {current.id === catalog.profile.hidden_power?.move_id ? (
                <p className="small muted">
                  {t(
                    catalog.profile.hidden_power?.formula === "gen6_fixed60"
                      ? "hiddenPowerFixedHelp"
                      : "hiddenPowerReferenceHelp",
                  )}
                </p>
              ) : (
                <Types
                  catalog={catalog}
                  values={[(current as Move).move_type]}
                />
              )}
            </>
          )}
          {tab === "moves" && current && (
            <AcquisitionPanel
              target={{ kind: "move", id: +selected }}
              catalog={catalog}
              maps={world?.maps ?? []}
              save={save}
              onTarget={goTarget}
              onMap={goMap}
              onError={onError}
            />
          )}
          {tab === "items" && current && (
            <>
              <p>{(current as Item).description}</p>
              <AcquisitionPanel
                target={{ kind: "item", id: +selected }}
                catalog={catalog}
                maps={world?.maps ?? []}
                save={save}
                onTarget={goTarget}
                onMap={goMap}
                onError={onError}
              />
              <div className="detail-pairs">
                <span>{t("price")}</span>
                <span>{(current as Item).price}</span>
                <span>{t("pocket")}</span>
                <span>
                  {itemPocketLabel((current as Item).pocket, catalog, t)}
                </span>
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
              {hidden?.find((row) => row.id === current.id)?.reason ===
                "invalid_layout" && (
                <p className="small muted">{t("referenceInvalidMap")}</p>
              )}
              <div className="muted">
                {(current as GameMap).width} × {(current as GameMap).height}
              </div>
              {mapImage ? (
                <MapExplorer
                  map={current as GameMap}
                  image={mapImage}
                  warnings={mapImageWarnings}
                  report={world?.map_events.find((m) => m.map_id === selected)}
                  catalog={catalog}
                  fishing={fishing?.map_id === selected ? fishing : null}
                  focus={mapFocus}
                  navigation={navigation}
                  onMap={goMap}
                  onItem={(id) => goTarget({ kind: "item", id })}
                  onSpecies={(id) => goTarget({ kind: "species", id })}
                  onMove={(id) => goTarget({ kind: "move", id })}
                />
              ) : mapImageError ? (
                <p className="warning-text">{t("mapImageUnavailable")}</p>
              ) : (
                <p className="muted">{t("loading")}</p>
              )}
              <MapNavigationPanel
                report={navigation}
                maps={world?.maps ?? []}
                onMap={goMap}
                catalog={catalog}
                onTarget={goTarget}
              />
              <details
                className="reference-section"
                key={`map-trainers:${selected}`}
              >
                <summary>{t("mapTrainers")}</summary>
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
              </details>
              <MapEncounters
                catalog={catalog}
                save={save}
                encounters={(world?.encounters ?? []).filter(
                  (e) => e.map_id === selected,
                )}
                onSpecies={goSpecies}
                onTemplate={onTemplate}
              />
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
              {!!(current as Opponent).diagnostics.length && (
                <p className="warning-text">{t("trainerIncompleteRecord")}</p>
              )}
              <div className="muted small">
                {(current as Opponent).party.length} Pokémon · {t("items")}{" "}
                {(current as Opponent).items
                  .filter(Boolean)
                  .map((id) => catalog.items[id]?.name ?? id)
                  .join(" / ") || "—"}
              </div>

              <TrainerParty
                trainer={current as Opponent}
                catalog={catalog}
                save={save}
                difficulty={
                  catalog.profile.id === "ultimate-emerald-55"
                    ? trainerDifficulty
                    : null
                }
                onSpecies={goSpecies}
                onAbility={(id) => {
                  setTab("abilities");
                  setSelected(id);
                  setSearch("");
                }}
              />
              <TrainerLocationsPanel
                catalog={catalog}
                trainerId={+selected}
                onMap={goMap}
                onTarget={goTarget}
              />
            </>
          )}
        </div>
      </div>
    </Floating>
  );
}
