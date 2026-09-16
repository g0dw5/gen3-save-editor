import { useEffect, useState } from "react";
import { api } from "./api";

type CharacterKind = "trainer_sprite" | "object_sprite";
interface CharacterImage {
  url: string;
  width: number;
  height: number;
}
const images = new Map<string, Promise<CharacterImage | null>>();

// Shared by trainer portraits and map objects. A ROM switch must never reuse
// another ROM's graphics, even when its object IDs happen to match.
export function useRomCharacterImage(
  md5: string,
  kind: CharacterKind,
  id: number | null,
) {
  const key = `${md5}:${kind}:${id}`;
  const [loaded, setLoaded] = useState<{
    key: string;
    image: CharacterImage | null;
  } | null>(null);
  useEffect(() => {
    if (id === null) return;
    let active = true;
    if (!images.has(key)) {
      images.set(
        key,
        api<{ url: string }>(kind, { id })
          .then(({ url }) => {
            if (!url) return null;
            return new Promise<CharacterImage | null>((resolve) => {
              const image = new Image();
              image.onload = () =>
                resolve({
                  url,
                  width: image.naturalWidth,
                  height: image.naturalHeight,
                });
              image.onerror = () => resolve(null);
              image.src = url;
            });
          })
          .catch(() => null),
      );
    }
    images.get(key)!.then((image) => {
      if (active) setLoaded({ key, image });
    });
    return () => {
      active = false;
    };
  }, [key, id, kind]);
  return id === null ? null : loaded?.key === key ? loaded.image : undefined;
}
