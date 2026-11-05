/* @layer renderer-app @kind logic */
import { ICON_MAX, ICON_PX, PNG_PICK_MAX, PNG_PREFIX, SVG_MAX, SVG_PREFIX } from './board-icon.constants';
import type { IconFileResult } from './board-icon.type';

const refuse = (reason: string): IconFileResult => ({ ok: false, reason });

const readUrl = (file: Blob): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error('That image could not be read.'));
    reader.readAsDataURL(file);
  });

const loadImage = (url: string): Promise<HTMLImageElement> =>
  new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error('That image could not be decoded.'));
    image.src = url;
  });

const downsize = async (file: File): Promise<string> => {
  const image = await loadImage(await readUrl(file));
  const scale = Math.min(1, ICON_PX / Math.max(image.naturalWidth, image.naturalHeight));
  const canvas = document.createElement('canvas');
  canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
  canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));
  canvas.getContext('2d')?.drawImage(image, 0, 0, canvas.width, canvas.height);
  return canvas.toDataURL('image/png');
};

const fromPng = async (file: File): Promise<IconFileResult> => {
  if (file.size > PNG_PICK_MAX) return refuse('That PNG is over 1 MB. Pick a smaller one.');
  const url = await downsize(file);
  return url.startsWith(PNG_PREFIX) && url.length <= ICON_MAX ? { ok: true, url } : refuse('That PNG is too detailed. Pick a simpler one.');
};

const fromSvg = async (file: File): Promise<IconFileResult> => {
  if (file.size > SVG_MAX) return refuse('That SVG is over 32 KB. Pick a smaller one.');
  const url = await readUrl(file);
  const full = `${SVG_PREFIX}${url.slice(url.indexOf(',') + 1)}`;
  return full.length <= ICON_MAX ? { ok: true, url: full } : refuse('That SVG is too large once encoded.');
};

const fileToIcon = async (file: File): Promise<IconFileResult> => {
  try {
    if (file.type === 'image/png') return await fromPng(file);
    if (file.type === 'image/svg+xml') return await fromSvg(file);
    return refuse('Only PNG and SVG images can be a board icon.');
  } catch (error) {
    return refuse(error instanceof Error ? error.message : 'That image could not be read.');
  }
};

export { fileToIcon };
