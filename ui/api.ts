import { invoke } from "@tauri-apps/api/core";
import { open, save, confirm } from "@tauri-apps/plugin-dialog";
export const native = "__TAURI_INTERNALS__" in window;
export interface ApiError {
  code: string;
  detail: string;
}
export async function api<T>(
  command: string,
  payload: unknown = {},
): Promise<T> {
  if (native) return invoke<T>("request", { input: { command, payload } });
  if (!import.meta.env.DEV) throw { code: "desktop_required", detail: "" };
  const response = await fetch("/api", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ command, payload }),
  });
  if (!response.ok) throw { code: "backend", detail: `${response.status}` };
  const result = await response.json();
  if (!result.ok) throw result.error;
  return result.data;
}
export async function chooseFile(
  kind: "rom" | "save" | "pokemon",
): Promise<Record<string, string> | null> {
  const extensions =
    kind === "rom" ? ["gba"] : kind === "save" ? ["sav", "srm"] : ["json"];
  if (native) {
    const path = await open({
      multiple: false,
      filters: [{ name: kind.toUpperCase(), extensions }],
    });
    return path && typeof path === "string" ? { path } : null;
  }
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = extensions.map((e) => "." + e).join(",");
    input.oncancel = () => resolve(null);
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      const reader = new FileReader();
      reader.onload = () =>
        resolve({
          bytes: String(reader.result).split(",")[1],
          name: file.name,
        });
      reader.readAsDataURL(file);
    };
    input.click();
  });
}
export async function outputPath(
  name: string,
  extension: string,
): Promise<string | null> {
  return save({
    defaultPath: name,
    filters: [{ name: extension.toUpperCase(), extensions: [extension] }],
  });
}
export function download(
  name: string,
  data: BlobPart,
  type = "application/octet-stream",
) {
  const url = URL.createObjectURL(new Blob([data], { type }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
export function fromBase64(data: string): Uint8Array<ArrayBuffer> {
  return Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
}

export async function confirmAction(message: string): Promise<boolean> {
  return native
    ? confirm(message, { title: document.title, kind: "warning" })
    : window.confirm(message);
}
