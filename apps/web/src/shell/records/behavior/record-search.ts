/* @layer renderer-app @kind logic */
import { OPEN_PARAM, PANE_PARAM } from '../records.constants';

const recordSearch = (search: string, code: string | null, tab?: string | null): string => {
  const params = new URLSearchParams(search);
  params.delete(OPEN_PARAM);
  params.delete(PANE_PARAM);
  if (code) params.set(OPEN_PARAM, code);
  if (code && tab) params.set(PANE_PARAM, tab);
  const text = params.toString();
  return text ? `?${text}` : '';
};

export { recordSearch };
