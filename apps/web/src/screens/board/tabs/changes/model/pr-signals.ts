/* @layer renderer-app @kind logic */
import type { CheckRun } from '../../../../../data/remote';
import type { PrSignal, PullRequestDoc } from '../changes.type';
import { checksProgress } from './checks-progress';

const prSignals = (pr: PullRequestDoc, runs: readonly CheckRun[] | null): PrSignal[] => {
  if (pr.status !== 'open') return [];
  const found: PrSignal[] = [];
  if (checksProgress(runs)?.failed) found.push({ id: 'failing', icon: 'circle-x', label: 'A check run is failing', tone: 'danger' });
  if (pr.replay?.state === 'failed') found.push({ id: 'conflict', icon: 'triangle-alert', label: 'The replay to GitHub failed: a conflict or a refusal', tone: 'warning' });
  return found;
};

export { prSignals };
