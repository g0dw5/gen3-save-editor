import { useEffect, useMemo, useState } from "react";
import { api } from "./api";
import { useI18n } from "./i18n";
import type { Catalog, Opponent, World } from "./types";

const images = new Map<string, Promise<string>>();
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
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    setUrl(null);
    const key = `${md5}:${kind}:${id}`;
    if (!images.has(key))
      images.set(
        key,
        api<{ url: string }>(kind, { id })
          .then((value) => value.url)
          .catch(() => ""),
      );
    images.get(key)!.then((value) => {
      if (active) setUrl(value);
    });
    return () => {
      active = false;
    };
  }, [md5, id, kind]);
  return (
    <figure className={`trainer-art-frame ${kind}`}>
      <div>
        {url ? (
          <img src={url} alt={label} draggable={false} />
        ) : (
          <span className="small muted">
            {t(url === null ? "loading" : "artUnavailable")}
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
