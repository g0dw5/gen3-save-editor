import { speciesDisplayName } from "./speciesDisplay";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  BookOpen,
  Box,
  Check,
  ChevronDown,
  Download,
  FolderOpen,
  Grid2X2,
  History,
  Languages,
  PanelRightClose,
  PanelRightOpen,
  Plus,
  Redo2,
  Search,
  ShieldCheck,
  Undo2,
  Wallet,
  X,
} from "lucide-react";
import {
  api,
  confirmAction,
  chooseFile,
  download,
  fromBase64,
  native,
  outputPath,
  type ApiError,
} from "./api";
import {
  Floating,
  NumberField,
  SelectField,
  Sprite,
  Toggle,
} from "./components";
import { I18n, en, zh, type Key, type Locale, useI18n } from "./i18n";
import { romOption, itemOption } from "./names";
import { PokemonEditor, type PokemonEditorTab } from "./PokemonEditor";
import { ReferenceWindow } from "./ReferenceWindow";
import {
  fromKey,
  locationKey,
  type Catalog,
  type Location,
  type RefTab,
  type RefWindow,
  type Snapshot,
  type Template,
  type Trainer,
  type World,
} from "./types";

type Carry =
  | { kind: "move"; from: Location; copy: boolean }
  | { kind: "template"; template: Template };
