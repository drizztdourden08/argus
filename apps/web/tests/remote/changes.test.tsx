/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { ago, MIN, prDoc, replay, runDoc, stepDoc } from './helpers/fixtures';
import { mountChanges } from './helpers/mount-changes';

cleanAfterEach();

const orderOf = (container: HTMLElement): (string | null)[] => [...container.querySelectorAll('[data-pr]')].map((card) => card.getAttribute('data-pr'));

const cardOf = async (id: string, title = `Title of ${id}`) => within(await screen.findByRole('group', { name: `Pull request ${id}: ${title}` }));

describe('the Changes tab list', () => {
  it('says there is no pull request yet', async () => {
    await mountChanges([]);
    expect(await screen.findByText('No pull requests yet.')).toBeTruthy();
    expect(screen.getByText(/coord pr open/)).toBeTruthy();
  });

  it('lists the open ones first, then the newest, and counts them', async () => {
    const { container } = await mountChanges([
      prDoc('PR-1', { status: 'merged', createdAt: ago(60 * MIN) }),
      prDoc('PR-2', { createdAt: ago(30 * MIN) }),
      prDoc('PR-3', { createdAt: ago(10 * MIN) }),
      prDoc('PR-4', { status: 'closed', createdAt: ago(5 * MIN) }),
    ], { '/api/p/rotp/checks?pr=PR-1': { runs: [] } });
    expect(await screen.findByText('4 pull requests')).toBeTruthy();
    expect(orderOf(container)).toEqual(['PR-3', 'PR-2', 'PR-4', 'PR-1']);
  });

  it('shows the status, the branches, the item link, the author, the age and the replay state', async () => {
    await mountChanges([prDoc('PR-1')]);
    const card = await cardOf('PR-1');
    expect(card.getByText('Open')).toBeTruthy();
    expect(card.getByText('feat/login')).toBeTruthy();
    expect(card.getByText('main')).toBeTruthy();
    expect(card.getByRole('link', { name: 'I-12' }).getAttribute('href')).toBe('#I-12');
    expect(card.getByText('by coordinator')).toBeTruthy();
    expect(card.getByText('5m ago')).toBeTruthy();
    expect(card.getByText('waits for the go')).toBeTruthy();
  });

  it('shows a merged pull request with its merge commit and a link to its replay', async () => {
    await mountChanges([prDoc('PR-1', {
      status: 'merged', mergedAt: ago(2 * MIN), mergeCommit: '9999999aaaaaaa', replay: replay('done', 'https://github.com/o/r/pull/7'),
    })]);
    const card = await cardOf('PR-1');
    expect(card.getByText('Merged')).toBeTruthy();
    expect(card.getByText('9999999')).toBeTruthy();
    expect(card.getByRole('link', { name: 'on GitHub' }).getAttribute('href')).toBe('https://github.com/o/r/pull/7');
  });

  it('marks a failed replay and leaves out a missing item link', async () => {
    await mountChanges([prDoc('PR-1', { item: null, replay: replay('failed') })]);
    const card = await cardOf('PR-1');
    expect(card.getByText('replay failed')).toBeTruthy();
    expect(card.queryByRole('link')).toBeNull();
  });

  it('follows a pull request live', async () => {
    const { change } = await mountChanges([prDoc('PR-1')]);
    await cardOf('PR-1');
    change('PR-1', prDoc('PR-1', { title: 'A new title', status: 'merged' }));
    const card = await cardOf('PR-1', 'A new title');
    expect(card.getByText('Merged')).toBeTruthy();
  });

  it('draws the title as text, never as markup', async () => {
    const { container } = await mountChanges([prDoc('PR-1', { title: '<img src=x onerror=alert(1)>' })]);
    expect(await screen.findByText('<img src=x onerror=alert(1)>')).toBeTruthy();
    expect(container.querySelector('img')).toBeNull();
  });
});

