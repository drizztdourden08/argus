/* @layer renderer-app @kind test */
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { defDoc, launcherDoc, planDoc, runOn, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const launched = (run: object = {}) => ({
  group: 'run',
  items: [planDoc('I-9', ['T-1']), runOn('I-50', 'l1', run)],
  launchers: [launcherDoc('l1', { cwd: 'C:\\work\\rotp' })],
  launches: [defDoc('l1')],
  tests: [testDoc('T-1')],
});

const playDisabled = (tag: string): boolean => screen.getByRole<HTMLButtonElement>('button', { name: `Play ${tag}` }).disabled;

describe('S1 the header of a run with a linked build', { tags: ['standard'] }, () => {
  it('shows the build as a chip that opens it, and no inline command', async () => {
    await mountTab(launched());
    expect(await screen.findByRole('button', { name: 'Open build l1' })).toBeTruthy();
    expect(screen.queryByDisplayValue('rotp launch l1')).toBeNull();
    expect(screen.queryByRole('button', { name: /Copy/ })).toBeNull();
  });

  it('has no Play for a draft, which is not built yet', async () => {
    await mountTab(launched({ stage: 'draft' }));
    expect(await screen.findByRole('button', { name: 'Open build l1' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Play l1' })).toBeNull();
  });
});

describe('S1 the header of a run without a usable build', { tags: ['standard'] }, () => {
  it('flags a retired build and disables Play', async () => {
    await mountTab({
      group: 'run',
      items: [planDoc('I-9', ['T-1']), runOn('I-50', 'gone')],
      launchers: [launcherDoc('gone', { retiredAt: 'then', retiredBy: 'lead', retireNote: 'replaced' })],
      launches: [],
      tests: [testDoc('T-1')],
    });
    expect(await screen.findByTitle(/launcher gone was retired by lead \(replaced\)/)).toBeTruthy();
    expect(playDisabled('gone')).toBe(true);
  });

  it('flags a missing build and disables Play', async () => {
    await mountTab({ group: 'run', items: [planDoc('I-9', ['T-1']), runOn('I-50', 'nope')], tests: [testDoc('T-1')] });
    expect(await screen.findByTitle(/launcher nope is missing/)).toBeTruthy();
    expect(playDisabled('nope')).toBe(true);
  });

  it('shows no build chip and no Play for a run without a build', async () => {
    await mountTab({ group: 'run', items: [planDoc('I-9', ['T-1']), runOn('I-50', null)], tests: [testDoc('T-1')] });
    await screen.findByText('Test number 1');
    expect(screen.queryByRole('button', { name: /Open build/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /Play/ })).toBeNull();
  });
});
