/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { finishedRuns } from '../../../../../data/runs';
import type { RunEntry } from '../../../../../data/runs';

const finishedCount = (done: readonly WithId<Agent>[], runs: readonly RunEntry[]): number => done.length + finishedRuns(runs).length;

export { finishedCount };
