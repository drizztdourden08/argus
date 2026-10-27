/* @layer renderer-app @kind test */
import type { RunEntry } from '../../src/data/runs';
import type { BuildDoc, LauncherDoc, launcherViews, ViewInput } from '../../src/screens/board/tabs/builds/model';
import { defDoc, launcherDoc } from './fixtures';

const NOW = Date.parse('2026-10-26T10:00:00.000Z');

const at = (ms: number): string => new Date(NOW - ms).toISOString();

const withId = (key: string) => (doc: object) => ({ ...doc, id: String((doc as Record<string, unknown>)[key]) });

const launchers = (...docs: object[]) => docs.map(withId('name')) as LauncherDoc[];

const defs = (...docs: object[]) => docs.map(withId('name')) as ViewInput['defs'][number][];

const builds = (...docs: object[]) => docs.map(withId('tag')) as BuildDoc[];

const runs = (...docs: object[]) => docs as RunEntry[];

const input = (overrides: Partial<Parameters<typeof launcherViews>[0]> = {}) => ({
  launchers: launchers(launcherDoc('master')), defs: defs(defDoc('master')), runs: runs(), builds: builds(), now: NOW, ...overrides,
});

export { at, builds, defs, input, launchers, NOW, runs };
