/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { parseRecord, splitRecords } from '../../src/shell';

const refsOf = (text: string): string[] =>
  splitRecords(text)
    .filter((part) => part.ref !== null)
    .map((part) => part.text);

describe('I-82 the text linker', { tags: ['standard'] }, () => {
  it('finds the ids of every kind and a board prefix, and keeps the text around them', () => {
    const parts = splitRecords('Wait for I-12 and Q-4, see rotp/I-105 or T-9.');
    expect(parts.map((part) => part.text).join('')).toBe('Wait for I-12 and Q-4, see rotp/I-105 or T-9.');
    expect(refsOf('Wait for I-12 and Q-4, see rotp/I-105 or T-9.')).toEqual(['I-12', 'Q-4', 'rotp/I-105', 'T-9']);
  });

  it('leaves other text alone', () => {
    const text = 'Use UTF-8, XT-5, T-5x, T-5-9, #T-5, t-5 and a plain sentence.';
    expect(splitRecords(text)).toEqual([{ text, ref: null }]);
  });

  it('gives an empty text no part', () => {
    expect(splitRecords('')).toEqual([]);
  });

  it('reads the kind, the board and the stored id of a token', () => {
    const read = (token: string) => {
      const ref = parseRecord(token);
      return ref && { kind: ref.kind, code: ref.code, id: ref.id, board: ref.board };
    };
    expect(read('I-12')).toMatchObject({ code: 'I-12', id: 'I-12', board: null });
    expect(read('E-12')).toEqual({ kind: 'epic', code: 'E-12', id: 'I-12', board: null });
    expect(read('S-13')).toEqual({ kind: 'story', code: 'S-13', id: 'I-13', board: null });
    expect(read('rotp/I-105')).toMatchObject({ id: 'I-105', board: 'rotp' });
    expect(read('Q-4')).toEqual({ kind: 'question', code: 'Q-4', id: 'Q-4', board: null });
    expect(read('D-2')).toEqual({ kind: 'decision', code: 'D-2', id: 'D-2', board: null });
    expect(read('H-7')).toEqual({ kind: 'todo', code: 'H-7', id: 'H-7', board: null });
    expect(read('T-3')).toEqual({ kind: 'test-case', code: 'T-3', id: 'T-3', board: null });
    expect(read('A-1')).toEqual({ kind: 'agent', code: 'A-1', id: 'A-1', board: null });
    expect(read('Z-1')).toBeNull();
  });
});
