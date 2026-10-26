/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { buildDoc, launcherDoc } from './fixtures';
import { cardOf, withDefs } from './list-dom';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

const orderOf = (container: HTMLElement): (string | null)[] => [...container.querySelectorAll('[data-launcher]')].map((card) => card.getAttribute('data-launcher'));

describe('the launcher list', () => {
  it('pins master first, then the current ones, then the rest by name, and counts them', async () => {
    const { container } = await mountBuilds(withDefs([
      launcherDoc('zeta-1'), launcherDoc('build-2', { current: true }), launcherDoc('master'), launcherDoc('alpha-9'),
    ]));
    expect(await screen.findByText('4 to start')).toBeTruthy();
    expect(orderOf(container)).toEqual(['master', 'build-2', 'alpha-9', 'zeta-1']);
  });

  it('sorts numbered names the way a person counts', async () => {
    const { container } = await mountBuilds(withDefs([launcherDoc('master'), launcherDoc('build-10'), launcherDoc('build-9')]));
    await screen.findByText('3 to start');
    expect(orderOf(container)).toEqual(['master', 'build-9', 'build-10']);
  });

  it('shows the chips, the branch, the commit, who set it, the note and the command of a row', async () => {
    const board = withDefs(
      [launcherDoc('master'), launcherDoc('build-2', { current: true, kind: 'dev', branch: 'feat/login', commit: 'abc1234', note: 'the **login** fix' })],
      { builds: [buildDoc('open-1', { launcher: 'build-2' })] },
    );
    const { container } = await mountBuilds(board);
    const card = within(await screen.findByRole('group', { name: 'Launcher build-2: Launcher build-2' }));
    expect(card.getByText('dev')).toBeTruthy();
    expect(card.getByText('current')).toBeTruthy();
    expect(card.getByText('feat/login')).toBeTruthy();
    expect(card.getByText('abc1234')).toBeTruthy();
    expect(card.getByText('started by bundle open-1')).toBeTruthy();
    expect(card.getByText(/set by coordinator/)).toBeTruthy();
    expect(card.getByText('Note')).toBeTruthy();
    expect(card.getByText('game run --build-2')).toBeTruthy();
    expect(within(cardOf(container, 'master')).getAllByText('master')).toHaveLength(2);
  });

  it('says where the command runs: the working folder or the checkout', async () => {
    const { container } = await mountBuilds(withDefs([launcherDoc('master'), launcherDoc('wt-1', { cwd: 'C:\\work\\wt-1' })]));
    await screen.findByText('2 to start');
    expect(within(cardOf(container, 'wt-1')).getAllByText('C:\\work\\wt-1')).toHaveLength(2);
    expect(within(cardOf(container, 'master')).getByText(/run from the checkout/)).toBeTruthy();
  });

  it('draws the text of a launcher as text, never as markup', async () => {
    const { container } = await mountBuilds(withDefs([launcherDoc('master', { title: '<img src=x onerror=alert(1)>', cmd: 'run "a" <b> & c', branch: '<br>' })]));
    expect(await screen.findByText('<img src=x onerror=alert(1)>')).toBeTruthy();
    expect(screen.getByText('run "a" <b> & c')).toBeTruthy();
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('b')).toBeNull();
  });

  it('follows a launcher live: a new command is the one shown', async () => {
    const { change } = await mountBuilds(withDefs([launcherDoc('master')]));
    await screen.findByText('game run --master');
    change('launchers', 'master', launcherDoc('master', { cmd: 'game run --master --fresh' }));
    expect(await screen.findByText('game run --master --fresh')).toBeTruthy();
  });
});

describe('an empty or masterless board', () => {
  it('says the registered agent fills the list, and that master is missing', async () => {
    await mountBuilds();
    expect(await screen.findByText('Nothing to start yet.')).toBeTruthy();
    expect(screen.getByText(/The registered agent fills this list/)).toBeTruthy();
    expect(screen.getByText(/No master launcher: the thread must set one/)).toBeTruthy();
  });

  it('warns when launchers exist but none is master', async () => {
    await mountBuilds(withDefs([launcherDoc('build-2')]));
    expect(await screen.findByText(/No master launcher: the thread must set one/)).toBeTruthy();
    expect(screen.getByText(/Until then no build can be handed to you/)).toBeTruthy();
  });

  it('has no warning once master is there', async () => {
    await mountBuilds(withDefs([launcherDoc('master')]));
    await screen.findByText('1 to start');
    expect(screen.queryByText(/No master launcher/)).toBeNull();
  });
});
