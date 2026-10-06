import { useI18n } from "./i18n";
import type {
  Catalog,
  PokemonSource,
  QueryTarget,
  StaticBattleMember,
} from "./types";

function memberName(
  member: StaticBattleMember,
  catalog: Catalog,
  t: (key: string) => string,
): string {
  const species =
    member.species == null
      ? t("unresolved")
      : (catalog.species.find((s) => s.id === member.species)?.name ??
        `#${member.species}`);
  const level = member.level == null ? t("unresolved") : `Lv. ${member.level}`;
  const item =
    member.held_item == null
      ? t("unresolved")
      : member.held_item === 0
        ? t("noHeldItem")
        : (catalog.items.find((i) => i.id === member.held_item)?.name ??
          `#${member.held_item}`);
  return `${species} · ${level} · ${t("held_item")}: ${item}`;
}

export function staticBattleSummary(
  mon: PokemonSource,
  catalog: Catalog,
  t: (key: string) => string,
): string[] {
  if (!mon.battle_members?.length) return [];
  return [
    t("staticBattlePair"),
    ...mon.battle_members.map((m) => memberName(m, catalog, t)),
    t("staticBattlePairHelp"),
  ];
}

export function StaticBattleDetails({
  mon,
  catalog,
  onTarget,
}: {
  mon?: PokemonSource | null;
  catalog: Catalog;
  onTarget: (target: QueryTarget) => void;
}) {
  const { t } = useI18n();
  if (!mon?.battle_members?.length) return null;
  return (
    <div className="small">
      <strong>{t("staticBattlePair")}</strong>
      {mon.battle_members.map((member) => (
        <p key={member.member}>
          {member.species == null ? (
            t("unresolved")
          ) : (
            <button
              className="link-button"
              onClick={() => onTarget({ kind: "species", id: member.species! })}
            >
              {catalog.species.find((s) => s.id === member.species)?.name ??
                `#${member.species}`}{" "}
              ↗
            </button>
          )}
          {` · ${member.level == null ? t("unresolved") : `Lv. ${member.level}`} · ${t("held_item")}: `}
          {member.held_item == null ? (
            t("unresolved")
          ) : member.held_item === 0 ? (
            t("noHeldItem")
          ) : (
            <button
              className="link-button"
              onClick={() => onTarget({ kind: "item", id: member.held_item! })}
            >
              {catalog.items.find((i) => i.id === member.held_item)?.name ??
                `#${member.held_item}`}{" "}
              ↗
            </button>
          )}
        </p>
      ))}
      <p className="muted">{t("staticBattlePairHelp")}</p>
    </div>
  );
}
