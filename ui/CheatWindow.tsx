import { useEffect, useRef, useState } from "react";
import { Copy, Download, Search, ShieldCheck } from "lucide-react";
import { api, download } from "./api";
import { SearchSelect } from "./SearchSelect";
import { Floating } from "./components";
import { useI18n } from "./i18n";

type Text = { zh: string; en: string };
type Format = "gameshark_v1_v2" | "codebreaker";
const formatLabel = (format: Format) =>
  format === "codebreaker" ? "CodeBreaker" : "GameShark Advance V1/V2";
type Parameters =
  | { kind: "encounter"; species: number; level: number }
  | { kind: "teleport"; map_id: string; warp_id: number };
interface MapChoice {
  id: string;
  group: number;
  number: number;
  name: string;
  region: number;
  code: string;
  landings: { id: number; x: number; y: number }[];
}
interface Options {
  species: { id: number; name: string }[];
  maps: MapChoice[];
}
interface Recipe {
  parameters?: "encounter" | "teleport" | null;
  id: string;
  category: string;
  title: Text;
  summary: Text;
  scope: Text;
  steps: Text[];
  limitations: Text[];
  formats: Format[];
}
interface CheatCatalog {
  rom: { md5: string; label: string; editor_supported: boolean };
  entries: Recipe[];
  options?: Options;
}
interface Code {
  rom_md5: string;
  cheat_id: string;
  format: Format;
  lines: string[];
  compact_lines: string[];
  parameters?: Parameters | null;
}
export function CheatWindow({
  romMd5,
  onClose,
}: {
  romMd5: string;
  onClose: () => void;
}) {
  const { t, locale } = useI18n();
  const [catalog, setCatalog] = useState<CheatCatalog | null>(null);
  const [selected, setSelected] = useState("");
  const [parameters, setParameters] = useState<Parameters | null>(null);
  const [search, setSearch] = useState("");
  const [code, setCode] = useState<Code | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<{ code: string; detail: string } | null>(
    null,
  );
  const [copied, setCopied] = useState(false);
  const [compact, setCompact] = useState(false);
  const generation = useRef(0);
  const copyGeneration = useRef(0);
  const txt = (value: Text) => value[locale];
  const fail = (e: unknown) => {
    const error = e as { code?: string; detail?: string };
    setError({
      code: error?.code || "operationFailed",
      detail: error?.detail || String(e),
    });
  };
  useEffect(() => {
    const token = ++generation.current;
    setCatalog(null);
    setCode(null);
    setError(null);
    setCopied(false);
    setBusy(true);
    api<CheatCatalog>("cheats", { expected_rom_md5: romMd5 })
      .then((data) => {
        if (token !== generation.current) return;
        setCatalog(data);
        setSelected(data.entries[0]?.id || "");
        setParameters(null);
      })
      .catch((e) => {
        if (token === generation.current) fail(e);
      })
      .finally(() => {
        if (token === generation.current) setBusy(false);
      });
    return () => {
      ++generation.current;
    };
  }, [romMd5]);
  useEffect(() => {
    let active = true;
    ++copyGeneration.current;
    setCode(null);
    setCopied(false);
    const entry = catalog?.entries.find((entry) => entry.id === selected);
    if (
      catalog &&
      entry &&
      (!entry.parameters || parameters?.kind === entry.parameters)
    ) {
      api<Code>("cheat_code", {
        expected_rom_md5: catalog.rom.md5,
        cheat_id: entry.id,
        format: entry.formats[0],
        parameters: entry.parameters ? parameters : null,
      })
        .then((data) => {
          if (active) setCode(data);
        })
        .catch((e) => {
          if (active) fail(e);
        });
    }
    return () => {
      active = false;
      ++copyGeneration.current;
    };
  }, [catalog, selected, parameters]);
  const entry = catalog?.entries.find((item) => item.id === selected);
  const ready =
    !busy &&
    code &&
    catalog &&
    entry &&
    code.rom_md5 === catalog.rom.md5 &&
    code.cheat_id === entry.id &&
    code.format === entry.formats[0] &&
    sameParameters(code.parameters, entry.parameters ? parameters : null) &&
    (!entry.parameters || parameters?.kind === entry.parameters);
  const lines = ready
    ? (compact ? code.compact_lines : code.lines).join("\n")
    : "";
  const copy = async () => {
    if (!ready) return;
    const token = copyGeneration.current;
    try {
      await navigator.clipboard.writeText(lines);
      if (token === copyGeneration.current) setCopied(true);
    } catch (e) {
      if (token === copyGeneration.current) fail(e);
    }
  };
  const exportText = () => {
    if (!ready) return;
    const body = [
      catalog.rom.label,
      `MD5: ${catalog.rom.md5}`,
      txt(entry.title),
      ...(code.parameters
        ? [parameterDescription(code.parameters, catalog.options)]
        : []),
      formatLabel(code.format),
      txt(entry.scope),
      lines,
      ...entry.steps.map((s, i) => `${i + 1}. ${txt(s)}`),
      ...entry.limitations.map(txt),
    ].join("\n\n");
    download(
      `cheats-${catalog.rom.md5.slice(0, 8)}-${entry.id}.txt`,
      body,
      "text/plain;charset=utf-8",
    );
  };
  return (
    <Floating title={t("cheatsTitle")} onClose={onClose} wide>
      <div className="cheats" aria-busy={busy}>
        <div className="cheats-context">
          <div>
            <strong>{catalog?.rom.label || t("cheatsOpenPrompt")}</strong>
            <p>{t("cheatsReadOnly")}</p>
            {catalog && <code>MD5 · {catalog.rom.md5}</code>}
          </div>
        </div>
        {error && (
          <div className="cheats-error" role="alert">
            <strong>{t(error.code)}</strong>
            <details>
              <summary>{t("technicalDetails")}</summary>
              {error.detail}
            </details>
          </div>
        )}
        {busy ? (
          <p className="cheats-empty" role="status">
            {t("working")}
          </p>
        ) : !catalog ? (
          <div className="cheats-empty">
            <h3>{t("cheatsOpenPrompt")}</h3>
          </div>
        ) : !catalog.entries.length ? (
          <div className="cheats-empty">
            <h3>{t("cheatsNone")}</h3>
            <p>{t("cheatsNoneHelp")}</p>
          </div>
        ) : (
          <div className="cheats-layout">
            <aside className="cheats-list">
              <p className="cheats-count">
                {t("cheatsCount").replace(
                  "{n}",
                  String(catalog.entries.length),
                )}
              </p>
              <label className="search-field">
                <Search size={15} />
                <input
                  aria-label={t("cheatsSearch")}
                  placeholder={t("cheatsSearch")}
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                />
              </label>
              {catalog.entries
                .filter((item) =>
                  `${txt(item.title)} ${txt(item.summary)}`
                    .toLowerCase()
                    .includes(search.toLowerCase()),
                )
                .map((item) => (
                  <button
                    key={item.id}
                    className={selected === item.id ? "active" : ""}
                    aria-pressed={selected === item.id}
                    onClick={() => {
                      setSelected(item.id);
                      setParameters(null);
                      setCopied(false);
                      setError(null);
                    }}
                  >
                    <strong>{txt(item.title)}</strong>
                    <span>
                      {t(`cheatsCategory_${item.category || "battle"}`)}
                    </span>
                    <span>
                      <ShieldCheck size={13} />
                      {t("cheatsVerified")}
                    </span>
                  </button>
                ))}
              {!catalog.entries.some((item) =>
                `${txt(item.title)} ${txt(item.summary)}`
                  .toLowerCase()
                  .includes(search.toLowerCase()),
              ) && <p>{t("cheatsNoResults")}</p>}
            </aside>
            {entry && (
              <article
                key={`${catalog.rom.md5}:${entry.id}`}
                className="cheats-detail"
              >
                <h2>{txt(entry.title)}</h2>
                <p>{txt(entry.summary)}</p>
                <p className="cheats-scope">{txt(entry.scope)}</p>
                {entry.parameters && catalog.options && (
                  <ParameterPicker
                    key={`${catalog.rom.md5}:${entry.id}`}
                    kind={entry.parameters}
                    options={catalog.options}
                    onChange={(value) => {
                      setCode(null);
                      setError(null);
                      setParameters(value);
                    }}
                  />
                )}
                <section className="cheats-code">
                  <strong>{formatLabel(entry.formats[0])}</strong>
                  {entry.formats[0] === "gameshark_v1_v2" && (
                    <label>
                      <input
                        type="checkbox"
                        checked={compact}
                        onChange={(e) => {
                          setCompact(e.target.checked);
                          ++copyGeneration.current;
                          setCopied(false);
                        }}
                      />
                      {t("cheatsVba")}
                    </label>
                  )}
                  {ready && (
                    <p className="cheats-count">
                      {t("cheatsLineCount").replace(
                        "{n}",
                        String(code.lines.length),
                      )}
                    </p>
                  )}
                  <pre aria-label={t("cheatsCode")}>
                    {lines ||
                      t(
                        entry.parameters && !parameters
                          ? "cheatsSelectParameters"
                          : "working",
                      )}
                  </pre>
                  <div className="cheats-actions">
                    <button disabled={!ready} onClick={() => void copy()}>
                      <Copy size={14} />
                      {copied ? t("cheatsCopied") : t("cheatsCopy")}
                    </button>
                    <button disabled={!ready} onClick={exportText}>
                      <Download size={14} />
                      {t("cheatsExport")}
                    </button>
                  </div>
                </section>
                <h3>{t("cheatsUsage")}</h3>
                <ol>
                  {entry.steps.map((s, i) => (
                    <li key={i}>{txt(s)}</li>
                  ))}
                </ol>
                <h3>{t("cheatsLimits")}</h3>
                <ul>
                  {entry.limitations.map((s, i) => (
                    <li key={i}>{txt(s)}</li>
                  ))}
                </ul>
              </article>
            )}
          </div>
        )}
      </div>
    </Floating>
  );
}

