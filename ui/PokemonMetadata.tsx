import { itemOption } from "./names";
import { NumberField, SelectField, Toggle } from "./components";
import { useI18n } from "./i18n";
import type { Catalog, Pokemon, OriginOptions } from "./types";

export const contestCategories = ["cool", "beauty", "cute", "smart", "tough"];
export const ribbonNames = [
  "champion",
  "winning",
  "victory",
  "artist",
  "effort",
  "marine",
  "land",
  "sky",
  "country",
  "national",
  "earth",
  "world",
];
// Preserve unrelated and reserved bits, including the unsigned high bit.
export const replaceBits = (word: number, mask: number, value: number) =>
  ((word & ~mask) | (value & mask)) >>> 0;
interface Props {
  pokemon: Pokemon;
  catalog: Catalog;
  change: (key: string, value: unknown) => void;
  free: boolean;
  pidLocked: boolean;
}

export function PokemonOrigin({
  pokemon: p,
  catalog,
  change,
  free,
  origins,
}: Props & { origins: OriginOptions | null }) {
  const { t } = useI18n();
  const kind = p.met_level === 0 ? "hatched" : "caught";
  const options = (
    values: { value: number; label: string }[],
    current: number,
  ) =>
    values.some((v) => v.value === current)
      ? values
      : [
          ...values,
          {
            value: current,
            label: `${t("unknownValue")} #${current}`,
            disabled: true,
          },
        ];
  const games = [1, 2, 3, 4, 5, 15].map((id) => ({
    value: id,
    label: t(`originGame_${id}`),
  }));
  const languages = [1, 2, 3, 4, 5, 7].map((id) => ({
    value: id,
    label: t(`pokemonLanguage_${id}`),
  }));
  const eligible = new Set(
    kind === "hatched"
      ? (origins?.hatch_regions ?? [])
      : (origins?.encounters.map((e) => e.region) ?? []),
  );
  const locations: { value: number; label: string; disabled?: boolean }[] = [
    ...catalog.met_locations
      .filter((l) => free || eligible.has(l.id))
      .map((l) => ({ value: l.id, label: `${l.name} · #${l.id}` })),
    ...(free
      ? [253, 254, 255].map((id) => ({
          value: id,
          label: t(`metSpecial_${id}`),
        }))
      : []),
  ];
  if (!locations.some((l) => l.value === p.met_location)) {
    const name =
      catalog.met_locations.find((l) => l.id === p.met_location)?.name ??
      (p.met_location >= 253
        ? t(`metSpecial_${p.met_location}`)
        : t("unknownValue"));
    locations.push({
      value: p.met_location,
      label: `${name} · #${p.met_location} — ${t("originCurrent")}`,
      disabled: true,
    } as { value: number; label: string; disabled?: boolean });
  }
  return (
    <>
      <label className="field">
        <span>{t("ot_name")}</span>
        <input
          value={p.ot_name}
          onChange={(e) => change("ot_name", e.target.value)}
        />
      </label>
      <div className="field-grid">
        <NumberField
          label={t("publicTrainerId")}
          value={p.ot_id & 0xffff}
          max={65535}
          onChange={(v) => change("ot_id", replaceBits(p.ot_id, 0xffff, v))}
        />
        <NumberField
          label={t("secretTrainerId")}
          value={p.ot_id >>> 16}
          max={65535}
          onChange={(v) =>
            change("ot_id", replaceBits(p.ot_id, 0xffff0000, v << 16))
          }
        />
        <SelectField
          label={t("ot_gender")}
          value={p.ot_gender}
          onChange={(v) => change("ot_gender", +v)}
          options={[
            { value: 0, label: t("male") },
            { value: 1, label: t("female") },
          ]}
        />
        <NumberField
          label={t("met_level")}
          value={p.met_level}
          min={kind === "hatched" ? 0 : 1}
          max={127}
          disabled={kind === "hatched" && !free}
          onChange={(v) => change("met_level", v)}
        />
      </div>
      <SelectField
        label={t("originKind")}
        value={kind}
        options={[
          { value: "caught", label: t("originCaught") },
          {
            value: "hatched",
            label: t("originHatched"),
            disabled: !free && !origins?.can_hatch,
          },
        ]}
        onChange={(v) => {
          change(
            "met_level",
            v === "hatched" ? 0 : Math.max(1, p.met_level || p.level),
          );
        }}
      />
      <SelectField
        searchable
        disabled={!free && !origins}
        label={t("met_location")}
        value={p.met_location}
        onChange={(v) => change("met_location", +v)}
        options={locations}
      />
      <p className="small muted">
        {t(
          !free && !origins
            ? "originLoading"
            : kind === "hatched"
              ? "hatchHelp"
              : "originHelp",
        )}
      </p>
      <SelectField
        label={t("origin_game")}
        value={p.origin_game}
        onChange={(v) => change("origin_game", +v)}
        options={options(games, p.origin_game)}
      />
      <SelectField
        searchable
        label={t("ball")}
        value={p.ball}
        onChange={(v) => change("ball", +v)}
        options={options(
          catalog.items
            .filter(
              (i) =>
                catalog.editor_rules?.balls.includes(i.id) ??
                (i.id >= 1 && i.id <= 12),
            )
            .map((i) => itemOption(catalog, i)),
          p.ball,
        )}
      />
      <SelectField
        label={t("language")}
        value={p.language}
        onChange={(v) => change("language", +v)}
        options={options(languages, p.language)}
      />
      <p className="small muted">{t("pokemonLanguageHelp")}</p>
      <Toggle
        label={t("fatefulEncounter")}
        checked={Boolean(p.ribbons & 0x80000000)}
        onChange={(v) =>
          change(
            "ribbons",
            replaceBits(p.ribbons, 0x80000000, v ? 0x80000000 : 0),
          )
        }
      />
      <details className="metadata-details">
        <summary>{t("storedIdentifiers")}</summary>
        <NumberField
          label={t("ot_id")}
          value={p.ot_id}
          max={0xffffffff}
          onChange={() => {}}
          disabled
        />
        <p className="small muted">{t("combinedIdHelp")}</p>
      </details>
    </>
  );
}

