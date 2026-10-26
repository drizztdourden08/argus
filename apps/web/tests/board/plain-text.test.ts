/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { plainText } from '../../src/screens/board/plain-text';

describe('plain text of a clamped card', () => {
  it('removes the marks, keeps the words, one line', () => {
    expect(plainText('# Title\n\n**bold** and *it* and ~~del~~ and `code`\n- item one\n- [x] item two\n> quoted\n1. first')).toBe(
      'Title bold and it and del and code item one item two quoted first',
    );
    expect(plainText('[link text](https://a.test/x) and ![alt text](/files/a.png)')).toBe('link text and alt text');
    expect(plainText('```js\nconst a = 1;\n```\nafter')).toBe('const a = 1; after');
    expect(plainText('| a | b |\n|---|---|\n| 1 | 2 |')).toBe('a b 1 2');
    expect(plainText('\\*kept\\* snake_case_word __init__ 2*3*4')).toBe('*kept* snake_case_word init 2*3*4');
  });

  it('keeps unpaired stars literal and real emphasis goes', () => {
    const text = 'render Bash(cmd one *), Bash(cmd two *) and Bash(cmd three *) now; *real* and **bold**';
    expect(plainText(text)).toBe('render Bash(cmd one *), Bash(cmd two *) and Bash(cmd three *) now; real and bold');
  });

  it('passes plain text, empty values and numbers through', () => {
    expect(plainText('just text')).toBe('just text');
    expect(plainText('')).toBe('');
    expect(plainText(null)).toBe('');
    expect(plainText(42)).toBe('42');
  });

  it('never returns markup that was not in the text', () => {
    expect(plainText('<b>x</b>')).toBe('<b>x</b>');
  });

  it('stays quick on a hostile input', () => {
    const started = Date.now();
    plainText(`${'*a '.repeat(20000)}\n${'`'.repeat(5000)}`);
    expect(Date.now() - started).toBeLessThan(3000);
  });
});
