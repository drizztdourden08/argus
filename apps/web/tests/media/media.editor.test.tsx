/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { checkField, richTextCount } from '../../src/data/content';
import { picturesOf } from '../../src/data/media';
import { EditableMarkdown } from '../../src/screens/board/shared';
import { inBoard } from '../markdown/render-markdown';

afterEach(cleanup);

const URL_OF_ONE = '/files/demo/proofs/I-1-1.png';

const pictures = picturesOf([{ n: 1, url: URL_OF_ONE, caption: 'the form' }]);

const problem = (text: string): string | null => {
  const verdict = checkField('description', text, { images: [1] });
  return verdict.ok ? null : verdict.message;
};

const mount = (value: string, onSave: (text: string) => void) =>
  render(
    inBoard(
      <EditableMarkdown
        noun="description"
        value={value}
        empty="add a description"
        limit={500}
        problem={problem}
        media={{ pictures, count: richTextCount, onImageFiles: () => Promise.resolve('\n![the form](img:1)\n') }}
        onSave={onSave}
      />,
    ),
  );

const LONE = 'This is a plain paragraph that runs on well past the short idea limit of a hundred and twenty characters so it must be refused.';

describe('P8 the rich description field', { tags: ['critical'] }, () => {
  it('refuses a lone paragraph before saving and says how to write it, then saves bullets with a screenshot', () => {
    const onSave = vi.fn();
    mount('', onSave);
    fireEvent.click(screen.getByRole('button', { name: 'Edit description' }));
    const box = screen.getByRole('textbox', { name: 'Description' });
    fireEvent.change(box, { target: { value: LONE } });
    fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
    expect(screen.getByText(/lone paragraph is not accepted/)).toBeTruthy();
    expect(onSave).not.toHaveBeenCalled();
    fireEvent.change(box, { target: { value: '- the form\n![the form](img:1)' } });
    fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
    expect(onSave).toHaveBeenCalledWith('- the form\n![the form](img:1)');
  });

  it('draws img:n as the stored picture and opens it in the zoom viewer', () => {
    const { container } = mount('- the form\n![the form](img:1)', vi.fn());
    expect(container.querySelector('img')?.getAttribute('src')).toBe(URL_OF_ONE);
    fireEvent.click(screen.getByRole('button', { name: 'the form' }));
    const dialog = document.body.querySelector('[role="dialog"]');
    expect(dialog).not.toBeNull();
    expect(dialog?.querySelector('img')?.getAttribute('src')).toBe(URL_OF_ONE);
  });
});