export function PokemonAdvanced({
  pokemon: p,
  catalog,
  change,
  free,
  pidLocked,
}: Props) {
  const { t } = useI18n();
  const strain = p.pokerus >>> 4;
  const days = p.pokerus & 15;
  const invalidVirus = (strain === 0 && days !== 0) || days > (strain % 4) + 1;
  const virusState = invalidVirus
    ? "custom"
    : !p.pokerus
      ? "none"
      : days
        ? "active"
        : "cured";
  const updateVirus = (state: string) => {
    const nextStrain = strain || 1;
    change(
      "pokerus",
      state === "none"
        ? 0
        : (nextStrain << 4) |
            (state === "active"
              ? Math.max(
                  1,
                  Math.min(days || (nextStrain % 4) + 1, (nextStrain % 4) + 1),
                )
              : 0),
    );
  };
  return (
    <>
      <h3>{t("markings")}</h3>
      <p className="small muted">{t("markingsHelp")}</p>
      <div className="marking-controls">
        {["●", "■", "▲", "♥"].map((symbol, i) => (
          <label key={symbol} title={t(`marking_${i}`)}>
            <input
              type="checkbox"
              aria-label={t(`marking_${i}`)}
              checked={Boolean(p.markings & (1 << i))}
              onChange={(e) =>
                change(
                  "markings",
                  replaceBits(
                    p.markings & 15,
                    1 << i,
                    e.target.checked ? 1 << i : 0,
                  ),
                )
              }
            />
            <span>{symbol}</span>
          </label>
        ))}
      </div>
      <h3>{t("pokerus")}</h3>
      <SelectField
        label={t("pokerusState")}
        value={virusState}
        onChange={updateVirus}
        options={[
          ...["none", "active", "cured"].map((v) => ({
            value: v,
            label: t(`pokerus_${v}`),
          })),
          ...(invalidVirus
            ? [
                {
                  value: "custom",
                  label: `${t("unknownValue")} · ${p.pokerus}`,
                  disabled: true,
                },
              ]
            : []),
        ]}
      />
      <div className="field-grid">
        <SelectField
          label={t("pokerusStrain")}
          value={strain}
          disabled={!p.pokerus || (invalidVirus && !free)}
          options={Array.from({ length: 16 }, (_, value) => ({
            value,
            label: String(value),
            disabled: value === 0 && !free,
          }))}
          onChange={(v) =>
            change(
              "pokerus",
              (+v << 4) | Math.min(days, free ? 15 : (+v % 4) + 1),
            )
          }
        />
        <SelectField
          label={t("pokerusDays")}
          value={days}
          disabled={!days || (invalidVirus && !free)}
          options={Array.from(
            { length: Math.max(days, free ? 15 : (strain % 4) + 1) + 1 },
            (_, value) => ({
              value,
              label: String(value),
              disabled: !free && value > (strain % 4) + 1,
            }),
          )}
          onChange={(v) => change("pokerus", (strain << 4) | +v)}
        />
      </div>
      <p className="small muted">{t("pokerusHelp")}</p>
      <h3>{t("ribbons")}</h3>
      <p className="small muted">{t("contestRibbonHelp")}</p>
      <div className="field-grid">
        {contestCategories.map((key, i) => {
          const rank = (p.ribbons >>> (i * 3)) & 7;
          const maxRank = catalog.editor_rules?.contest_ranks[i] ?? 4;
          return (
            <SelectField
              key={key}
              label={`${t(key)} ${t("ribbonRank")}`}
              value={rank}
              onChange={(v) =>
                change(
                  "ribbons",
                  replaceBits(p.ribbons, 7 << (i * 3), +v << (i * 3)),
                )
              }
              options={[
                ...Array.from({ length: maxRank + 1 }, (_, value) => ({
                  value,
                  label: t(`contestRank_${value}`),
                })),
                ...(rank > maxRank
                  ? [
                      {
                        value: rank,
                        label: `${t("unknownValue")} #${rank}`,
                        disabled: true,
                      },
                    ]
                  : []),
              ]}
            />
          );
        })}
      </div>
      <div className="ribbon-checklist">
        {ribbonNames.map((key, i) => (
          <Toggle
            key={key}
            label={t(`ribbon_${key}`)}
            checked={Boolean(p.ribbons & (1 << (i + 15)))}
            onChange={(v) =>
              change(
                "ribbons",
                replaceBits(p.ribbons, 1 << (i + 15), v ? 1 << (i + 15) : 0),
              )
            }
          />
        ))}
      </div>
      <details className="metadata-details">
        <summary>{t("storedIdentifiers")}</summary>
        <NumberField
          label={t("pid")}
          value={p.pid}
          max={0xffffffff}
          disabled={!free || pidLocked}
          onChange={(v) => change("pid", v)}
        />
        <p className="small muted">
          {t(
            catalog.editor_rules?.nature_override
              ? "pidOverrideHelp"
              : "pidHelp",
          )}
        </p>
        <NumberField
          label={t("reservedRibbonBits")}
          value={(p.ribbons >>> 27) & 15}
          max={15}
          onChange={() => {}}
          disabled
        />
        <label className="field">
          <span>{t("checksumState")}</span>
          <input
            disabled
            value={t(p.checksum_ok ? "checksumValid" : "checksumInvalid")}
          />
        </label>
      </details>
    </>
  );
}
