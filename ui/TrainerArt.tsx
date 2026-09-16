import { useMemo } from "react";
import { useRomCharacterImage } from "./romCharacterImage";
import { useI18n } from "./i18n";
import type { Catalog, Opponent, World } from "./types";

function RomCharacter({
  md5,
  id,
  kind,
  label,
}: {
  md5: string;
  id: number;
  kind: "trainer_sprite" | "object_sprite";
  label: string;
}) {
  const { t } = useI18n();
  const image = useRomCharacterImage(md5, kind, id);
  return (
    <figure className={`trainer-art-frame ${kind}`}>
      <div>
        {image ? (
          <img src={image.url} alt={label} draggable={false} />
        ) : (
          <span className="small muted">
            {t(image === undefined ? "loading" : "artUnavailable")}
          </span>
        )}
      </div>
      <figcaption>{label}</figcaption>
    </figure>
  );
}
export function TrainerArt({
  trainer,
  world,
  catalog,
}: {
  trainer: Opponent;
  world: World | null;
  catalog: Catalog;
}) {
  const { t } = useI18n();
  const links = useMemo(
    () =>
      world?.trainer_locations.locations.filter(
        (link) => link.trainer_id === trainer.id,
      ) ?? [],
    [world, trainer.id],
  );
  const graphics = [
    ...new Set(
      links.flatMap((link) => link.actors.map((actor) => actor.graphics_id)),
    ),
  ];
  return (
    <div className="trainer-art">
      <RomCharacter
        md5={catalog.profile.md5}
        id={trainer.portrait}
        kind="trainer_sprite"
        label={t("battlePortrait")}
      />
      <div className="trainer-map-art">
        <div className="trainer-map-figures">
          {graphics.map((id, index) => (
            <RomCharacter
              key={id}
              md5={catalog.profile.md5}
              id={id}
              kind="object_sprite"
              label={`${t("overworldSprite")}${graphics.length > 1 ? ` ${index + 1}` : ""}`}
            />
          ))}
        </div>
        {!graphics.length && (
          <p className="small muted">{t("overworldUnknown")}</p>
        )}
        {!!graphics.length && (
          <details className="small muted">
            <summary>{t("artMapEvidence")}</summary>
            {links.flatMap((link) =>
              link.actors.map((actor) => (
                <div key={`${link.map_id}:${actor.local_id}`}>
                  {link.map_name} · #{actor.local_id} → {actor.graphics_id}
                </div>
              )),
            )}
          </details>
        )}
      </div>
    </div>
  );
}
