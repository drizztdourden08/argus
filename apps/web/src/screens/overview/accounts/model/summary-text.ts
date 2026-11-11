/* @layer renderer-app @kind logic */
import { NOTHING_TEXT } from '../compare.constants';

const asText = (value: unknown): string | null => {
  if (typeof value === 'string') return value;
  if (value === null || typeof value !== 'object') return null;
  const parts = Object.entries(value)
    .filter(([, count]) => typeof count === 'number' || typeof count === 'string')
    .map(([name, count]) => `${count} ${name.replace(/_/g, ' ')}`);
  return parts.length > 0 ? parts.join(', ') : null;
};

const summaryText = (result: unknown): string => {
  const summary = (result as { summary?: unknown } | null)?.summary;
  return asText(summary ?? result) ?? NOTHING_TEXT;
};

export { summaryText };