function parameterDescription(value: Parameters, options?: Options) {
  if (value.kind === "encounter")
    return `#${value.species} ${options?.species.find((s) => s.id === value.species)?.name || ""} · Lv. ${value.level}`;
  const m = options?.maps.find((m) => m.id === value.map_id);
  const w = m?.landings.find((w) => w.id === value.warp_id);
  return `${m?.name || ""} · ${value.map_id} · ${m?.code || ""} · warp ${value.warp_id} (${w?.x}, ${w?.y})`;
}
function ParameterPicker({
  kind,
  options,
  onChange,
}: {
  kind: "encounter" | "teleport";
  options: Options;
  onChange: (value: Parameters | null) => void;
}) {
  const { t } = useI18n();
  const [species, setSpecies] = useState("");
  const [level, setLevel] = useState("5");
  const [region, setRegion] = useState("");
  const [mapId, setMapId] = useState("");
  const [warp, setWarp] = useState("");
  const map = options.maps.find((m) => m.id === mapId);
  const encounter = (id: string, lv: string) => {
    onChange(
      id && /^\d+$/.test(lv) && Number(lv) >= 1 && Number(lv) <= 100
        ? { kind: "encounter", species: Number(id), level: Number(lv) }
        : null,
    );
  };
  const regions = [
    ...new Map(options.maps.map((m) => [m.region, m.name])).entries(),
  ];
  return (
    <section className="cheats-parameters">
      {kind === "encounter" ? (
        <>
          <SearchSelect
            label={t("cheatsSpecies")}
            value={species}
            options={options.species.map((s) => ({
              value: s.id,
              label: `#${s.id} ${s.name}`,
            }))}
            onChange={(id) => {
              setSpecies(id);
              encounter(id, level);
            }}
          />
          <label>
            {t("level")}
            <input
              type="number"
              min={1}
              max={100}
              value={level}
              onChange={(e) => {
                setLevel(e.target.value);
                encounter(species, e.target.value);
              }}
            />
          </label>
        </>
      ) : (
        <>
          <SearchSelect
            label={t("cheatsRegion")}
            value={region}
            options={regions.map(([id, name]) => ({
              value: id,
              label: `${name} · #${id}`,
            }))}
            onChange={(id) => {
              setRegion(id);
              setMapId("");
              setWarp("");
              onChange(null);
            }}
          />
          <SearchSelect
            label={t("cheatsMap")}
            value={mapId}
            disabled={!region}
            options={options.maps
              .filter((m) => String(m.region) === region)
              .map((m) => ({
                value: m.id,
                label: `${m.name} · ${m.id} · ${m.code}${m.landings.length ? "" : ` · ${t("cheatsNoLanding")}`}`,
                disabled: !m.landings.length,
              }))}
            onChange={(id) => {
              setMapId(id);
              const w = options.maps.find((m) => m.id === id)?.landings[0];
              setWarp(w ? String(w.id) : "");
              onChange(
                w ? { kind: "teleport", map_id: id, warp_id: w.id } : null,
              );
            }}
          />
          {map && (
            <>
              <SearchSelect
                label={t("cheatsLanding")}
                value={warp}
                options={map.landings.map((w) => ({
                  value: w.id,
                  label: `#${w.id} · (${w.x}, ${w.y})`,
                }))}
                onChange={(id) => {
                  setWarp(id);
                  onChange({
                    kind: "teleport",
                    map_id: map.id,
                    warp_id: Number(id),
                  });
                }}
              />
              <p className="cheats-map-code">
                {t("cheatsMapCode")} <code>{map.code}</code> · {map.group} /{" "}
                {map.number}
              </p>
            </>
          )}
          <p>{t("cheatsLandingHelp")}</p>
        </>
      )}
    </section>
  );
}

// Rust's JSON object key ordering differs from the order of UI literals.
function sameParameters(a?: Parameters | null, b?: Parameters | null) {
  if (!a || !b) return !a && !b;
  if (a.kind === "encounter" && b.kind === "encounter") {
    return a.species === b.species && a.level === b.level;
  }
  if (a.kind === "teleport" && b.kind === "teleport") {
    return a.map_id === b.map_id && a.warp_id === b.warp_id;
  }
  return false;
}
