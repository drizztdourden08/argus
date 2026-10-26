/* @layer renderer-app @kind test */
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, defDoc, launcherDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const launched = (extra: object = {}) => ({
  builds: [buildDoc('b1', { launcher: 'l1', title: 'First bundle', note: 'Read me first.' })],
  launchers: [launcherDoc('l1', { cwd: 'C:\\work\\rotp', ...extra })],
  launches: [defDoc('l1')],
  tests: [testDoc('T-1', { build: 'b1' })],
});

const playDisabled = (tag: string): boolean => screen.getByRole<HTMLButtonElement>('button', { name: `Play ${tag}` }).disabled;

describe('the launch box of a linked launcher', () => {
  it('shows the command of the linked launcher, where it runs and the link to the launcher', async () => {
    await mountTab(launched());
    expect(await screen.findByText('rotp launch l1')).toBeTruthy();
    expect(screen.getByText('C:\\work\\rotp')).toBeTruthy();
    expect(screen.getByRole('link', { name: 'launcher l1' }).getAttribute('href')).toBe('#builds');
    expect(screen.getByText('First bundle')).toBeTruthy();
    expect(screen.getByText('Note')).toBeTruthy();
  });

  it('follows the launcher live: a new command is the one shown', async () => {
    const { change } = await mountTab(launched());
    await screen.findByText('rotp launch l1');
    change('launchers', 'l1', launcherDoc('l1', { cmd: 'rotp launch l1 --fresh' }));
    expect(await screen.findByText('rotp launch l1 --fresh')).toBeTruthy();
  });

  it('says a draft is not built: the command shows, there is no Play', async () => {
    await mountTab({ ...launched(), builds: [buildDoc('b1', { launcher: 'l1', status: 'draft' })] });
    expect(await screen.findByText('rotp launch l1')).toBeTruthy();
    expect(screen.getByText(/Draft: not built yet, so no Play/)).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Play b1' })).toBeNull();
  });
});

describe('the launch box without a usable launcher', () => {
  it('says a retired launcher was retired and disables Play', async () => {
    await mountTab({
      builds: [buildDoc('b3', { launcher: 'gone' })],
      launchers: [launcherDoc('gone', { retiredAt: 'then', retiredBy: 'lead', retireNote: 'replaced' })],
      launches: [],
      tests: [testDoc('T-1', { build: 'b3' })],
    });
    expect(await screen.findByText(/launcher gone was retired by lead \(replaced\)/)).toBeTruthy();
    expect(playDisabled('b3')).toBe(true);
  });

  it('says a missing launcher is missing and disables Play', async () => {
    await mountTab({ builds: [buildDoc('b4', { launcher: 'nope' })], tests: [testDoc('T-1', { build: 'b4' })] });
    expect(await screen.findByText(/launcher nope is missing/)).toBeTruthy();
    expect(playDisabled('b4')).toBe(true);
  });

  it('says a bundle without a command has none and has no Play', async () => {
    await mountTab({ builds: [buildDoc('b5')], tests: [testDoc('T-1', { build: 'b5' })] });
    expect(await screen.findByText(/no launch command yet/)).toBeTruthy();
    expect(screen.queryByRole('button', { name: /Play/ })).toBeNull();
  });

  it('plays a bundle written before launchers with its own command and no launcher link', async () => {
    await mountTab({
      builds: [buildDoc('b6', { cmd: 'rotp launch b6', cwd: null })],
      launches: [{ name: 'bundle-b6', bundle: 'b6', cmd: 'rotp launch b6', hand: true, window: true }],
      tests: [testDoc('T-1', { build: 'b6' })],
    });
    expect(await screen.findByText('rotp launch b6')).toBeTruthy();
    expect(screen.queryByRole('link', { name: /launcher/ })).toBeNull();
    expect(playDisabled('b6')).toBe(false);
  });
});
