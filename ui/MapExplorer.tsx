import { useEffect, useMemo, useState } from "react";
import { Gift, MapPin, Search, Sparkles, Users, CircleDot } from "lucide-react";
import { useI18n } from "./i18n";
import { useRomCharacterImage } from "./romCharacterImage";
import type { Catalog, GameMap, MapEventReport, MapMarker } from "./types";

const layers = ["pickup", "hidden", "gift", "npc"] as const;
const icons = { pickup: CircleDot, hidden: Sparkles, gift: Gift, npc: Users };
export function MapExplorer({
  map,
  image,
  report,
  catalog,
}: {
  map: GameMap;
  image: string;
  report?: MapEventReport;
  catalog: Catalog;
}) {
  const { t } = useI18n();
  const [enabled, setEnabled] = useState({
    pickup: true,
    hidden: true,
    gift: true,
    npc: true,
  });
  const [grid, setGrid] = useState(false);
  const [zoom, setZoom] = useState(1);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  useEffect(() => setSelected(null), [map.id]);
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
      ...new Set(
        marker.rewards.map((r) => items.get(r.item)?.name ?? `#${r.item}`),
      ),
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
      </div>
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
                onClick={() => setSelected(key)}
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
              <p className="small muted">
                {t("mapElevation")} {marker.elevation}
              </p>
              {marker.rewards.length ? (
                [
                  ...new Set(
                    marker.rewards.map(
                      (r) =>
                        `${items.get(r.item)?.name ?? `#${r.item}`} × ${r.quantity ?? "?"}`,
                    ),
                  ),
                ].map((name) => <p key={name}>{name}</p>)
              ) : (
                <p>{t("mapNoReward")}</p>
              )}
              {marker.rewards.some((r) => r.conditions.length > 0) && (
                <p className="small muted">{t("mapConditionalReward")}</p>
              )}
              {marker.movement_type === 76 && (
                <p className="small muted">{t("mapInvisibleObject")}</p>
              )}
              {marker.stopped_at.length > 0 && (
                <p className="small muted">{t("mapPartialScript")}</p>
              )}
              <details>
                <summary>{t("mapEventEvidence")}</summary>
                <code>
                  {t("mapEventOffset")} 0x{marker.offset.toString(16)}
                  {marker.script !== null
                    ? ` · Script 0x${marker.script.toString(16)}`
                    : ""}
                  {marker.flag !== null
                    ? ` · Flag 0x${marker.flag.toString(16)}`
                    : ""}
                </code>
              </details>
            </div>
          ))
        ) : (
          <p className="muted">{t("mapSelectMarker")}</p>
        )}
      </div>
      {!!report?.unplaced_rewards.length && (
        <details>
          <summary>{t("mapUnplacedRewards")}</summary>
          <p className="small muted">{t("mapUnplacedHelp")}</p>
          {[
            ...new Set(
              report.unplaced_rewards.map(
                (r) => items.get(r.item)?.name ?? `#${r.item}`,
              ),
            ),
          ].map((name) => (
            <p key={name}>{name}</p>
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