export default function App() {
  const [locale, setLocale] = useState<Locale>(() =>
    localStorage.getItem("gen3.locale") === "en" ? "en" : "zh",
  );
  const t = useCallback(
    (key: string) => (locale === "zh" ? zh : en)[key as Key] ?? key,
    [locale],
  );
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const canEdit = catalog?.profile.capabilities?.save_edit !== false;
  const [save, setSave] = useState<Snapshot | null>(null);
  const [world, setWorld] = useState<World | null>(null);
  const [selected, setSelected] = useState("p:0");
  const [editorTab, setEditorTab] = useState<PokemonEditorTab>("overview");
  const [multi, setMulti] = useState<string[]>([]);
  const [revision, setRevision] = useState(0);
  const [page, setPage] = useState("pokemon");
  const [windows, setWindows] = useState<RefWindow[]>([]);
  const nextWindow = useRef(0);
  const [free, setFree] = useState(false);
  const [formDirty, setFormDirty] = useState(false);
  const [compact, setCompact] = useState(true);
  const [hideInspector, setHideInspector] = useState(false);
  const [query, setQuery] = useState("");
  const [carry, setCarry] = useState<Carry | null>(null);
  const [draft, setDraft] = useState<{
    location: Location;
    template: Template;
  } | null>(null);
  const [boxSettings, setBoxSettings] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<ApiError | null>(null);
  const [message, setMessage] = useState("");
  const inFlight = useRef(false);
  const worldLoading = useRef(false);
  const worldGeneration = useRef(0);
  const board = useRef<HTMLDivElement>(null);
  const onError = useCallback((e: unknown) => {
    const v = e as ApiError;
    setError(
      v && typeof v === "object" && v.code
        ? v
        : { code: "operationFailed", detail: String(e) },
    );
  }, []);
  const refresh = useCallback((s: Snapshot) => {
    setSave(s);
    setRevision((v) => v + 1);
    setFormDirty(false);
  }, []);
  useEffect(() => {
    api<{ catalog: Catalog | null; save: Snapshot | null }>("state")
      .then((r) => {
        setCatalog(r.catalog);
        setSave(r.save);
      })
      .catch((e) => {
        if (native) onError(e);
      });
  }, [onError]);
  useEffect(() => {
    localStorage.setItem("gen3.locale", locale);
    document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
    document.title = t("app");
    if (native) getCurrentWindow().setTitle(t("app")).catch(onError);
  }, [locale, t, onError]);
  useEffect(() => {
    const beforeUnload = (e: BeforeUnloadEvent) => {
      if (save?.dirty || formDirty || draft) {
        e.preventDefault();
        e.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, [save?.dirty, formDirty, draft]);
  useEffect(() => {
    if (!native) return;
    const win = getCurrentWindow();
    const stop = win.onCloseRequested(async (event) => {
      if (save?.dirty || formDirty || draft) {
        event.preventDefault();
        if (await confirmAction(t("discardSession"))) await win.destroy();
      }
    });
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [save?.dirty, formDirty, draft, t]);
  const guard = useCallback(
    async () => !formDirty || (await confirmAction(t("unsavedPrompt"))),
    [formDirty, t],
  );
  const act = useCallback(
    async (
      action: Record<string, unknown>,
      preserveForm = false,
    ): Promise<Snapshot | null> => {
      if (!canEdit || inFlight.current) return null;
      // Reserve the mutation before yielding: two drops can arrive before React
      // renders disabled slots, and both must not pass an awaited draft guard.
      inFlight.current = true;
      try {
        if (!preserveForm && !(await guard())) return null;
        setBusy(true);
        setError(null);
        const r = await api<{ save: Snapshot }>("action", {
          action,
          policy: free ? "free" : "standard",
        });
        refresh(r.save);
        return r.save;
      } catch (e) {
        onError(e);
        return null;
      } finally {
        inFlight.current = false;
        setBusy(false);
      }
    },
    [canEdit, free, guard, onError, refresh],
  );
  const loadWorld = useCallback(() => {
    if (
      world ||
      worldLoading.current ||
      !catalog ||
      catalog.profile.capabilities?.world === false
    )
      return;
    worldLoading.current = true;
    const generation = worldGeneration.current;
    api<World>("world")
      .then((result) => {
        if (generation === worldGeneration.current) setWorld(result);
      })
      .catch((error) => {
        if (generation === worldGeneration.current) onError(error);
      })
      .finally(() => {
        if (generation === worldGeneration.current)
          worldLoading.current = false;
      });
  }, [catalog, world, onError]);
  const openRef = useCallback(
    (tab: RefTab = "species", id?: number | string) =>
      setWindows((old) => [
        ...old,
        { id: nextWindow.current++, tab, selected: id },
      ]),
    [],
  );
  const byLocation = useMemo(
    () => new Map(save?.pokemon.map((p) => [locationKey(p.location), p])),
    [save],
  );
  const row = byLocation.get(selected);
  const selectedLocation = fromKey(selected);
  const load = async (kind: "rom" | "save") => {
    if (busy || !(await guard())) return;
    if (save?.dirty && !(await confirmAction(t("discardSession")))) return;
    const file = await chooseFile(kind);
    if (!file) return;
    setBusy(true);
    setError(null);
    try {
      if (kind === "rom") {
        worldGeneration.current += 1;
        worldLoading.current = false;
        const r = await api<{ catalog: Catalog }>("open_rom", file);
        setCatalog(r.catalog);
        setSave(null);
        setWorld(null);
        setWindows([]);
        setFree(false);
      } else refresh(await api<Snapshot>("open_save", file));
      setSelected("p:0");
      setMulti([]);
      setCarry(null);
      setDraft(null);
      setFormDirty(false);
      setPage("pokemon");
      setRevision((v) => v + 1);
    } catch (e) {
      onError(e);
    } finally {
      setBusy(false);
    }
  };
  const exportSave = async () => {
    if (!save || busy || !(await guard())) return;
    setBusy(true);
    setError(null);
    try {
      if (native) {
        const path = await outputPath(
          `${catalog?.profile.id}-edited.sav`,
          "sav",
        );
        if (!path) return;
        const r = await api<{ save: Snapshot }>("export_save", { path });
        refresh(r.save);
      } else {
        const r = await api<{ bytes: string }>("save_bytes");
        download(`${catalog?.profile.id}-edited.sav`, fromBase64(r.bytes));
      }
      setMessage(t("saved"));
    } catch (e) {
      onError(e);
    } finally {
      setBusy(false);
    }
  };
  const history = useCallback(
    async (direction: "undo" | "redo") => {
      if (busy || !(await guard())) return;
      setBusy(true);
      try {
        refresh(await api<Snapshot>(direction));
        setDraft(null);
        setCarry(null);
        setMulti([]);
      } catch (e) {
        onError(e);
      } finally {
        setBusy(false);
      }
    },
    [busy, guard, refresh, onError],
  );
  useEffect(() => {
    const keydown = (e: KeyboardEvent) => {
      if (
        (e.metaKey || e.ctrlKey) &&
        e.key.toLowerCase() === "z" &&
        !(
          e.target instanceof HTMLInputElement ||
          e.target instanceof HTMLTextAreaElement
        )
      ) {
        e.preventDefault();
        void history(e.shiftKey ? "redo" : "undo");
      }
      if (e.key === "Escape") {
        setCarry(null);
      }
    };
    document.addEventListener("keydown", keydown);
    return () => document.removeEventListener("keydown", keydown);
  }, [history]);
  const receive = async (location: Location, payload: Carry) => {
    if (busy || !(await guard())) return;
    if (payload.kind === "template") {
      if (byLocation.has(locationKey(location))) {
        onError({ code: "occupied_slot", detail: "" });
        return;
      }
      setDraft({ location, template: payload.template });
      setSelected(locationKey(location));
      setHideInspector(false);
      setCarry(null);
      setFormDirty(false);
      setMulti([]);
      return;
    }
    const result = await act({
      type: "transfer",
      from: payload.from,
      to: location,
      copy: payload.copy,
    });
    if (result) {
      const dest =
        location.kind === "party" && !byLocation.has(locationKey(location))
          ? {
              kind: "party" as const,
              slot:
                result.pokemon.filter((p) => p.location.kind === "party")
                  .length - 1,
            }
          : location;
      setSelected(locationKey(dest));
      setCarry(null);
      setDraft(null);
      setMulti([]);
    }
  };
  const chooseSlot = async (location: Location, modifier = false) => {
    if (carry) {
      void receive(location, carry);
      return;
    }
    if (!(await guard())) return;
    const key = locationKey(location);
    setSelected(key);
    setDraft(null);
    setFormDirty(false);
    setHideInspector(false);
    setMulti((old) =>
      modifier && byLocation.has(key)
        ? old.includes(key)
          ? old.filter((k) => k !== key)
          : [...new Set([...old, key])]
        : [],
    );
  };
  const slot = (location: Location) => {
    const key = locationKey(location);
    const record = byLocation.get(key);
    const mon = record?.pokemon;
    const isDraft = draft && locationKey(draft.location) === key;
    const species = mon?.species ?? (isDraft ? draft.template.species : 0);
    const speciesName =
      catalog && species
        ? speciesDisplayName(catalog, species, t, mon?.pid)
        : "";
    const match =
      !query ||
      `${mon?.nickname ?? ""} ${speciesName} ${species}`
        .toLocaleLowerCase()
        .includes(query.toLocaleLowerCase());
    const label = `${location.kind === "party" ? t("party") : `${t("box")} ${location.box_index + 1}`} · ${location.slot + 1} · ${species ? `${mon?.nickname || speciesName} Lv. ${mon?.level ?? draft?.template.level}` : t("empty")}`;
    return (
      <button
        type="button"
        key={key}
        data-location={key}
        aria-label={label}
        aria-pressed={selected === key}
        className={`storage-slot ${species ? "occupied" : ""} ${selected === key ? "selected" : ""} ${multi.includes(key) ? "multi-selected" : ""} ${isDraft ? "draft-slot" : ""} ${!match ? "dimmed" : ""}`}
        disabled={busy}
        draggable={!!mon && canEdit}
        title={speciesName || label}
        onClick={(e) =>
          chooseSlot(location, e.ctrlKey || e.metaKey || e.shiftKey)
        }
        onDragStart={(e) => {
          if (!canEdit || formDirty) {
            e.preventDefault();
            onError({ code: "unsavedPrompt", detail: "" });
            return;
          }
          e.dataTransfer.setData(
            "application/x-gen3",
            JSON.stringify({
              kind: "move",
              profile: catalog?.profile.md5,
              from: location,
              copy: e.altKey,
            }),
          );
          e.dataTransfer.effectAllowed = "copyMove";
        }}
        onDragOver={(e) => {
          if (canEdit && e.dataTransfer.types.includes("application/x-gen3")) {
            e.preventDefault();
            e.currentTarget.classList.add("drop-target");
          }
        }}
        onDragLeave={(e) => e.currentTarget.classList.remove("drop-target")}
        onDrop={(e) => {
          e.preventDefault();
          e.currentTarget.classList.remove("drop-target");
          if (!canEdit) return;
          try {
            const payload = JSON.parse(
              e.dataTransfer.getData("application/x-gen3"),
            );
            if (payload.profile !== catalog?.profile.md5)
              throw { code: "profile_mismatch", detail: "" };
            void receive(location, payload);
          } catch (e) {
            onError(e);
          }
        }}
      >
        {species && catalog ? (
          <Sprite
            catalog={catalog}
            species={species}
            shiny={mon?.shiny}
            pid={mon?.pid}
          />
        ) : (
          <span className="slot-empty" />
        )}
        {location.kind === "party" && (
          <span className="party-caption">
            {species ? (
              <>
                {speciesName}
                <small>
                  Lv. {mon?.level ?? draft?.template.level}
                  {mon?.shiny ? " ✦" : ""}
                </small>
              </>
            ) : (
              <small>{location.slot + 1}</small>
            )}
          </span>
        )}
        {multi.includes(key) && <Check size={11} className="selection-check" />}
      </button>
    );
  };
  const exportPokemon = async () => {
    if (!row) return;
    try {
      const path = native
        ? await outputPath(
            `${row.pokemon.nickname || row.pokemon.species}.gen3.json`,
            "json",
          )
        : null;
      if (native && !path) return;
      const file = await api("export_pokemon", {
        location: row.location,
        ...(path ? { path } : {}),
      });
      if (!native)
        download(
          `${row.pokemon.species}.gen3.json`,
          JSON.stringify(file, null, 2),
          "application/json",
        );
    } catch (e) {
      onError(e);
    }
  };
  const importPokemon = async () => {
    if (!(await guard())) return;
    const file = await chooseFile("pokemon");
    if (!file) return;
    try {
      refresh(
        await api<Snapshot>("import_pokemon", {
          ...file,
          location: selectedLocation,
        }),
      );
    } catch (e) {
      onError(e);
    }
  };
  return (
    <I18n.Provider value={{ locale, t }}>
      <div className="app-shell">
        <header className="app-header">
          <div className="brand">
            <span className="brand-mark">
              <Box size={22} />
            </span>
            <div>
              <strong>{t("brandName")}</strong>
              <span>{t("brandRole")}</span>
            </div>
          </div>
          <div className="file-context">
            {catalog ? (
              <>
                <strong>{catalog.profile.label}</strong>
                <span>
                  <ShieldCheck size={12} />
                  {t("verified")} · {catalog.profile.md5.slice(0, 8)}
                  {save?.dirty ? ` · ${t("unsaved")}` : ""}
                </span>
              </>
            ) : (
              <span>{t("local")}</span>
            )}
          </div>
          <div className="header-actions">
            <button onClick={() => void load("rom")} disabled={busy}>
              <FolderOpen size={15} />
              {t("openRom")}
            </button>
            <button
              onClick={() => void load("save")}
              disabled={!catalog || busy}
            >
              <FolderOpen size={15} />
              {t("openSave")}
            </button>
            <button onClick={() => openRef()} disabled={!catalog}>
              <BookOpen size={15} />
              {t("references")}
            </button>
            <button
              className="primary"
              disabled={!save || busy || !canEdit}
              onClick={() => void exportSave()}
            >
              <Download size={15} />
              {t("exportSave")}
            </button>
            <button
              className="icon-button"
              aria-label={locale === "zh" ? "English" : "简体中文"}
              title={t("languageHelp")}
              onClick={() => setLocale((l) => (l === "zh" ? "en" : "zh"))}
            >
              <Languages size={18} />
            </button>
          </div>
        </header>
        {error && (
          <div className="error-banner" role="alert">
            <div>
              <strong>
                {t(error.code) === error.code
                  ? t("operationFailed")
                  : t(error.code)}
              </strong>
              {error.detail && (
                <details>
                  <summary>{t("technicalDetails")}</summary>
                  <code>
                    {error.code}: {error.detail}
                  </code>
                </details>
              )}
            </div>
            <button
              className="icon-button"
              onClick={() => setError(null)}
              aria-label={t("close")}
            >
              <X size={17} />
            </button>
          </div>
        )}
        {!catalog ? (
          <main className="welcome">
            <div className="welcome-symbol">
              <Grid2X2 size={52} strokeWidth={1} />
            </div>
            <span className="eyebrow">POKÉMON · GENERATION III</span>
            <h1>{t("welcome")}</h1>
            <p>{t("intro")}</p>
            <button
              className="primary large-button"
              onClick={() => void load("rom")}
              disabled={busy}
            >
              <FolderOpen size={18} />
              {t(busy ? "loading" : "openRom")}
            </button>
            <div className="supported-list">
              <strong>{t("supported")}</strong>
              <div>
                5.0EX+BW <code>0d9b129f7dd76895f79bb47ad7dec2fe</code>
              </div>
              <div>
                5.0EX+DP <code>cb2940215f4dafb1bef133c3af379f44</code>
              </div>
              <div>
                {t("rocketVersion")}{" "}
                <code>59c658a1081f542086de1060bb65f0b3</code>
              </div>
            </div>
          </main>
        ) : (
          <>
            <div className="workspace-toolbar">
              <nav>
                {[
                  ["pokemon", Grid2X2],
                  ["bag", Wallet],
                  ["player", ShieldCheck],
                  ["pokedex", BookOpen],
                  ["changes", History],
                ]
                  .filter(
                    ([key]) =>
                      (key !== "pokedex" ||
                        catalog.profile.capabilities?.dex !== false) &&
                      (canEdit ||
                        !["player", "changes"].includes(key as string)),
                  )
                  .map(([key, Icon]) => {
                    const C = Icon as typeof Grid2X2;
                    return (
                      <button
                        key={key as string}
                        className={page === key ? "active" : ""}
                        disabled={busy}
                        onClick={async () => {
                          if (await guard()) {
                            setPage(key as string);
                            setFormDirty(false);
                          }
                        }}
                      >
                        <C size={15} />
                        {t(key === "bag" ? "inventory" : (key as string))}
                        {key === "changes" && !!save?.changes.length && (
                          <span className="count">{save.changes.length}</span>
                        )}
                      </button>
                    );
                  })}
              </nav>
              <div className="toolbar-actions">
                <button
                  className="icon-button"
                  disabled={!save?.can_undo || busy}
                  aria-label={t("undo")}
                  title={`${t("undo")} ⌘Z`}
                  onClick={() => void history("undo")}
                >
                  <Undo2 size={17} />
                </button>
                <button
                  className="icon-button"
                  disabled={!save?.can_redo || busy}
                  aria-label={t("redo")}
                  onClick={() => void history("redo")}
                >
                  <Redo2 size={17} />
                </button>
                {page === "pokemon" && (
                  <>
                    <button onClick={() => setCompact((c) => !c)}>
                      {t(compact ? "compact" : "comfortable")}
                      <ChevronDown size={13} />
                    </button>
                    <button
                      className="icon-button"
                      aria-label={t("inspector")}
                      onClick={() => setHideInspector((v) => !v)}
                    >
                      {hideInspector ? (
                        <PanelRightOpen size={18} />
                      ) : (
                        <PanelRightClose size={18} />
                      )}
                    </button>
                  </>
                )}
              </div>
            </div>
            {!save ? (
              <main className="empty-workspace">
                <Box size={42} strokeWidth={1} />
                <h2>{t("noSave")}</h2>
                <p>{catalog.profile.label}</p>
                <div className="button-row">
                  <button className="primary" onClick={() => void load("save")}>
                    {t("openSave")}
                  </button>
                  <button onClick={() => openRef()}>{t("references")} ↗</button>
                </div>
              </main>
            ) : page === "pokemon" ? (
              <div
                className={`storage-workspace ${hideInspector ? "inspector-hidden" : ""}`}
              >
                <div className="storage-main">
                  <div className="storage-heading">
                    <div>
                      <h1>{t("boxes")}</h1>
                      <span className="muted small">
                        {save.pokemon.length} Pokémon · {save.boxes.length}{" "}
                        {t("box")}
                      </span>
                    </div>
                    <label className="search-field">
                      <Search size={16} />
                      <input
                        placeholder={t("search")}
                        aria-label={t("search")}
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                      />
                    </label>
                  </div>
                  {carry && (
                    <div className="carry-banner" role="status">
                      <span>
                        {t(
                          carry.kind === "template"
                            ? "destination"
                            : "moveDestination",
                        )}
                      </span>
                      <button
                        className="icon-button"
                        onClick={() => setCarry(null)}
                        aria-label={t("cancel")}
                      >
                        <X size={15} />
                      </button>
                    </div>
                  )}
                  {!!multi.length && (
                    <div className="multi-banner">
                      <span>
                        {t("selection")} {multi.length}
                      </span>
                      <button
                        className="link-button"
                        onClick={() => setMulti([])}
                      >
                        {t("clearSelection")}
                      </button>
                    </div>
                  )}
                  <div
                    className="storage-board"
                    ref={board}
                    onDragOver={(e) => {
                      if (!board.current) return;
                      const rect = board.current.getBoundingClientRect();
                      if (e.clientY > rect.bottom - 65)
                        board.current.scrollTop += 14;
                      if (e.clientY < rect.top + 65)
                        board.current.scrollTop -= 14;
                    }}
                  >
                    <section className="party-panel">
                      <header>
                        <strong>{t("party")}</strong>
                        <span>
                          {
                            save.pokemon.filter(
                              (p) => p.location.kind === "party",
                            ).length
                          }
                          /6
                        </span>
                      </header>
                      <div className="party-slots">
                        {Array.from({ length: 6 }, (_, slotIndex) =>
                          slot({ kind: "party", slot: slotIndex }),
                        )}
                      </div>
                    </section>
                    <div className={`all-boxes ${compact ? "compact" : ""}`}>
                      {save.boxes.map((box) => (
                        <section className="box-panel" key={box.index}>
                          <header>
                            <button
                              className="box-title"
                              disabled={!canEdit}
                              onClick={() => setBoxSettings(box.index)}
                            >
                              {box.name.trim() ||
                                `${t("box")} ${String(box.index + 1).padStart(2, "0")}`}
                            </button>
                            <span>{box.count}/30</span>
                          </header>
                          <div className="box-slots">
                            {Array.from({ length: 30 }, (_, slotIndex) =>
                              slot({
                                kind: "box",
                                box_index: box.index,
                                slot: slotIndex,
                              }),
                            )}
                          </div>
                        </section>
                      ))}
                    </div>
                  </div>
                </div>
                {!hideInspector && (
                  <aside className="inspector">
                    {draft ? (
                      <Draft
                        key={locationKey(draft.location)}
                        template={draft.template}
                        catalog={catalog}
                        onCancel={() => setDraft(null)}
                        onCreate={async (template) => {
                          const result = await act({
                            type: "create",
                            location: draft.location,
                            ...template,
                          });
                          if (result) {
                            const actual =
                              draft.location.kind === "party"
                                ? {
                                    kind: "party" as const,
                                    slot:
                                      result.pokemon.filter(
                                        (p) => p.location.kind === "party",
                                      ).length - 1,
                                  }
                                : draft.location;
                            setSelected(locationKey(actual));
                            setDraft(null);
                          }
                        }}
                      />
                    ) : row ? (
                      <PokemonEditor
                        key={`${catalog.profile.md5}:${selected}`}
                        row={row}
                        tab={editorTab}
                        onTabChange={setEditorTab}
                        catalog={catalog}
                        free={free}
                        setFree={setFree}
                        onDirty={setFormDirty}
                        batchLocations={multi.map(fromKey)}
                        onApply={async (patch) =>
                          !!(await act(
                            multi.length > 1
                              ? {
                                  type: "batch",
                                  locations: multi.map(fromKey),
                                  patch,
                                }
                              : {
                                  type: "pokemon",
                                  location: row.location,
                                  patch,
                                },
                            true,
                          ))
                        }
                        onReference={(id) => openRef("species", id)}
                        onTransfer={async (copy) => {
                          if (await guard())
                            setCarry({
                              kind: "move",
                              from: row.location,
                              copy,
                            });
                        }}
                        onDelete={async () => {
                          if (await confirmAction(t("deletePrompt")))
                            void act({
                              type: "delete",
                              location: row.location,
                            });
                        }}
                        onExport={() => void exportPokemon()}
                      />
                    ) : (
                      <div className="empty-inspector">
                        <Plus size={34} strokeWidth={1} />
                        <h2>{t("empty")}</h2>
                        <p>{t("noSelection")}</p>
                        <button
                          className="primary"
                          disabled={!canEdit}
                          onClick={() =>
                            setDraft({
                              location: selectedLocation,
                              template: { species: 1, level: 5 },
                            })
                          }
                        >
                          {t("createHere")}
                        </button>
                        <button
                          disabled={!canEdit}
                          onClick={() => void importPokemon()}
                        >
                          {t("importPokemon")}
                        </button>
                      </div>
                    )}
                  </aside>
                )}
              </div>
            ) : page === "bag" ? (
              <BagEditor
                key={catalog.profile.md5}
                catalog={catalog}
                save={save}
                free={free}
                setFree={setFree}
                act={act}
                onDirty={setFormDirty}
              />
            ) : page === "player" ? (
              <TrainerEditor
                key={revision}
                trainer={save.trainer}
                act={act}
                onDirty={setFormDirty}
              />
            ) : page === "pokedex" ? (
              <DexEditor save={save} catalog={catalog} act={act} />
            ) : (
              <Changes save={save} />
            )}
          </>
        )}
        <footer className="statusbar">
          <span aria-live="polite">
            {busy ? t("working") : message || t("local")}
          </span>
          <span>
            {save && `${save.active_slot === 0 ? "A" : "B"} · #${save.counter}`}
            {save && !save.backup_valid && (
              <span className="warning-text"> · {t("damagedBackup")}</span>
            )}
          </span>
        </footer>
        {catalog &&
          windows.map((info) => (
            <ReferenceWindow
              key={`${catalog.profile.md5}:${info.id}`}
              window={info}
              catalog={catalog}
              world={world}
              save={save}
              loadWorld={loadWorld}
              onClose={() =>
                setWindows((old) => old.filter((w) => w.id !== info.id))
              }
              onTemplate={async (template) => {
                if (!save) {
                  onError({ code: "no_save", detail: "" });
                  return;
                }
                if (await guard()) {
                  setCarry({ kind: "template", template });
                  setPage("pokemon");
                }
              }}
              onError={onError}
            />
          ))}
        {boxSettings !== null && save && (
          <BoxSettings
            box={save.boxes[boxSettings]}
            onClose={() => setBoxSettings(null)}
            onSave={async (name, wallpaper) => {
              if (
                await act({ type: "box", index: boxSettings, name, wallpaper })
              )
                setBoxSettings(null);
            }}
            onSort={async () => {
              if (await act({ type: "sort", index: boxSettings }))
                setBoxSettings(null);
            }}
          />
        )}
      </div>
    </I18n.Provider>
  );
}

function Draft({
  template,
  catalog,
  onCancel,
  onCreate,
}: {
  template: Template;
  catalog: Catalog;
  onCancel: () => void;
  onCreate: (template: Template) => Promise<void>;
}) {
  const { t } = useI18n();
  const [value, setValue] = useState(template);
  return (
    <form
      className="draft-editor"
      onSubmit={(e) => {
        e.preventDefault();
        void onCreate(value);
      }}
    >
      <span className="eyebrow amber">{t("draft")}</span>
      <Sprite catalog={catalog} species={value.species} large />
      <h2>{speciesDisplayName(catalog, value.species, t)}</h2>
      <p className="muted">{t("draftHelp")}</p>
      <SelectField
        searchable
        label={t("species")}
        value={value.species}
        onChange={(v) => setValue((old) => ({ ...old, species: +v }))}
        options={catalog.species
          .filter((s) => s.stats[0])
          .map((s) => romOption(s))}
      />
      <NumberField
        label={t("level")}
        value={value.level}
        min={1}
        max={catalog.profile.max_level ?? 100}
        onChange={(level) => setValue((old) => ({ ...old, level }))}
      />
      <NumberField
        label={t("met_location")}
        value={value.met_location ?? 0}
        onChange={(met_location) =>
          setValue((old) => ({ ...old, met_location }))
        }
      />
      <Toggle
        label={t("egg")}
        checked={value.egg ?? false}
        onChange={(egg) => setValue((old) => ({ ...old, egg }))}
      />
      <div className="button-row">
        <button className="primary" type="submit">
          {t("create")}
        </button>
        <button type="button" onClick={onCancel}>
          {t("cancel")}
        </button>
      </div>
    </form>
  );
}
type Act = (
  action: Record<string, unknown>,
  preserveForm?: boolean,
) => Promise<Snapshot | null>;
function BagEditor({
  catalog,
  save,
  free,
  setFree,
  act,
  onDirty,
}: {
  catalog: Catalog;
  save: Snapshot;
  free: boolean;
  setFree: (v: boolean) => void;
  act: Act;
  onDirty: (v: boolean) => void;
}) {
  const { t } = useI18n();
  const [pocket, setPocket] = useState("items");
  const [selectedSlot, setSelectedSlot] = useState(0);
  const [saving, setSaving] = useState(false);
  const entries = save.bag.filter((e) => e.pocket === pocket);
  const selected = entries.find((e) => e.slot === selectedSlot);
  const [item, setItem] = useState(0);
  const [quantity, setQuantity] = useState(1);
  useEffect(() => {
    setItem(selected?.item ?? 0);
    setQuantity(selected?.quantity || 1);
  }, [selected]);
  const dirty =
    !!selected &&
    (item !== selected.item || quantity !== (selected.quantity || 1));
  useEffect(() => onDirty(dirty), [dirty, onDirty]);
  const guardBag = async () =>
    !dirty || (await confirmAction(t("unsavedPrompt")));
  return (
    <main className="data-page">
      <div className="page-heading">
        <h1>{t("inventory")}</h1>
        {catalog.profile.capabilities?.save_edit !== false && (
          <Toggle label={t("free")} checked={free} onChange={setFree} />
        )}
      </div>
      <div className="pocket-tabs">
        {[...new Set(save.bag.map((e) => e.pocket))].map((k) => (
          <button
            key={k}
            className={k === pocket ? "active" : ""}
            aria-pressed={k === pocket}
            disabled={saving}
            onClick={async () => {
              if (await guardBag()) {
                setPocket(k);
                setSelectedSlot(0);
                const first = save.bag.find(
                  (e) => e.pocket === k && e.slot === 0,
                );
                setItem(first?.item ?? 0);
                setQuantity(first?.quantity || 1);
              }
            }}
          >
            {t(k === "items" ? "bagItems" : k)}
          </button>
        ))}
      </div>
      <p className="small muted">{t("inventoryHelp")}</p>
      <div className="bag-layout">
        <table className="data-table">
          <thead>
            <tr>
              <th>#</th>
              <th>{t("name")}</th>
              <th>{t("quantity")}</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {entries.map((e) => (
              <tr
                className={selected?.slot === e.slot ? "selected" : ""}
                key={e.slot}
              >
                <td>{e.slot + 1}</td>
                <td>
                  {e.item ? (
                    `${catalog.items[e.item]?.name ?? e.item}${catalog.items[e.item]?.tm_move ? ` · ${catalog.moves[catalog.items[e.item].tm_move!]?.name}` : ""}`
                  ) : (
                    <span className="muted">{t("empty")}</span>
                  )}
                </td>
                <td>{e.quantity || "—"}</td>
                <td>
                  <button
                    disabled={saving}
                    onClick={async () => {
                      if (await guardBag()) {
                        setSelectedSlot(e.slot);
                        setItem(e.item);
                        setQuantity(e.quantity || 1);
                      }
                    }}
                  >
                    {t("details")}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <aside>
          {selected && (
            <form
              onSubmit={async (e) => {
                e.preventDefault();
                if (saving) return;
                setSaving(true);
                try {
                  await act(
                    {
                      type: "bag",
                      pocket,
                      slot: selected.slot,
                      item,
                      quantity,
                    },
                    true,
                  );
                } finally {
                  setSaving(false);
                }
              }}
            >
              <fieldset
                className="inventory-fields"
                disabled={
                  saving || catalog.profile.capabilities?.save_edit === false
                }
              >
                <h2>
                  {t(pocket === "items" ? "bagItems" : pocket)} ·{" "}
                  {selected.slot + 1}
                </h2>
                <SelectField
                  searchable
                  label={t("items")}
                  value={item}
                  onChange={(v) => setItem(+v)}
                  options={catalog.items.map((i) =>
                    i.id
                      ? itemOption(catalog, i)
                      : { value: 0, label: t("emptyMove") },
                  )}
                />
                <NumberField
                  label={t("quantity")}
                  value={quantity}
                  onChange={setQuantity}
                  max={
                    free
                      ? 65535
                      : pocket === "key_items"
                        ? 1
                        : ["pc", "tmhm", "berries"].includes(pocket)
                          ? 999
                          : 99
                  }
                  min={1}
                  disabled={!item}
                />
                <div className="inventory-actions">
                  <button className="primary" type="submit" disabled={!dirty}>
                    {t("apply")}
                  </button>
                  <button
                    type="button"
                    disabled={!item}
                    onClick={() => {
                      setItem(0);
                      setQuantity(1);
                    }}
                  >
                    {t("clearSlot")}
                  </button>
                </div>
              </fieldset>
            </form>
          )}
        </aside>
      </div>
    </main>
  );
}
function TrainerEditor({
  trainer,
  act,
  onDirty,
}: {
  trainer: Trainer;
  act: Act;
  onDirty: (v: boolean) => void;
}) {
  const { t } = useI18n();
  const [patch, setPatch] = useState<Partial<Trainer>>({});
  const data = { ...trainer, ...patch };
  useEffect(() => onDirty(!!Object.keys(patch).length), [patch, onDirty]);
  return (
    <main className="data-page">
      <h1>{t("player")}</h1>
      <form
        className="trainer-form"
        onSubmit={async (e) => {
          e.preventDefault();
          await act({ type: "trainer", patch }, true);
        }}
      >
        <label className="field">
          <span>{t("name")}</span>
          <input
            value={data.name}
            onChange={(e) =>
              setPatch((old) => ({ ...old, name: e.target.value }))
            }
          />
        </label>
        <div className="field-grid">
          {(
            [
              "gender",
              "tid",
              "sid",
              "money",
              "coins",
              "hours",
              "minutes",
              "seconds",
              "registered_item",
            ] as const
          ).map((key) => (
            <NumberField
              key={key}
              label={t(key)}
              value={data[key]}
              onChange={(v) => setPatch((old) => ({ ...old, [key]: v }))}
              max={
                key === "money"
                  ? 999999
                  : key === "coins"
                    ? 9999
                    : key === "gender"
                      ? 1
                      : key === "minutes" || key === "seconds"
                        ? 59
                        : 65535
              }
            />
          ))}
        </div>
        <button
          className="primary"
          type="submit"
          disabled={!Object.keys(patch).length}
        >
          {t("apply")}
        </button>
      </form>
    </main>
  );
}
function DexEditor({
  save,
  catalog,
  act,
}: {
  save: Snapshot;
  catalog: Catalog;
  act: Act;
}) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  return (
    <main className="data-page">
      <h1>{t("pokedex")}</h1>
      <p className="muted">{t("dexHelp")}</p>
      <label className="field narrow">
        <span>{t("number")}</span>
        <input value={query} onChange={(e) => setQuery(e.target.value)} />
      </label>
      <div className="dex-flags">
        {save.dex
          .filter(
            (d) =>
              !query ||
              `${d.number} ${catalog.species.find((s) => s.dex_number === d.number)?.name ?? ""}`.includes(
                query,
              ),
          )
          .map((d) => (
            <div key={d.number}>
              <strong>
                #{d.number} ·{" "}
                {catalog.species.find((s) => s.dex_number === d.number)?.name ??
                  t("unknown")}
              </strong>
              <Toggle
                label={t("seen")}
                checked={d.seen}
                onChange={(seen) => {
                  void act({
                    type: "dex",
                    number: d.number,
                    seen,
                    owned: seen ? d.owned : false,
                  });
                }}
              />
              <Toggle
                label={t("owned")}
                checked={d.owned}
                onChange={(owned) => {
                  void act({
                    type: "dex",
                    number: d.number,
                    seen: d.seen,
                    owned,
                  });
                }}
              />
            </div>
          ))}
      </div>
    </main>
  );
}
function Changes({ save }: { save: Snapshot }) {
  const { t } = useI18n();
  return (
    <main className="data-page">
      <h1>{t("changes")}</h1>
      {!save.changes.length && <p className="muted">{t("noChanges")}</p>}
      {save.changes.map((change, i) => (
        <article className="change-card" key={i}>
          <header>
            <strong>
              {i + 1}. {t(String(change.action.type))}
            </strong>
            <span>
              {change.bytes_changed} {t("bytesChanged")}
            </span>
          </header>
          {change.findings.map((f, i) => (
            <p className="warning-text small" key={i}>
              {t(f.code)} · {t(f.field)}
            </p>
          ))}
          <details>
            <summary>{t("showDiff")}</summary>
            <table className="data-table">
              <thead>
                <tr>
                  <th>{t("field")}</th>
                  <th>{t("before")}</th>
                  <th>{t("after")}</th>
                </tr>
              </thead>
              <tbody>
                {change.fields.map((f) => (
                  <tr key={f.path}>
                    <td>
                      {f.path
                        .split("/")
                        .filter(Boolean)
                        .map((k) => t(k))
                        .join(" / ")}
                    </td>
                    <td>
                      <code>{JSON.stringify(f.before)}</code>
                    </td>
                    <td>
                      <code>{JSON.stringify(f.after)}</code>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </details>
        </article>
      ))}
    </main>
  );
}
function BoxSettings({
  box,
  onClose,
  onSave,
  onSort,
}: {
  box: { index: number; name: string; wallpaper: number };
  onClose: () => void;
  onSave: (name: string, wallpaper: number) => Promise<void>;
  onSort: () => Promise<void>;
}) {
  const { t } = useI18n();
  const [name, setName] = useState(box.name);
  const [wallpaper, setWallpaper] = useState(box.wallpaper);
  return (
    <Floating title={`${t("box")} ${box.index + 1}`} onClose={onClose}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void onSave(name, wallpaper);
        }}
      >
        <label className="field">
          <span>{t("renameBox")}</span>
          <input value={name} onChange={(e) => setName(e.target.value)} />
        </label>
        <NumberField
          label={t("wallpaper")}
          value={wallpaper}
          onChange={setWallpaper}
          max={15}
        />
        <div className="button-row">
          <button className="primary" type="submit">
            {t("apply")}
          </button>
          <button type="button" onClick={() => void onSort()}>
            {t("sort")}
          </button>
        </div>
      </form>
    </Floating>
  );
}
