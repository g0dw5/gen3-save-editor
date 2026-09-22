import { useEffect, useRef, useState } from "react";
import { Copy, Download, FolderOpen, Search, ShieldCheck } from "lucide-react";
import { api, chooseFile, download } from "./api";
import { Floating } from "./components";
import { useI18n } from "./i18n";

type Text = { zh: string; en: string };
type Format = "gameshark_v1_v2";
interface Recipe {
  id: string;
  category: string;
  title: Text;
  summary: Text;
  scope: Text;
  steps: Text[];
  limitations: Text[];
  verification: Text[];
  formats: Format[];
}
interface CheatCatalog {
  rom: { md5: string; label: string; editor_supported: boolean };
  entries: Recipe[];
}
interface Code {
  rom_md5: string;
  cheat_id: string;
  format: Format;
  lines: string[];
  compact_lines: string[];
}
export function CheatWindow({
  editorMd5,
  onClose,
}: {
  editorMd5?: string;
  onClose: () => void;
}) {
  const { t, locale } = useI18n();
  const [catalog, setCatalog] = useState<CheatCatalog | null>(null);
  const [selected, setSelected] = useState("");
  const [search, setSearch] = useState("");
  const [code, setCode] = useState<Code | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<{ code: string; detail: string } | null>(
    null,
  );
  const [copied, setCopied] = useState(false);
  const [compact, setCompact] = useState(false);
  const generation = useRef(0);
  const copyGeneration = useRef(0);
  const loading = useRef(false);
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
    loading.current = false;
    setBusy(!!editorMd5);
    if (editorMd5) {
      api<CheatCatalog>("cheats", { expected_rom_md5: editorMd5 })
        .then((data) => {
          if (token !== generation.current) return;
          setCatalog(data);
          setSelected(data.entries[0]?.id || "");
        })
        .catch((e) => {
          if (token === generation.current) fail(e);
        })
        .finally(() => {
          if (token === generation.current) setBusy(false);
        });
    }
    return () => {
      ++generation.current;
    };
  }, [editorMd5]);
  useEffect(() => {
    let active = true;
    ++copyGeneration.current;
    setCode(null);
    setCopied(false);
    const entry = catalog?.entries.find((entry) => entry.id === selected);
    if (catalog && entry) {
      api<Code>("cheat_code", {
        expected_rom_md5: catalog.rom.md5,
        cheat_id: entry.id,
        format: entry.formats[0],
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
  }, [catalog, selected]);
  const open = async () => {
    if (loading.current) return;
    loading.current = true;
    const token = ++generation.current;
    setBusy(true);
    setError(null);
    setCopied(false);
    try {
      const file = await chooseFile("rom");
      if (!file || token !== generation.current) return;
      // Never leave another ROM's copyable code visible during import or a failed import.
      setCatalog(null);
      setCode(null);
      const data = await api<CheatCatalog>("open_cheat_rom", file);
      if (token !== generation.current) return;
      setCatalog(data);
      setSelected(data.entries[0]?.id || "");
      setSearch("");
    } catch (e) {
      if (token === generation.current) fail(e);
    } finally {
      if (token === generation.current) {
        setBusy(false);
        loading.current = false;
      }
    }
  };
  const entry = catalog?.entries.find((item) => item.id === selected);
  const ready =
    !busy &&
    code &&
    catalog &&
    entry &&
    code.rom_md5 === catalog.rom.md5 &&
    code.cheat_id === entry.id;
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
      "GameShark Advance V1/V2",
      txt(entry.scope),
      lines,
      ...entry.steps.map((s, i) => `${i + 1}. ${txt(s)}`),
      ...entry.limitations.map(txt),
      ...entry.verification.map(txt),
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
            {catalog && (
              <>
                <code>MD5 · {catalog.rom.md5}</code>
                {!catalog.rom.editor_supported && (
                  <p className="cheats-scope">{t("cheatsOnlySupport")}</p>
                )}
              </>
            )}
          </div>
          <button onClick={() => void open()} disabled={busy}>
            <FolderOpen size={15} />
            {t("cheatsOpenRom")}
          </button>
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
            <p>{t("cheatsSupported")}</p>
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
              <article className="cheats-detail">
                <h2>{txt(entry.title)}</h2>
                <p>{txt(entry.summary)}</p>
                <p className="cheats-scope">{txt(entry.scope)}</p>
                <section className="cheats-code">
                  <strong>GameShark Advance V1/V2</strong>
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
                  {ready && (
                    <p className="cheats-count">
                      {t("cheatsLineCount").replace(
                        "{n}",
                        String(code.lines.length),
                      )}
                    </p>
                  )}
                  <pre aria-label={t("cheatsCode")}>
                    {lines || t("working")}
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
                <details className="cheats-evidence">
                  <summary>{t("cheatsEvidence")}</summary>
                  <ul>
                    {entry.verification.map((s, i) => (
                      <li key={i}>{txt(s)}</li>
                    ))}
                  </ul>
                </details>
              </article>
            )}
          </div>
        )}
      </div>
    </Floating>
  );
}
