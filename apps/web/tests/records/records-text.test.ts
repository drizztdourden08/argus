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

  it('reads the kind, the board and the item number of a token', () => {
    expect(parseRecord('I-12')).toEqual({ kind: 'item', id: 'I-12', board: null });
    expect(parseRecord('E-12')).toEqual({ kind: 'item', id: 'I-12', board: null });
    expect(parseRecord('S-13')).toEqual({ kind: 'item', id: 'I-13', board: null });
    expect(parseRecord('rotp/I-105')).toEqual({ kind: 'item', id: 'I-105', board: 'rotp' });
    expect(parseRecord('Q-4')).toEqual({ kind: 'entry', id: 'Q-4', board: null });
    expect(parseRecord('D-2')).toEqual({ kind: 'entry', id: 'D-2', board: null });
    expect(parseRecord('N-1')).toEqual({ kind: 'entry', id: 'N-1', board: null });
    expect(parseRecord('H-7')).toEqual({ kind: 'todo', id: 'H-7', board: null });
    expect(parseRecord('T-3')).toEqual({ kind: 'test', id: 'T-3', board: null });
    expect(parseRecord('A-1')).toEqual({ kind: 'agent', id: 'A-1', board: null });
    expect(parseRecord('X-1')).toBeNull();
  });
});
