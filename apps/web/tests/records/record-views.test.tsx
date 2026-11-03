/* @layer renderer-app @kind test */
import { Text } from '@drizztdourden08/tessera';
import { cleanup, fireEvent, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { RecordCard, RecordGroup, RecordHead, RecordRow } from '../../src/shell';
import { HEAD_SIZE_PX } from '../../src/shell/records/head.constants';
import { head, mount, where } from './records-mount';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('I-85 P3 the head of a record, for any kind', { tags: ['critical'] }, () => {
  it('draws the outline type icon at 16 px, the code as a link, the owner avatar and the menu', () => {
    const { container } = mount(<RecordHead {...head('question', 'Q-4', 'owner')} />);
    expect(container.querySelector('[data-type-mark="question"] svg')?.getAttribute('width')).toBe(String(HEAD_SIZE_PX));
    expect(screen.getByRole('link', { name: 'Q-4' })).toBeTruthy();
    expect(screen.getByRole('img', { name: 'Assigned to you' })).toBeTruthy();
    expect(container.querySelector('[data-type-mark="question"] .type-mark--outline.type-mark--xs')).not.toBeNull();
    expect(screen.getByRole('button', { name: 'More actions for Q-4' }).className).toContain('icon-btn--2xs');
  });

  it('draws a lead as the square app logo, not a round avatar', () => {
    const { container } = mount(<RecordHead {...head('release', 'V-1.0.0', 'lead:tessera')} />);
    expect(container.querySelector('[data-lead-mark="brand"] svg')).toBeTruthy();
  });

  it('swaps the type icon for a checkbox while selecting', () => {
    const toggle = vi.fn();
    const { container } = mount(<RecordHead {...head('task', 'I-1', 'worker')} selection={{ selecting: true, selected: false, onToggle: toggle, onMark: () => undefined }} />);
    expect(container.querySelector('[data-type-mark]')).toBeNull();
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select I-1' }));
    expect(toggle).toHaveBeenCalled();
  });
});

describe('I-85 P3 the card, the row and the group', { tags: ['critical'] }, () => {
  it('opens a card from its title, and shows its signals and foot', () => {
    const open = vi.fn();
    mount(<RecordCard head={head('agent', 'A-2', 'worker')} title="coder" openLabel="Open A-2: coder" onOpen={open} signals={<Text>needs you</Text>} foot={<Text>foot line</Text>} />);
    fireEvent.click(screen.getByRole('button', { name: 'Open A-2: coder' }));
    expect(open).toHaveBeenCalledTimes(1);
    expect(screen.getByText('needs you')).toBeTruthy();
    expect(screen.getByText('foot line')).toBeTruthy();
  });

  it('shows a row with its code, title, chips and age, and opens it', () => {
    const open = vi.fn();
    mount(<RecordRow head={head('build', 'BL-web', 'worker')} title="web" onOpen={open} chips={[{ id: 'port', text: ':5173' }]} age="3m" />);
    expect(screen.getByText('BL-web')).toBeTruthy();
    expect(screen.getByText(':5173')).toBeTruthy();
    expect(screen.getByText('3m')).toBeTruthy();
    fireEvent.click(screen.getByText('web'));
    expect(open).toHaveBeenCalledTimes(1);
  });

  it('shows a group with its count, folded or open, and holds its rows', () => {
    mount(<RecordGroup header={<Text>I-1 Login</Text>} count={2} label="Questions of I-1" defaultOpen><Text>row one</Text><Text>row two</Text></RecordGroup>);
    expect(screen.getByText('I-1 Login')).toBeTruthy();
    expect(screen.getByText('row one')).toBeTruthy();
  });

  it('opens the pane from the code link of a head, in the address, without leaving the board', () => {
    mount(<RecordHead {...head('question', 'Q-4', 'owner')} />);
    fireEvent.click(within(document.body).getByRole('link', { name: 'Q-4' }));
    expect(where()).toBe('?open=Q-4');
  });
});
