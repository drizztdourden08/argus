/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LinkedText, OpenEntryContext, RecordLink } from '../../src/shell';

afterEach(cleanup);

const Where = () => {
  const { pathname, hash } = useLocation();
  return <output aria-label="where">{`${pathname}${hash}`}</output>;
};

const mount = (node: ReactNode, openEntry: (id: string | null) => void = () => undefined) =>
  render(
    <MemoryRouter initialEntries={['/p/argus']}>
      <OpenEntryContext.Provider value={openEntry}>
        <Routes>
          <Route path="/p/:key" element={<>{node}<Where /></>} />
        </Routes>
      </OpenEntryContext.Provider>
    </MemoryRouter>,
  );

const where = (): string => screen.getByLabelText('where').textContent ?? '';

describe('I-82 every record reference is a link', { tags: ['standard'] }, () => {
  it('an item id opens that item in the pane of this board', () => {
    mount(<RecordLink id="I-12" />);
    fireEvent.click(screen.getByRole('link', { name: 'I-12' }));
    expect(where()).toBe('/p/argus#I-12');
  });

  it('a typed code opens the item with the same number', () => {
    mount(<RecordLink id="E-12" />);
    fireEvent.click(screen.getByRole('link', { name: 'E-12' }));
    expect(where()).toBe('/p/argus#I-12');
  });

  it('a question, decision or note opens the entry pane and closes nothing else', () => {
    const openEntry = vi.fn();
    mount(<RecordLink id="Q-4" />, openEntry);
    fireEvent.click(screen.getByRole('link', { name: 'Q-4' }));
    expect(openEntry).toHaveBeenCalledWith('Q-4');
    expect(where()).toBe('/p/argus');
  });

  it('a test and a to-do open through the hash of the board', () => {
    mount(<><RecordLink id="T-3" /><RecordLink id="H-7" /></>);
    fireEvent.click(screen.getByRole('link', { name: 'T-3' }));
    expect(where()).toBe('/p/argus#T-3');
    fireEvent.click(screen.getByRole('link', { name: 'H-7' }));
    expect(where()).toBe('/p/argus#H-7');
  });

  it('an id of another board opens that board page at that item', () => {
    mount(<RecordLink id="rotp/I-105" />);
    const link = screen.getByRole('link', { name: 'rotp/I-105' });
    expect(link.getAttribute('href')).toBe('/p/rotp#I-105');
    fireEvent.click(link);
    expect(where()).toBe('/p/rotp#I-105');
  });

  it('opens from the keyboard: the link is a focusable anchor with an address', () => {
    mount(<RecordLink id="I-12" />);
    const link = screen.getByRole('link', { name: 'I-12' });
    link.focus();
    expect(document.activeElement).toBe(link);
    expect(link.getAttribute('href')).toBe('/p/argus#I-12');
  });

  it('links the ids inside a free text and leaves the words alone', () => {
    const { container } = mount(<p><LinkedText text="Blocked by I-3 until Q-2 is answered (see rotp/I-9), not UTF-8." /></p>);
    expect([...container.querySelectorAll('a')].map((link) => link.textContent)).toEqual(['I-3', 'Q-2', 'rotp/I-9']);
    expect(container.querySelector('p')?.textContent).toBe('Blocked by I-3 until Q-2 is answered (see rotp/I-9), not UTF-8.');
  });
});
