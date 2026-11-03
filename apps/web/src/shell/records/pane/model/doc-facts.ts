/* @layer renderer-app @kind logic */
import type { DocFact, PaneDoc } from '../pane.type';
import { FACT_HIDDEN, FACT_MAX } from './pane-model.constants';

const labelOf = (key: string): string => {
  const words = key.replace(/([a-z0-9])([A-Z])/g, '$1 $2').toLowerCase();
  return words.charAt(0).toUpperCase() + words.slice(1);
};

const isScalar = (value: unknown): value is string | number | boolean => ['string', 'number', 'boolean'].includes(typeof value);

const docFacts = (doc: PaneDoc): DocFact[] =>
  Object.entries(doc)
    .filter(([key, value]) => !FACT_HIDDEN.includes(key) && isScalar(value) && String(value) !== '')
    .slice(0, FACT_MAX)
    .map(([key, value]) => ({ key, label: labelOf(key), text: String(value) }));

export { docFacts };
