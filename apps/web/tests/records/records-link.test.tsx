/* @layer renderer-app @kind test */
import { Text } from '@drizztdourden08/tessera';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router';
import { afterEach, describe, expect, it } from 'vitest';
import { LinkedText, RecordLink } from '../../src/shell';

afterEach(cleanup);

const Where = () => {
  const { pathname, search, hash } = useLocation();
  return <Text as="output" aria-label="where">{`${pathname}${search}${hash}`}</Text>;
};

const mount = (node: ReactNode) =>
  render(
    <MemoryRouter initialEntries={['/p/argus']}>
      <Routes>
        <Route path="/p/:key" element={<>{node}<Where /></>} />
      </Routes>
    </MemoryRouter>,
  );

const where = (): string => screen.getByLabelText('where').textContent;

describe('I-85 every record reference is a link that opens the one pane', { tags: ['critical'] }, () => {
  it('an item id opens that item in the pane of this board', () => {
    mount(<RecordLink id="I-12" />);
    fireEvent.click(screen.getByRole('link', { name: 'I-12' }));
    expect(where()).toBe('/p/argus?open=I-12');
  });

  it('a typed code opens the record under that code', () => {
    mount(<RecordLink id="E-12" />);
    fireEvent.click(screen.getByRole('link', { name: 'E-12' }));
    expect(where()).toBe('/p/argus?open=E-12');
  });

  it('every kind opens the same way, never changing the board tab', () => {
    mount(<><RecordLink id="Q-4" /><RecordLink id="T-3" /><RecordLink id="H-7" /><RecordLink id="A-2" /></>);
    for (const code of ['Q-4', 'T-3', 'H-7', 'A-2']) {
      fireEvent.click(screen.getByRole('link', { name: code }));
      expect(where()).toBe(`/p/argus?open=${code}`);
    }
  });

  it('an id of another board opens that board page at that record', () => {
    mount(<RecordLink id="rotp/I-105" />);
    const link = screen.getByRole('link', { name: 'rotp/I-105' });
    expect(link.getAttribute('href')).toBe('/p/rotp?open=I-105');
    fireEvent.click(link);
    expect(where()).toBe('/p/rotp?open=I-105');
  });

  it('opens from the keyboard: the link is a focusable anchor with an address', () => {
    mount(<RecordLink id="I-12" />);
    const link = screen.getByRole('link', { name: 'I-12' });
    link.focus();
    expect(document.activeElement).toBe(link);
    expect(link.getAttribute('href')).toBe('/p/argus?open=I-12');
  });

  it('links both a typed code and the stored id inside a free text', () => {
    const { container } = mount(<Text as="p"><LinkedText text="See B-12 and I-12, not B-12x." /></Text>);
    const links = [...container.querySelectorAll('a')];
    expect(links.map((link) => link.textContent)).toEqual(['B-12', 'I-12']);
    fireEvent.click(links[0] as HTMLElement);
    expect(where()).toBe('/p/argus?open=B-12');
  });

  it('links the ids inside a free text and leaves the words alone', () => {
    const { container } = mount(<Text as="p"><LinkedText text="Blocked by I-3 until Q-2 is answered (see rotp/I-9), not UTF-8." /></Text>);
    expect([...container.querySelectorAll('a')].map((link) => link.textContent)).toEqual(['I-3', 'Q-2', 'rotp/I-9']);
    expect(container.querySelector('p')?.textContent).toBe('Blocked by I-3 until Q-2 is answered (see rotp/I-9), not UTF-8.');
  });
});
