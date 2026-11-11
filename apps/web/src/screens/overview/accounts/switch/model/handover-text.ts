/* @layer renderer-app @kind logic */
import type { HandoverDoc } from '../../../../../data/types';

const handoverTitle = (doc: HandoverDoc): string => (doc.thread ? `${doc.board ?? ''} / ${doc.thread}` : (doc.board ?? doc.id ?? ''));

const handoverEstimate = (doc: HandoverDoc): string => {
  const estimate = doc.estimate;
  if (!estimate) return '';
  const agents = estimate.agents === undefined ? '' : `${estimate.agents} agent${estimate.agents === 1 ? '' : 's'}`;
  const percent = typeof estimate.percent === 'number' ? `est. ${estimate.percent}%` : '';
  const minutes = typeof estimate.minutes === 'number' ? `${estimate.minutes} min` : '';
  return [agents, percent, minutes].filter(Boolean).join(' · ');
};

const notAsked = (leads: readonly string[] | undefined, handovers: readonly HandoverDoc[]): number =>
  (leads ?? []).filter((id) => !handovers.some((doc) => doc.id === id)).length;

export { handoverEstimate, handoverTitle, notAsked };
