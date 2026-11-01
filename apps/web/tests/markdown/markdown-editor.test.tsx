/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import { BoardMarkdownEditor } from '../../src/shell';
import { inBoard } from './render-markdown';

afterEach(cleanup);

interface Boxed {
  area: HTMLTextAreaElement;
  submitted: string[];
}

const Harness = ({ start, onSubmit }: { start: string; onSubmit: (value: string) => void }) => {
  const [value, setValue] = useState(start);
  return <BoardMarkdownEditor value={value} onChange={setValue} onSubmit={onSubmit} aria-label="Text" layout="tabs" />;
};

const mount = (start: string, from = 0, to = from): Boxed => {
  const submitted: string[] = [];
  render(inBoard(<Harness start={start} onSubmit={(value) => submitted.push(value)} />));
  const area = screen.getByLabelText<HTMLTextAreaElement>('Text');
  area.setSelectionRange(from, to);
  return { area, submitted };
};

const press = (name: string): void => {
  fireEvent.click(screen.getByRole('button', { name }));
};

describe('S1 markdown editor toolbar', { tags: ['standard'] }, () => {
  it('bold and italic wrap the selection and unwrap it again', () => {
    const { area } = mount('word', 0, 4);
    press('Bold');
    expect(area.value).toBe('**word**');
    area.setSelectionRange(2, 6);
    press('Bold');
    expect(area.value).toBe('word');
    area.setSelectionRange(0, 4);
    press('Italic');
    expect(area.value).toBe('*word*');
  });

  it('bold without a selection leaves the caret between the marks', () => {
    const { area } = mount('');
    press('Bold');
    expect(area.value).toBe('****');
    expect([area.selectionStart, area.selectionEnd]).toEqual([2, 2]);
  });

  it('code is inline for one line and a fence for several', () => {
    const one = mount('one', 0, 3);
    press('Inline code');
    expect(one.area.value).toBe('`one`');
    cleanup();
    const several = mount('a\nb', 0, 3);
    press('Code block');
    expect(several.area.value).toBe('```\na\nb\n```');
  });

  it('link wraps the selection and leaves the address to type over', () => {
    const { area } = mount('docs', 0, 4);
    press('Link');
    expect(area.value).toBe('[docs](https://)');
    expect(area.value.slice(area.selectionStart, area.selectionEnd)).toBe('https://');
  });

  it('bullet and numbered lines are added, and applying again removes them', () => {
    const { area } = mount('a\nb', 0, 3);
    press('Bulleted list');
    expect(area.value).toBe('- a\n- b');
    area.setSelectionRange(0, area.value.length);
    press('Bulleted list');
    expect(area.value).toBe('a\nb');
    area.setSelectionRange(0, area.value.length);
    press('Numbered list');
    expect(area.value).toBe('1. a\n2. b');
  });
});

describe('S1 markdown editor quote and focus', { tags: ['standard'] }, () => {
  it('Quote puts a prefix before every selected line, and a second press removes it', () => {
    const { area } = mount('a\nb', 0, 3);
    press('Quote');
    expect(area.value).toBe('> a\n> b');
    area.setSelectionRange(0, area.value.length);
    press('Quote');
    expect(area.value).toBe('a\nb');
  });

  it('autoFocus puts the caret at the end of the text', () => {
    render(inBoard(<BoardMarkdownEditor value="hello" onChange={() => undefined} aria-label="Text" autoFocus />));
    const area = screen.getByLabelText<HTMLTextAreaElement>('Text');
    expect(document.activeElement).toBe(area);
    expect([area.selectionStart, area.selectionEnd]).toEqual([5, 5]);
  });
});

describe('S1 markdown editor panes', { tags: ['standard'] }, () => {
  it('Preview renders the draft with the markdown renderer and Write brings the text back', () => {
    const { area } = mount('**bold** text');
    fireEvent.click(screen.getByRole('tab', { name: 'Preview' }));
    expect(screen.getByRole('region', { name: 'Preview' }).querySelector('strong')?.textContent).toBe('bold');
    fireEvent.click(screen.getByRole('tab', { name: 'Write' }));
    expect(area.value).toBe('**bold** text');
  });
});

describe('S1 markdown editor keys', { tags: ['standard'] }, () => {
  it('Enter never submits and Ctrl or Cmd with Enter does', () => {
    const { area, submitted } = mount('text');
    fireEvent.keyDown(area, { key: 'Enter' });
    fireEvent.keyDown(area, { key: 'Enter', shiftKey: true });
    expect(submitted).toEqual([]);
    fireEvent.keyDown(area, { key: 'Enter', ctrlKey: true });
    fireEvent.keyDown(area, { key: 'Enter', metaKey: true });
    expect(submitted).toEqual(['text', 'text']);
  });

  it('Ctrl+B and Ctrl+I mark the selection, and unknown keys change nothing', () => {
    const { area } = mount('word', 0, 4);
    fireEvent.keyDown(area, { key: 'b', ctrlKey: true });
    expect(area.value).toBe('**word**');
    fireEvent.keyDown(area, { key: 'q', ctrlKey: true });
    fireEvent.keyDown(area, { key: 'Tab' });
    expect(area.value).toBe('**word**');
  });
});

describe('S1 markdown editor list keys', { tags: ['standard'] }, () => {
  it('Enter continues a list and ends it on an empty item', () => {
    const { area } = mount('- a', 3);
    fireEvent.keyDown(area, { key: 'Enter' });
    expect(area.value).toBe('- a\n- ');
  });

  it('indent and outdent move list lines by two spaces', () => {
    const { area } = mount('- a', 3);
    fireEvent.keyDown(area, { key: 'Tab' });
    expect(area.value).toBe('  - a');
    fireEvent.keyDown(area, { key: 'Tab', shiftKey: true });
    expect(area.value).toBe('- a');
  });

  it('Backspace after the marker of an empty item takes the marker away', () => {
    const { area } = mount('- ', 2);
    fireEvent.keyDown(area, { key: 'Backspace' });
    expect(area.value).toBe('');
  });

  it('a space after "- []" becomes a checkbox item', () => {
    const { area } = mount('- []', 4);
    fireEvent.keyDown(area, { key: ' ' });
    expect(area.value).toBe('- [ ] ');
  });
});
