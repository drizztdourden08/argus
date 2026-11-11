/* @layer renderer-app @kind logic */
import type { HandoverDoc } from '../../../../../data/types';

const handoverEstimate = (doc: HandoverDoc): string => {
  const estimate = doc.estimate;
  if (!estimate) return '';
  const agents = estimate.agents === undefined ? '' : `${estimate.agents} agent${estimate.agents === 1 ? '' : 's'}`;
  const percent = typeof estimate.percent === 'number' ? `est. ${estimate.percent}%` : '';
  const minutes = typeof estimate.minutes === 'number' ? `${estimate.minutes} min` : '';
  return [agents, percent, minutes].filter(Boolean).join(' · ');
};

export { handoverEstimate };
