/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { RecordCard, RecordRow } from '../../src/shell';
import { head, mount, where } from './records-mount';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const linkNames = (): string[] => screen.queryAllByRole('link').map((link) => link.textContent);

describe('I-85 R13 the text of a card links its references', { tags: ['critical'] }, () => {
  const label = 'Open Q-4: Does I-3 wait for Q-2?';

  it('links each id in the title and keeps the head code as a link', () => {
    mount(<RecordCard head={head('question', 'Q-4', 'owner')} title="Does I-3 wait for Q-2?" openLabel={label} onOpen={vi.fn()} />);
    expect(linkNames()).toEqual(['Q-4', 'I-3', 'Q-2']);
  });

  it('opens the record behind the id and not the card', () => {
    const open = vi.fn();
    mount(<RecordCard head={head('question', 'Q-4', 'owner')} title="Does I-3 wait for Q-2?" openLabel={label} onOpen={open} />);
    fireEvent.click(screen.getByRole('link', { name: 'I-3' }));
    expect(where()).toBe('?open=I-3');
    expect(open).toHaveBeenCalledTimes(0);
  });

  it('opens the card from the words of the title and from the open button', () => {
    const open = vi.fn();
    mount(<RecordCard head={head('question', 'Q-4', 'owner')} title="Does I-3 wait for Q-2?" openLabel={label} onOpen={open} />);
    fireEvent.click(screen.getByText(/Does/));
    fireEvent.keyDown(screen.getByRole('button', { name: label }), { key: 'Enter' });
    expect(open).toHaveBeenCalledTimes(2);
  });

  it('keeps a title without ids as one plain open button', () => {
    const open = vi.fn();
    mount(<RecordCard head={head('question', 'Q-4', 'owner')} title="Ship it today?" openLabel="Open Q-4: Ship it today?" onOpen={open} />);
    fireEvent.click(screen.getByRole('button', { name: 'Open Q-4: Ship it today?' }));
    expect(open).toHaveBeenCalledTimes(1);
    expect(linkNames()).toEqual(['Q-4']);
  });
});

describe('I-85 R13 the text of a row links its references', { tags: ['critical'] }, () => {
  const rowHead = head('build', 'BL-web', 'worker');

  it('links the ids in the title, and the code becomes the open button', () => {
    const open = vi.fn();
    mount(<RecordRow head={rowHead} title="Blocked by I-3 until Q-2" onOpen={open} />);
    expect(linkNames()).toEqual(['I-3', 'Q-2']);
    fireEvent.click(screen.getByRole('button', { name: 'BL-web' }));
    expect(open).toHaveBeenCalledTimes(1);
  });

  it('opens the record behind an id, not the row, and the row from its words', () => {
    const open = vi.fn();
    mount(<RecordRow head={rowHead} title="Blocked by I-3 until Q-2" onOpen={open} />);
    fireEvent.click(screen.getByRole('link', { name: 'Q-2' }));
    expect(where()).toBe('?open=Q-2');
    expect(open).toHaveBeenCalledTimes(0);
    fireEvent.click(screen.getByText(/Blocked by/));
    expect(open).toHaveBeenCalledTimes(1);
  });

  it('keeps a plain row as one open button', () => {
    const open = vi.fn();
    mount(<RecordRow head={rowHead} title="web" onOpen={open} chips={[{ id: 'stage', text: 'doing' }]} />);
    expect(linkNames()).toEqual([]);
    fireEvent.click(screen.getByText('web'));
    expect(open).toHaveBeenCalledTimes(1);
  });
});

describe('I-85 a row chip can be a link to a record', { tags: ['critical'] }, () => {
  const placeChip = { id: 'where', text: 'WT agent-ab12', title: 'X:/p/.claude/worktrees/agent-ab12', link: 'WT-agent-ab12' };

  it('draws the chip as a link to the pane of the record', () => {
    mount(<RecordRow head={head('agent', 'A-2', 'worker')} title="coder" onOpen={vi.fn()} chips={[placeChip]} />);
    const link = screen.getByRole('link', { name: 'WT agent-ab12' });
    expect(link.getAttribute('href')).toBe('/p/argus?open=WT-agent-ab12');
    fireEvent.click(link);
    expect(where()).toBe('?open=WT-agent-ab12');
  });

  it('still opens the row from its title', () => {
    const open = vi.fn();
    mount(<RecordRow head={head('agent', 'A-2', 'worker')} title="coder" onOpen={open} chips={[placeChip]} />);
    fireEvent.click(screen.getByText('coder'));
    expect(open).toHaveBeenCalledTimes(1);
  });

  it('shows a chip whose link is not a record as plain text', () => {
    const open = vi.fn();
    mount(<RecordRow head={head('agent', 'A-2', 'worker')} title="coder" onOpen={open} chips={[{ ...placeChip, link: 'WT-Bad_Name' }]} />);
    expect(linkNames()).toEqual([]);
    expect(screen.getByText('WT agent-ab12')).toBeTruthy();
    fireEvent.click(screen.getByText('coder'));
    expect(open).toHaveBeenCalledTimes(1);
  });
});
