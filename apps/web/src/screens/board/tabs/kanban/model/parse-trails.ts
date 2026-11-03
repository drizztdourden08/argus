/* @layer renderer-app @kind logic */
import type { TrailInfo } from './model.type';
import type { Row } from './row.type';

const isRow = (value: unknown): value is Row => typeof value === 'object' && value !== null && !Array.isArray(value);

const infoOf = (value: unknown): TrailInfo | null => {
  if (!isRow(value) || typeof value.depth !== 'number') return null;
  return { depth: value.depth, open: typeof value.open === 'number' ? value.open : 0 };
};

const entriesOf = (raw: unknown): [string, unknown][] => {
  const body = isRow(raw) && 'trails' in raw ? raw.trails : raw;
  if (Array.isArray(body)) return body.filter(isRow).map((row) => [String(row.id ?? row.item ?? ''), row]);
  return isRow(body) ? Object.entries(body) : [];
};

const parseTrails = (raw: unknown): Map<string, TrailInfo> => {
  const trails = new Map<string, TrailInfo>();
  for (const [id, value] of entriesOf(raw)) {
    const info = infoOf(value);
    if (id && info) trails.set(id, info);
  }
  return trails;
};

export { parseTrails };
