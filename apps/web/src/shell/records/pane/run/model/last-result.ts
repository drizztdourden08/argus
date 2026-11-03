/* @layer renderer-app @kind logic */
import type { PlanRun, RunResult } from '../run.type';

const lastResult = (run: PlanRun, id: string): RunResult | null => run.results.filter((result) => result.test === id).at(-1) ?? null;

export { lastResult };