const runs = [
  runDoc('C-1', { startedAt: ago(40 * MIN), status: 'fail', summary: 'A required step failed', steps: [stepDoc('lint', { exitCode: 1 })] }),
  runDoc('C-2', {
    steps: [stepDoc('lint'), stepDoc('test', { durationMs: 65_000 }), stepDoc('docs', { status: 'skipped-local', exitCode: null, durationMs: null, required: false })],
  }),
];

describe('the check runs of a pull request', () => {
  it('asks for the runs of the pull request and shows the newest first, open', async () => {
    const { calls } = await mountChanges([prDoc('PR-1', { checks: ['C-1', 'C-2'] })], { '/api/p/rotp/checks?pr=PR-1': { runs } });
    const card = await cardOf('PR-1');
    expect(await card.findByText('C-2')).toBeTruthy();
    expect(calls.some((call) => call.path === '/api/p/rotp/checks?pr=PR-1')).toBe(true);
    const names = card.getAllByText(/^C-\d$/).map((node) => node.textContent);
    expect(names).toEqual(['C-2', 'C-1']);
    expect(card.getByText('All required steps passed')).toBeTruthy();
  });

  it('shows each step with its name, status and duration, and what is not required', async () => {
    await mountChanges([prDoc('PR-1', { checks: ['C-2'] })], { '/api/p/rotp/checks?pr=PR-1': { runs: [runs[1]] } });
    const card = await cardOf('PR-1');
    const table = within(await card.findByRole('table', { name: 'Steps of C-2' }));
    expect(table.getByText('lint')).toBeTruthy();
    expect(table.getByText('1.5s')).toBeTruthy();
    expect(table.getByText('1m 5s')).toBeTruthy();
    expect(table.getByText('Skipped locally')).toBeTruthy();
    expect(table.getByText('optional')).toBeTruthy();
    expect(table.getAllByText('Passed')).toHaveLength(2);
  });

  it('derives the state of a step from its exit code when it has no status', async () => {
    const run = runDoc('C-3', { steps: [stepDoc('lint', { exitCode: 2 }), stepDoc('test', { exitCode: null, durationMs: null })] });
    await mountChanges([prDoc('PR-1', { checks: ['C-3'] })], { '/api/p/rotp/checks?pr=PR-1': { runs: [run] } });
    const table = within(await (await cardOf('PR-1')).findByRole('table', { name: 'Steps of C-3' }));
    expect(table.getByText('Failed')).toBeTruthy();
    expect(table.getByText('Queued')).toBeTruthy();
  });

  it('says there is no run yet', async () => {
    await mountChanges([prDoc('PR-1')], { '/api/p/rotp/checks?pr=PR-1': { runs: [] } });
    expect(await (await cardOf('PR-1')).findByText('No check run yet.')).toBeTruthy();
  });

  it('says when the runs could not be read', async () => {
    await mountChanges([prDoc('PR-1')]);
    expect(await screen.findByText(/The check runs could not be read/)).toBeTruthy();
  });

});

describe('the log of a step', () => {
  it('opens in a drawer', async () => {
    const { calls } = await mountChanges([prDoc('PR-1', { checks: ['C-2'] })], {
      '/api/p/rotp/checks?pr=PR-1': { runs: [runs[1]] },
      '/api/p/rotp/checks/C-2/log?step=1': 'running tests\n3 passed',
    });
    const card = await cardOf('PR-1');
    fireEvent.click(await card.findByRole('button', { name: 'Log of C-2 step test' }));
    expect(await screen.findByText(/3 passed/)).toBeTruthy();
    expect(screen.getByText('pnpm test')).toBeTruthy();
    expect(calls.some((call) => call.path === '/api/p/rotp/checks/C-2/log?step=1')).toBe(true);
  });

  it('says when a step wrote no log', async () => {
    await mountChanges([prDoc('PR-1', { checks: ['C-2'] })], {
      '/api/p/rotp/checks?pr=PR-1': { runs: [runs[1]] },
      '/api/p/rotp/checks/C-2/log?step=0': '',
    });
    const card = await cardOf('PR-1');
    fireEvent.click(await card.findByRole('button', { name: 'Log of C-2 step lint' }));
    expect(await screen.findByText('This step wrote no log.')).toBeTruthy();
  });
});
