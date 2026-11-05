/* @layer renderer-app @kind test */
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { defDoc, launcherDoc, planDoc, runOn, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const launched = (extra: object = {}) => ({
  builds: [buildDoc('b1', { launcher: 'l1', title: 'First bundle', note: 'Read me first.' })],
  launchers: [launcherDoc('l1', { cwd: 'C:\\work\\rotp', ...extra })],
  launches: [defDoc('l1')],
  tests: [testDoc('T-1', { build: 'b1' })],
});

const playDisabled = (tag: string): boolean => screen.getByRole<HTMLButtonElement>('button', { name: `Play ${tag}` }).disabled;

describe('S1 the launch box of a linked launcher', { tags: ['standard'] }, () => {
  it('follows the launcher live: a new command is the one shown', async () => {
    const { change } = await mountTab(launched());
    await screen.findByDisplayValue('rotp launch l1');
    change('launchers', 'l1', launcherDoc('l1', { cmd: 'rotp launch l1 --fresh' }));
    expect(await screen.findByDisplayValue('rotp launch l1 --fresh')).toBeTruthy();
  });

  it('says a draft is not built: the command shows, there is no Play', async () => {
    await mountTab({ ...launched(), items: [planDoc('I-9', ['T-1']), runOn('I-50', 'l1', { stage: 'draft' })] });
    expect(await screen.findByDisplayValue('rotp launch l1')).toBeTruthy();
    expect(screen.getByText(/Draft: not built yet, so no Play/)).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Play l1' })).toBeNull();
  });
});

describe('S1 the launch box without a usable launcher', { tags: ['standard'] }, () => {
  it('says a retired launcher was retired and disables Play', async () => {
    await mountTab({
      group: 'run',
      items: [planDoc('I-9', ['T-1']), runOn('I-50', 'gone')],
      launchers: [launcherDoc('gone', { retiredAt: 'then', retiredBy: 'lead', retireNote: 'replaced' })],
      launches: [],
      tests: [testDoc('T-1')],
    });
    expect(await screen.findByText(/launcher gone was retired by lead \(replaced\)/)).toBeTruthy();
    expect(playDisabled('gone')).toBe(true);
  });

  it('says a missing launcher is missing and disables Play', async () => {
    await mountTab({ group: 'run', items: [planDoc('I-9', ['T-1']), runOn('I-50', 'nope')], tests: [testDoc('T-1')] });
    expect(await screen.findByText(/launcher nope is missing/)).toBeTruthy();
    expect(playDisabled('nope')).toBe(true);
  });

  it('shows no launch box for a run without a build', async () => {
    await mountTab({ group: 'run', items: [planDoc('I-9', ['T-1']), runOn('I-50', null)], tests: [testDoc('T-1')] });
    await screen.findByText('Test number 1');
    expect(screen.queryByRole('group', { name: 'Launch I-50' })).toBeNull();
    expect(screen.queryByRole('button', { name: /Play/ })).toBeNull();
  });
});
