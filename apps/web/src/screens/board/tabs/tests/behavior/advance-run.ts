/* @layer renderer-app @kind logic */
import { OUTCOME_BUTTON_SELECTOR, RUN_ROW_SELECTOR } from '../tests.constants';

const shownRows = (): HTMLElement[] =>
  [...document.querySelectorAll<HTMLElement>(RUN_ROW_SELECTOR)].filter((row) => !row.closest('details:not([open])'));

const nextRunAfter = (testId: string): string | null => {
  const rows = shownRows();
  const at = rows.findIndex((row) => row.dataset.test === testId);
  return at >= 0 ? (rows[at + 1]?.dataset.test ?? null) : null;
};

const advanceTo = (testId: string | null): void => {
  const row = testId ? shownRows().find((candidate) => candidate.dataset.test === testId) : undefined;
  if (!row) return;
  const calm = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  row.scrollIntoView({ block: 'nearest', behavior: calm ? 'auto' : 'smooth' });
  row.querySelector<HTMLElement>(OUTCOME_BUTTON_SELECTOR)?.focus({ preventScroll: true });
};

export { advanceTo, nextRunAfter };
