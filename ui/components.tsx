import { useEffect, useRef, useState, type ReactNode } from "react";
import { X, GripHorizontal } from "lucide-react";
import { api } from "./api";
import { useI18n, typeNames } from "./i18n";
import type { Catalog } from "./types";

const sprites = new Map<string, Promise<string>>();
export function Sprite({
  catalog,
  species,
  shiny = false,
  large = false,
}: {
  catalog: Catalog;
  species: number;
  shiny?: boolean;
  large?: boolean;
}) {
  const [url, setUrl] = useState("");
  useEffect(() => {
    let active = true;
    setUrl("");
    if (!species) return;
    const key = `${catalog.profile.md5}:${species}:${shiny}`;
    if (!sprites.has(key))
      sprites.set(
        key,
        api<{ url: string }>("sprite", { id: species, shiny })
          .then((r) => r.url)
          .catch(() => ""),
      );
    sprites.get(key)!.then((url) => {
      if (active) setUrl(url);
    });
    return () => {
      active = false;
    };
  }, [catalog.profile.md5, species, shiny]);
  return url ? (
    <img
      className={`sprite ${large ? "large" : ""}`}
      src={url}
      alt=""
      draggable={false}
    />
  ) : (
    <span
      className={`sprite fallback ${large ? "large" : ""}`}
      aria-hidden="true"
    >
      {species ? "·" : ""}
    </span>
  );
}
export function Types({ values }: { values: number[] }) {
  const { locale } = useI18n();
  return (
    <span className="type-tags">
      {[...new Set(values)].map((v) => (
        <span className={`type-tag type-${v}`} key={v}>
          {typeNames[locale][v] ?? `#${v}`}
        </span>
      ))}
    </span>
  );
}
export function NumberField({
  label,
  value,
  onChange,
  min = 0,
  max = 255,
  disabled = false,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  disabled?: boolean;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input
        type="number"
        value={value}
        min={min}
        max={max}
        step="1"
        required
        disabled={disabled}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
}
export function SelectField({
  label,
  value,
  onChange,
  options,
  disabled = false,
}: {
  label: string;
  value: number | string;
  onChange: (value: string) => void;
  options: { value: number | string; label: string; disabled?: boolean }[];
  disabled?: boolean;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <select
        aria-label={label}
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value} disabled={o.disabled}>
            {o.label}
          </option>
        ))}
      </select>
    </label>
  );
}
export function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className="toggle">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span>{label}</span>
    </label>
  );
}
export function Floating({
  title,
  children,
  onClose,
  initial = 0,
  wide = false,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  initial?: number;
  wide?: boolean;
}) {
  const { t } = useI18n();
  const [position, setPosition] = useState({
    x: Math.min(90 + initial * 24, window.innerWidth - 440),
    y: 85 + (initial % 4) * 24,
  });
  const [z, setZ] = useState(100 + initial);
  const drag = useRef<{ x: number; y: number } | null>(null);
  const ref = useRef<HTMLElement>(null);
  return (
    <section
      ref={ref}
      className={`floating ${wide ? "wide" : ""}`}
      role="dialog"
      aria-label={title}
      aria-modal="false"
      style={{
        left: Math.max(8, position.x),
        top: Math.max(8, position.y),
        zIndex: z,
      }}
      onPointerDown={() => setZ((Date.now() % 1_000_000) + 100)}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.stopPropagation();
          onClose();
        }
      }}
    >
      <header
        className="floating-header"
        onPointerDown={(e) => {
          if ((e.target as HTMLElement).closest("button")) return;
          const rect = ref.current!.getBoundingClientRect();
          drag.current = { x: e.clientX - rect.left, y: e.clientY - rect.top };
          e.currentTarget.setPointerCapture(e.pointerId);
        }}
        onPointerMove={(e) => {
          if (!drag.current) return;
          setPosition({
            x: Math.max(
              8,
              Math.min(window.innerWidth - 120, e.clientX - drag.current.x),
            ),
            y: Math.max(
              8,
              Math.min(window.innerHeight - 60, e.clientY - drag.current.y),
            ),
          });
        }}
        onPointerUp={() => {
          drag.current = null;
        }}
        onPointerCancel={() => {
          drag.current = null;
        }}
      >
        <GripHorizontal size={16} />
        <strong>{title}</strong>
        <button
          type="button"
          className="icon-button"
          onClick={onClose}
          aria-label={t("close")}
        >
          <X size={17} />
        </button>
      </header>
      <div className="floating-body">{children}</div>
    </section>
  );
}
