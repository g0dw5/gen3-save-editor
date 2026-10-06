import { StaticBattleDetails } from "./StaticBattleDetails";
import { ConditionDetails } from "./ConditionDetails";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  Gift,
  MapPin,
  Search,
  Sparkles,
  Users,
  CircleDot,
  Fish,
  DoorOpen,
} from "lucide-react";
import { useI18n } from "./i18n";
import { TradeDetails } from "./TradeDetails";
import { useRomCharacterImage } from "./romCharacterImage";
import type {
  MapFocus,
  MapNavigation,
  Catalog,
  GameMap,
  MapEventReport,
  MapMarker,
  FishingReport,
} from "./types";

const layers = ["pickup", "hidden", "gift", "npc"] as const;
const icons = { pickup: CircleDot, hidden: Sparkles, gift: Gift, npc: Users };
export function MapExplorer({
  map,
  image,
  warnings = [],
  report,
  catalog,
  fishing,
  focus,
  navigation,
  onMap,
  onItem,
  onSpecies,
  onMove,
}: {
  map: GameMap;
  image: string;
  warnings?: string[];
  report?: MapEventReport;
  catalog: Catalog;
  fishing?: FishingReport | null;
  focus?: MapFocus;
  navigation?: MapNavigation | null;
  onMap?: (id: string, focus?: MapFocus) => void;
  onItem?: (id: number) => void;
  onSpecies?: (id: number) => void;
  onMove?: (id: number) => void;
}) {
  const { t } = useI18n();
  const [enabled, setEnabled] = useState({
    pickup: true,
    hidden: true,
    gift: true,
    npc: true,
  });
  const [query, setQuery] = useState("");
  const [showFishing, setShowFishing] = useState(true);
  const [fishSelected, setFishSelected] = useState<string | null>(null);
  const fishPins = useRef(new Map<string, HTMLButtonElement>());
  useEffect(() => setFishSelected(null), [map.id, fishing]);
  const fishName =
    catalog.species.find((s) => s.id === fishing?.species)?.name ?? "";
  const fishVisible = showFishing
    ? (fishing?.spots ?? []).filter((s) =>
        `${fishName} ${s.x},${s.y}`.toLowerCase().includes(query.toLowerCase()),
      )
    : [];
  const locateFish = (x: number, y: number) => {
    const key = `${x},${y}`;
    setShowFishing(true);
    setQuery("");
    setFishSelected(key);
    setSelected(null);
    requestAnimationFrame(() =>
      fishPins.current.get(key)?.scrollIntoView({
        block: "center",
        inline: "center",
        behavior: "smooth",
      }),
    );
  };
  const [showWarps, setShowWarps] = useState(true);
  const focusPin = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    setQuery("");
    if (focus) {
      setSelected(`${focus.x},${focus.y}`);
      requestAnimationFrame(() =>
        focusPin.current?.scrollIntoView({ block: "center", inline: "center" }),
      );
    }
  }, [map.id, focus]);
  const [grid, setGrid] = useState(false);
  const [zoom, setZoom] = useState(1);
  const [selected, setSelected] = useState<string | null>(null);
  useEffect(() => {
    if (!focus) setSelected(null);
  }, [map.id, focus]);
  const label = (kind: string) =>
    t(
      (
        {
          pickup: "mapPickups",
          hidden: "mapHidden",
          gift: "mapGifts",
          npc: "mapNpcs",
        } as const
      )[kind as (typeof layers)[number]] ?? "mapNpcs",
    );
  const items = useMemo(
    () => new Map(catalog.items.map((i) => [i.id, i])),
    [catalog],
  );
  const rewardNames = (marker: MapMarker) =>
    [
      ...new Set([
        ...marker.rewards.map((r) => items.get(r.item)?.name ?? `#${r.item}`),
        ...(marker.teaching ?? []).map(
          (offer) =>
            catalog.moves.find((m) => m.id === offer.move_id)?.name ??
            `#${offer.move_id}`,
        ),
        ...(marker.pokemon ?? []).map(
          (p) =>
            catalog.species.find((s) => s.id === p.species)?.name ??
            `#${p.species}`,
        ),
      ]),
    ].join(" / ");
  const markers = report?.markers ?? [];
  const visible = markers.filter(
    (m) =>
      enabled[m.kind === "event" ? "npc" : m.kind] &&
      `${rewardNames(m)} ${m.local_id ?? ""} ${m.x},${m.y}`
        .toLowerCase()
        .includes(query.toLowerCase()),
  );
  const groups = new Map<string, MapMarker[]>();
  for (const marker of visible) {
    if (
      marker.x < 0 ||
      marker.y < 0 ||
      marker.x >= map.width ||
      marker.y >= map.height
    )
      continue;
    const key = `${marker.x},${marker.y}`;
    groups.set(key, [...(groups.get(key) ?? []), marker]);
  }
  const chosen = selected ? groups.get(selected) : undefined;
  return (
    <section className="map-explorer">
      <div className="map-layer-controls" aria-label={t("mapLayers")}>
        {layers.map((kind) => {
          const Icon = icons[kind];
          return (
            <label key={kind} className={`map-layer layer-${kind}`}>
              <input
                type="checkbox"
                checked={enabled[kind]}
                onChange={(e) =>
                  setEnabled({ ...enabled, [kind]: e.target.checked })
                }
              />
              <Icon size={15} />
              <span>{label(kind)}</span>
              <small>
                {
                  markers.filter(
                    (m) =>
                      m.kind === kind || (kind === "npc" && m.kind === "event"),
                  ).length
                }
              </small>
            </label>
          );
        })}
        <label className="map-layer">
          <input
            type="checkbox"
            checked={showWarps}
            onChange={(e) => setShowWarps(e.target.checked)}
          />
          <DoorOpen size={15} />
          {t("navWarps")}
        </label>
        {fishing && (
          <label className="map-layer layer-fishing">
            <input
              type="checkbox"
              checked={showFishing}
              disabled={fishing.seed === null}
              onChange={(e) => setShowFishing(e.target.checked)}
            />
            <Fish size={15} />
            <span>{t("fishingSpots")}</span>
            <small>{fishing.spots.length}</small>
          </label>
        )}
      </div>
      {fishing && (
        <div className="fishing-info">
          <strong>
            <Fish size={16} /> {fishName} · {t("fishingSpots")}
          </strong>
          {fishing.seed === null ? (
            <p className="small muted">{t("fishingNeedsSave")}</p>
          ) : (
            <>
              <p className="small">
                {t("fishingChance")} {fishing.percent}% · Lv.{fishing.min_level}
                –{fishing.max_level}
              </p>
              <div className="fishing-locations">
                {fishing.spots.map((spot) => (
                  <button
                    key={`${spot.x},${spot.y}`}
                    aria-label={`${t("fishingLocate")} (${spot.x}, ${spot.y})`}
                    aria-pressed={fishSelected === `${spot.x},${spot.y}`}
                    onClick={() => locateFish(spot.x, spot.y)}
                  >
                    ({spot.x}, {spot.y})
                  </button>
                ))}
              </div>
              <p className="small muted">{t("fishingHelp")}</p>
              <p className="small muted">{t("fishingSource")}</p>
            </>
          )}
        </div>
      )}
      {warnings.map((warning) => (
        <p key={warning} className="warning-text" role="status">
          {t(warning as Parameters<typeof t>[0])}
        </p>
      ))}
      {map.invalid_events && (
        <p className="warning-text">{t("mapEventUnavailable")}</p>
      )}
      <div className="map-tools">
        <label className="map-search">
          <Search size={15} />
          <input
            aria-label={t("mapItemSearch")}
            placeholder={t("mapItemSearch")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={grid}
            onChange={(e) => setGrid(e.target.checked)}
          />{" "}
          {t("mapGrid")}
        </label>
        <select
          aria-label={t("mapZoom")}
          value={zoom}
          onChange={(e) => setZoom(Number(e.target.value))}
        >
          {[1, 2, 3, 4].map((z) => (
            <option key={z} value={z}>
              {z === 1 ? t("mapFit") : `${z}×`}
            </option>
          ))}
        </select>
      </div>
      <div className="map-viewport">
        <div
          className="map-canvas"
          style={{
            width: `${zoom * 100}%`,
            aspectRatio: `${map.width} / ${map.height}`,
          }}
        >
          <img
            src={image}
            alt={`${t("mapPreview")} · ${map.name}`}
            draggable={false}
          />
          {grid && (
            <div
              className="map-grid"
              style={{
                backgroundSize: `${100 / map.width}% ${100 / map.height}%`,
              }}
            />
          )}
          {showWarps &&
            navigation?.outgoing
              .filter(
                (e) =>
                  (e.kind === "warp" || e.kind === "script_warp") &&
                  e.x !== null &&
                  e.y !== null &&
                  e.x >= 0 &&
                  e.y >= 0 &&
                  e.x < map.width &&
                  e.y < map.height,
              )
              .map((edge, index) => (
                <button
                  key={`warp-${edge.offset}-${index}`}
                  className="map-marker layer-warp"
                  style={{
                    left: `${(100 * (edge.x! + 0.5)) / map.width}%`,
                    top: `${(100 * (edge.y! + 0.5)) / map.height}%`,
                  }}
                  title={`${t(edge.script ? "navScriptWarp" : "navWarp")} → ${edge.to ? edge.to : t("navDynamic")}${edge.script ? ` · ${t("navScriptAccessUnknown")}` : ""}`}
                  aria-label={`${t(edge.script ? "navScriptWarp" : "navWarp")} (${edge.x}, ${edge.y})`}
                  onClick={() => {
                    if (edge.to)
                      onMap?.(
                        edge.to,
                        edge.target_x !== null && edge.target_y !== null
                          ? { x: edge.target_x, y: edge.target_y }
                          : undefined,
                      );
                  }}
                >
                  <DoorOpen size={14} />
                </button>
              ))}
          {focus &&
            focus.x >= 0 &&
            focus.y >= 0 &&
            focus.x < map.width &&
            focus.y < map.height && (
              <button
                ref={focusPin}
                className="map-marker map-focus selected"
                style={{
                  left: `${(100 * (focus.x + 0.5)) / map.width}%`,
                  top: `${(100 * (focus.y + 0.5)) / map.height}%`,
                }}
                title={t("navTarget")}
                aria-label={t("navTarget")}
                onClick={() => setSelected(`${focus.x},${focus.y}`)}
              >
                <MapPin size={18} />
              </button>
            )}
          {fishVisible.map((spot) => {
            const key = `${spot.x},${spot.y}`;
            return (
              <button
                key={`fish-${key}`}
                ref={(node) => {
                  if (node) fishPins.current.set(key, node);
                  else fishPins.current.delete(key);
                }}
                className={`map-marker layer-fishing ${fishSelected === key ? "selected" : ""}`}
                style={{
                  left: `${(100 * (spot.x + 0.5)) / map.width}%`,
                  top: `${(100 * (spot.y + 0.5)) / map.height}%`,
                }}
                title={`${fishName} · (${spot.x}, ${spot.y}) · ${fishing!.percent}%`}
                aria-label={`${fishName} (${spot.x}, ${spot.y})`}
                onClick={() => {
                  setFishSelected(key);
                  setSelected(null);
                }}
              >
                <Fish size={14} />
              </button>
            );
          })}
          {[...groups].map(([key, group]) => {
            const first =
              group.find((m) => m.kind !== "npc" && m.kind !== "event") ??
              group[0];
            const title =
              group
                .map(
                  (m) =>
                    `${label(m.kind)}${m.local_id !== null ? ` #${m.local_id}` : ""} · ${rewardNames(m)}`,
                )
                .join("\n") + `\n(${first.x}, ${first.y})`;
            return (
              <MapEventPin
                key={key}
                map={map}
                marker={first}
                actor={group.find(
                  (m) =>
                    (m.kind === "npc" || m.kind === "gift") &&
                    m.graphics_id !== null,
                )}
                md5={catalog.profile.md5}
                title={title}
                count={group.length}
                selected={key === selected}
                onClick={() => {
                  setSelected(key);
                  setFishSelected(null);
                }}
              />
            );
          })}
        </div>
      </div>
      <p className="small muted">{t("mapAllEventsHelp")}</p>
      <div className="map-marker-details" aria-live="polite">
        {chosen ? (
          chosen.map((marker) => (
            <div key={marker.id}>
              <strong>
                <MapPin size={14} /> {label(marker.kind)}{" "}
                {marker.local_id !== null ? `#${marker.local_id}` : ""} · (
                {marker.x}, {marker.y})
              </strong>
              {marker.underfoot === true && (
                <p className="small muted">{t("mapHiddenUnderfoot")}</p>
              )}
              {(marker.pokemon ?? []).map((mon, i) => (
                <div key={`pokemon-${mon.offset}-${i}`}>
                  <button
                    className="link-button"
                    onClick={() => onSpecies?.(mon.species)}
                  >
                    {catalog.species.find((s) => s.id === mon.species)?.name ??
                      `#${mon.species}`}{" "}
                    ↗
                  </button>{" "}
                  · {t(mon.method)}
                  {mon.level !== null
                    ? ` · Lv. ${mon.level}`
                    : mon.trade
                      ? ""
                      : ` · ${t("unresolved")}`}
                  {marker.pokemon.findIndex((p) => p.offset === mon.offset) ===
                    i && (
                    <StaticBattleDetails
                      mon={mon}
                      catalog={catalog}
                      onTarget={(target) =>
                        target.kind === "species"
                          ? onSpecies?.(target.id)
                          : onItem?.(target.id)
                      }
                    />
                  )}
                  <TradeDetails
                    mon={mon}
                    catalog={catalog}
                    onSpecies={onSpecies}
                    onItem={onItem}
                  />
                </div>
              ))}
              {!!marker.pokemon?.length && (
                <p className="small muted">{t("acqScriptSourceHelp")}</p>
              )}
              {(marker.teaching ?? []).map((offer, i) => (
                <p key={`teaching-${offer.offset}-${i}`}>
                  {t("move_tutor")} ·{" "}
                  <button
                    className="link-button"
                    onClick={() => onMove?.(offer.move_id)}
                  >
                    {catalog.moves.find((m) => m.id === offer.move_id)?.name ??
                      `#${offer.move_id}`}{" "}
                    ↗
                  </button>
                </p>
              ))}
              {!!marker.teaching?.length && (
                <p className="small muted">{t("tutorSourceHelp")}</p>
              )}
              {!!marker.daycare?.length && (
                <>
                  <p>{t("breedServices")}</p>
                  <p className="small muted">{t("breedServicesHelp")}</p>
                  {marker.daycare.map((offer, i) => (
                    <ConditionDetails
                      key={`daycare-${i}`}
                      conditions={offer.conditions}
                      catalog={catalog}
                      onMap={onMap}
                      onTarget={(target) => onItem?.(target.id)}
                    />
                  ))}
                </>
              )}
              {marker.rewards.length ? (
                marker.rewards.map((r, i) => (
                  <p key={`${r.offset}-${i}`}>
                    <button
                      className="link-button"
                      onClick={() => onItem?.(r.item)}
                    >
                      {items.get(r.item)?.name ?? `#${r.item}`} ↗
                    </button>
                    {r.quantity !== null
                      ? ` × ${r.quantity}`
                      : r.via === "shop"
                        ? ` · ${t("acqShop")}`
                        : ""}
                  </p>
                ))
              ) : !marker.pokemon?.length &&
                !marker.teaching?.length &&
                !marker.daycare?.length ? (
                <p>{t("mapNoReward")}</p>
              ) : null}
              {[
                ...marker.rewards,
                ...(marker.pokemon ?? []),
                ...(marker.teaching ?? []),
              ]
                .filter((r) => r.conditions.length > 0)
                .map((r, index) => (
                  <ConditionDetails
                    key={index}
                    heading={`${"item" in r ? (items.get(r.item)?.name ?? `#${r.item}`) : "species" in r ? (catalog.species.find((s) => s.id === r.species)?.name ?? `#${r.species}`) : (catalog.moves.find((m) => m.id === r.move_id)?.name ?? `#${r.move_id}`)} · ${t("conditionSourcePath")}`}
                    conditions={r.conditions}
                    catalog={catalog}
                    onMap={onMap}
                    onTarget={(target) => onItem?.(target.id)}
                  />
                ))}
              {marker.movement_type === 76 && (
                <p className="small muted">{t("mapInvisibleObject")}</p>
              )}
              {!!marker.scripted_movements?.length && (
                <p className="small muted">{t("mapScriptedMovement")}</p>
              )}
              {marker.stopped_at.length > 0 && (
                <p className="small muted">{t("mapPartialScript")}</p>
              )}
              <details>
                <summary>{t("mapEventEvidence")}</summary>
                <code>
                  {t("mapElevation")} {marker.elevation} · {t("mapEventOffset")}{" "}
                  0x{marker.offset.toString(16)}
                  {marker.script !== null
                    ? ` · Script 0x${marker.script.toString(16)}`
                    : ""}
                  {marker.flag !== null
                    ? ` · Flag 0x${marker.flag.toString(16)}`
                    : ""}
                  {marker.receipt_flag != null
                    ? ` · ${t("acqReceiptEvidence")} 0x${marker.receipt_flag.toString(16)}`
                    : ""}
                </code>
                {!!marker.scripted_movements?.length && (
                  <pre>
                    {JSON.stringify(marker.scripted_movements, null, 2)}
                  </pre>
                )}
                {!!marker.pokemon?.length && (
                  <pre>{JSON.stringify(marker.pokemon, null, 2)}</pre>
                )}
                {!!marker.teaching?.length && (
                  <pre>{JSON.stringify(marker.teaching, null, 2)}</pre>
                )}
                {marker.rewards
                  .filter((r) => r.receipt)
                  .map((r, i) => (
                    <p key={i} className="small">
                      {t("acqReceiptEvidence")} ·{" "}
                      {items.get(r.item)?.name ?? `#${r.item}`} ·
                      <code>
                        {" "}
                        Flag 0x{r.receipt!.flag.toString(16)} · Script 0x
                        {r.receipt!.root.toString(16)} ·{" "}
                        {r
                          .receipt!.success_set_offsets.map(
                            (o) => `0x${o.toString(16)}`,
                          )
                          .join(" / ")}
                      </code>
                    </p>
                  ))}
              </details>
            </div>
          ))
        ) : (
          <p className="muted">{t("mapSelectMarker")}</p>
        )}
      </div>
      {!!report?.unplaced_movements?.length && (
        <details>
          <summary>{t("mapScriptedMovementEntry")}</summary>
          <p className="small muted">{t("mapScriptedMovement")}</p>
          <pre>{JSON.stringify(report.unplaced_movements, null, 2)}</pre>
        </details>
      )}
      {!!report?.unplaced_rewards.length && (
        <details>
          <summary>{t("mapUnplacedRewards")}</summary>
          <p className="small muted">{t("mapUnplacedHelp")}</p>
          {report.unplaced_rewards.map((reward, index) => (
            <div key={index}>
              <button
                className="link-button"
                onClick={() => onItem?.(reward.item)}
              >
                {items.get(reward.item)?.name ?? `#${reward.item}`} ↗
              </button>
              <ConditionDetails
                conditions={reward.conditions}
                catalog={catalog}
                onMap={onMap}
                onTarget={(target) => onItem?.(target.id)}
              />
            </div>
          ))}
        </details>
      )}
      {!!report?.unplaced_pokemon?.length && (
        <details>
          <summary>{t("mapUnplacedPokemon")}</summary>
          <p className="small muted">{t("mapUnplacedHelp")}</p>
          {report.unplaced_pokemon.map((mon, i) => (
            <div key={i}>
              <button
                className="link-button"
                onClick={() => onSpecies?.(mon.species)}
              >
                {catalog.species.find((s) => s.id === mon.species)?.name ??
                  `#${mon.species}`}{" "}
                ↗
              </button>{" "}
              · {t(mon.method)}
              <TradeDetails
                mon={mon}
                catalog={catalog}
                onSpecies={onSpecies}
                onItem={onItem}
              />
              <ConditionDetails
                conditions={mon.conditions}
                catalog={catalog}
                onMap={onMap}
                onTarget={(target) => onItem?.(target.id)}
              />
            </div>
          ))}
        </details>
      )}
      {!!report?.unplaced_teaching?.length && (
        <details>
          <summary>{t("mapUnplacedTeaching")}</summary>
          <p className="small muted">{t("mapUnplacedHelp")}</p>
          {report.unplaced_teaching.map((offer, i) => (
            <div key={i}>
              <button
                className="link-button"
                onClick={() => onMove?.(offer.move_id)}
              >
                {catalog.moves.find((m) => m.id === offer.move_id)?.name ??
                  `#${offer.move_id}`}{" "}
                ↗
              </button>
              <ConditionDetails
                conditions={offer.conditions}
                catalog={catalog}
                onMap={onMap}
                onTarget={(target) => onItem?.(target.id)}
              />
            </div>
          ))}
          <p className="small muted">{t("tutorSourceHelp")}</p>
        </details>
      )}
      {!!report?.unplaced_daycare?.length && (
        <details>
          <summary>{t("breedServices")}</summary>
          <p className="small muted">{t("mapUnplacedHelp")}</p>
          <p className="small muted">{t("breedServicesHelp")}</p>
          {report.unplaced_daycare.map((offer, i) => (
            <ConditionDetails
              key={i}
              conditions={offer.conditions}
              catalog={catalog}
              onMap={onMap}
              onTarget={(target) => onItem?.(target.id)}
            />
          ))}
        </details>
      )}
      {report &&
        (report.stopped_at.length > 0 ||
          markers.some((m) => m.stopped_at.length > 0)) && (
          <p className="small muted">{t("mapCoverageHelp")}</p>
        )}
    </section>
  );
}

function MapEventPin({
  map,
  marker,
  actor,
  md5,
  title,
  count,
  selected,
  onClick,
}: {
  map: GameMap;
  marker: MapMarker;
  actor?: MapMarker;
  md5: string;
  title: string;
  count: number;
  selected: boolean;
  onClick: () => void;
}) {
  const image = useRomCharacterImage(
    md5,
    "object_sprite",
    actor?.graphics_id ?? null,
  );
  const kind = marker.kind === "event" ? "npc" : marker.kind;
  const Icon = icons[kind];
  const { t } = useI18n();
  const description =
    actor && image === null ? `${title} · ${t("artUnavailable")}` : title;
  return (
    <button
      type="button"
      className={`map-marker layer-${kind}${image ? " map-actor" : ""}${actor?.movement_type === 76 ? " map-actor-invisible" : ""}${selected ? " selected" : ""}`}
      style={{
        left: `${((marker.x + 0.5) / map.width) * 100}%`,
        top: `${((marker.y + (image ? 1 : 0.5)) / map.height) * 100}%`,
        // Map tiles are 16 pixels. Keep the ROM sprite's size and anchor its feet
        // to the bottom of its event tile, including when zooming the map.
        ...(image
          ? {
              width: `${(image.width / (map.width * 16)) * 100}%`,
              height: `${(image.height / (map.height * 16)) * 100}%`,
            }
          : {}),
      }}
      aria-label={description}
      title={description}
      aria-pressed={selected}
      onClick={onClick}
    >
      {image ? (
        <>
          <img src={image.url} alt="" draggable={false} />
          {kind !== "npc" && (
            <span className="map-actor-badge">
              <Icon size={11} />
            </span>
          )}
        </>
      ) : (
        <Icon size={14} />
      )}
      {count > 1 && <small>{count}</small>}
    </button>
  );
}
