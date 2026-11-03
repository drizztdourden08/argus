/* @layer renderer-app @kind logic */
import { OPEN_PARAM, PANE_PARAM } from './records.constants';

const openHref = (project: string, code: string | null, tab?: string): string | null => {
  if (!project) return null;
  const base = `/p/${encodeURIComponent(project)}`;
  if (!code) return base;
  const pane = tab ? `&${PANE_PARAM}=${encodeURIComponent(tab)}` : '';
  return `${base}?${OPEN_PARAM}=${encodeURIComponent(code)}${pane}`;
};

export { openHref };
