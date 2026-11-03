/* @layer renderer-app @kind test */
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { launcherDoc } from './fixtures';
import { withDefs } from './list-dom';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

const orderOf = (container: HTMLElement): (string | null)[] => [...container.querySelectorAll('[data-launcher]')].map((card) => card.getAttribute('data-launcher'));

describe('S1 the launcher list', { tags: ['standard'] }, () => {
  it('sorts numbered names the way a person counts', async () => {
    const { container } = await mountBuilds(withDefs([launcherDoc('master'), launcherDoc('build-10'), launcherDoc('build-9')]));
    await screen.findByText('3 to start');
    expect(orderOf(container)).toEqual(['master', 'build-9', 'build-10']);
  });

  it('draws the text of a launcher as text, never as markup', async () => {
    const { container } = await mountBuilds(withDefs([launcherDoc('master', { title: '<img src=x onerror=alert(1)>', cmd: 'run "a" <b> & c', branch: '<br>' })]));
    expect(await screen.findByText('<img src=x onerror=alert(1)>')).toBeTruthy();
    expect(screen.getByDisplayValue('run "a" <b> & c')).toBeTruthy();
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('b')).toBeNull();
  });
});
