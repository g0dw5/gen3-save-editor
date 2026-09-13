import { useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useI18n } from "./i18n";

export interface SelectOption {
  value: number | string;
  label: string;
  disabled?: boolean;
  search?: string[];
}
export interface SelectProps {
  label: string;
  value: number | string;
  onChange: (value: string) => void;
  options: SelectOption[];
  disabled?: boolean;
  searchable?: boolean;
}
const normalize = (s: string) => s.normalize("NFKC").toLocaleLowerCase().trim();

/** A selection commits an ID; typing or dismissing never creates a free-text ID. */
export function SearchSelect({
  label,
  value,
  onChange,
  options,
  disabled,
}: SelectProps) {
  const { t } = useI18n();
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(-1);
  const [position, setPosition] = useState({
    left: 0,
    top: 0,
    width: 0,
    maxHeight: 280,
  });
  const selected = options.find((o) => String(o.value) === String(value));
  const words = normalize(query).split(/\s+/).filter(Boolean);
  const filtered = options.filter((o) =>
    words.every((word) =>
      normalize([o.label, o.value, ...(o.search ?? [])].join(" ")).includes(
        word,
      ),
    ),
  );
  const begin = () => {
    setQuery("");
    setActive(-1);
    setOpen(true);
  };
  const choose = (option: SelectOption) => {
    if (option.disabled || disabled || input.current?.matches(":disabled"))
      return;
    onChange(String(option.value));
    setOpen(false);
  };
  useEffect(() => {
    if (!open) return;
    const place = () => {
      const rect = input.current?.getBoundingClientRect();
      if (!rect) return;
      const below = window.innerHeight - rect.bottom - 8;
      const above = rect.top - 8;
      const up = below < 180 && above > below;
      const maxHeight = Math.max(60, Math.min(280, up ? above : below));
      setPosition({
        left: Math.max(
          4,
          Math.min(rect.left, window.innerWidth - rect.width - 4),
        ),
        top: up ? rect.top - maxHeight - 4 : rect.bottom + 4,
        width: rect.width,
        maxHeight,
      });
    };
    const outside = (e: PointerEvent) => {
      if (
        e.target !== input.current &&
        !popup.current?.contains(e.target as Node)
      )
        setOpen(false);
    };
    place();
    window.addEventListener("resize", place);
    window.addEventListener("scroll", place, true);
    document.addEventListener("pointerdown", outside);
    return () => {
      window.removeEventListener("resize", place);
      window.removeEventListener("scroll", place, true);
      document.removeEventListener("pointerdown", outside);
    };
  }, [open]);
  useEffect(() => {
    popup.current
      ?.querySelector(`[data-index="${active}"]`)
      ?.scrollIntoView({ block: "nearest" });
  }, [active]);
  return (
    <label className="field search-select">
      <span>{label}</span>
      <input
        ref={input}
        role="combobox"
        aria-label={label}
        aria-autocomplete="list"
        aria-expanded={open}
        aria-controls={open ? id : undefined}
        aria-activedescendant={
          open && active >= 0 && filtered[active]
            ? `${id}-${active}`
            : undefined
        }
        autoComplete="off"
        disabled={disabled}
        value={open ? query : (selected?.label ?? String(value))}
        placeholder={open ? t("searchChoices") : undefined}
        onFocus={begin}
        onClick={() => {
          if (!open) begin();
        }}
        onBlur={() => setOpen(false)}
        onChange={(e) => {
          setQuery(e.target.value);
          setActive(-1);
          setOpen(true);
        }}
        onKeyDown={(e) => {
          if (e.nativeEvent.isComposing) return;
          if (e.key === "Escape" && open) {
            e.preventDefault();
            e.stopPropagation();
            setOpen(false);
          }
          if (e.key === "ArrowDown" || e.key === "ArrowUp") {
            e.preventDefault();
            if (!open) {
              begin();
              return;
            }
            const step = e.key === "ArrowDown" ? 1 : -1;
            let next =
              active < 0 ? (step > 0 ? 0 : filtered.length - 1) : active + step;
            while (
              next >= 0 &&
              next < filtered.length &&
              filtered[next].disabled
            )
              next += step;
            if (next >= 0 && next < filtered.length) setActive(next);
          }
          if (e.key === "Enter") {
            e.preventDefault();
            if (!open) {
              begin();
              return;
            }
            const option =
              filtered[active] ?? filtered.find((o) => !o.disabled);
            if (option) choose(option);
          }
        }}
      />
      {open &&
        !disabled &&
        createPortal(
          <div
            ref={popup}
            className="select-popup"
            id={id}
            role="listbox"
            aria-label={label}
            style={position}
            onMouseDown={(e) => e.preventDefault()}
          >
            {filtered.map((o, i) => (
              <div
                key={o.value}
                id={`${id}-${i}`}
                data-index={i}
                role="option"
                aria-selected={String(value) === String(o.value)}
                aria-disabled={o.disabled || undefined}
                className={i === active ? "highlighted" : ""}
                onMouseEnter={() => setActive(i)}
                onClick={() => choose(o)}
              >
                {o.label}
              </div>
            ))}
            {!filtered.length && (
              <p className="small muted">{t("noChoiceResults")}</p>
            )}
          </div>,
          document.body,
        )}
    </label>
  );
}
