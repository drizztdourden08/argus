/* @layer renderer-app @kind test */
import { kindOf } from '../../src/data/registry';
import { PANE_TABS } from '../../src/shell/records/pane/pane.constants';
import { defDoc, launcherDoc, runDoc } from '../board-builds/fixtures';
import { item } from './kanban-harness';
import { tabNames } from './kanban-pane-dom';

interface Feed {
  coll: string;
  key: string;
  docs: Record<string, unknown>[];
}

const labelsOf = (id: string): string[] => (kindOf(id)?.tabs ?? []).filter((tab) => Object.hasOwn(PANE_TABS, tab)).map((tab) => PANE_TABS[tab]?.label ?? tab);

const names = (dialog: HTMLElement): string[] => tabNames(dialog).map((name) => name.replace(/\d+$/, ''));

const buildBoard = (runId: string, more: Feed[] = []) => ({
  items: [item('I-1')],
  feeds: [
    { coll: 'launchers', key: 'name', docs: [launcherDoc('web', { for: 'I-1' })] },
    { coll: 'launches', key: 'name', docs: [defDoc('web')] },
    { coll: 'runs', key: 'id', docs: [runDoc(runId, 'launcher-web', { status: 'failed', exitCode: 3 })] },
    ...more,
  ],
});

export { buildBoard, labelsOf, names };
