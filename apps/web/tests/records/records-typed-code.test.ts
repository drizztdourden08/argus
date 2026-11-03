/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { ITEM_REGISTRY, ITEM_STORED, kindOf } from '../../src/data/registry';
import { itemCode } from '../../src/shell/records/behavior/item-code';
import { parseRecord } from '../../src/shell/records/behavior/parse-record';
import { resolveCode } from '../../src/shell/records/behavior/resolve-code';
import { shownCode } from '../../src/shell/records/behavior/shown-code';

const board: Record<string, { type?: string }> = {
  'I-12': { type: 'bug' },
  'I-13': { type: 'epic' },
  'I-14': {},
  'I-15': { type: 'test-plan' },
};

const lookup = (stored: string) => board[stored];

const must = (id: string) => {
  const found = kindOf(id);
  if (!found) throw new Error(`no kind ${id}`);
  return found;
};

describe('I-85 the typed code of an item comes from the registry', { tags: ['critical'] }, () => {
  it('stores every work item type under I and shows it with the letters of its type', () => {
    const types = ITEM_REGISTRY.types.filter((type) => type.collection === 'items');
    expect(types.length).toBeGreaterThan(0);
    for (const type of types) {
      expect(must(type.id).stored).toBe(ITEM_STORED);
      expect(itemCode({ id: 'I-7', type: type.id })).toBe(`${type.code}-7`);
    }
  });

  it('keeps the ids of the kinds that are not work items as they are', () => {
    expect(itemCode({ id: 'T-3', type: 'test-case' })).toBe('T-3');
    expect(shownCode(must('build'), 'web')).toBe('BL-web');
  });

  it('reads an item without a type as a task, and an unknown type as the stored id', () => {
    expect(itemCode({ id: 'I-5' })).toBe('K-5');
    expect(itemCode({ id: 'I-5', type: 'nothing-known' })).toBe('I-5');
  });
});

describe('I-85 a typed code resolves back to the stored id', { tags: ['critical'] }, () => {
  it('maps the letters of the type of the item to its stored id', () => {
    expect(resolveCode('B-12', lookup)).toBe('I-12');
    expect(resolveCode('E-13', lookup)).toBe('I-13');
    expect(resolveCode('K-14', lookup)).toBe('I-14');
    expect(resolveCode('TP-15', lookup)).toBe('I-15');
  });

  it('refuses the letters of another type', () => {
    expect(resolveCode('S-12', lookup)).toBeNull();
    expect(resolveCode('K-13', lookup)).toBeNull();
    expect(resolveCode('B-14', lookup)).toBeNull();
  });

  it('keeps the old stored form and the ids that are not items', () => {
    expect(resolveCode('I-12', lookup)).toBe('I-12');
    expect(resolveCode('T-4', lookup)).toBe('T-4');
    expect(resolveCode('Q-3', lookup)).toBe('Q-3');
    expect(resolveCode('words', lookup)).toBe('words');
  });

  it('takes the letters of a type for an item that is not on the board', () => {
    expect(resolveCode('B-99', lookup)).toBe('I-99');
  });

  it('is the inverse of the code an item is shown with', () => {
    for (const [stored, doc] of Object.entries(board)) {
      expect(resolveCode(itemCode({ id: stored, type: doc.type }), lookup)).toBe(stored);
    }
  });
});

describe('I-85 every written form of an item reaches the same record', { tags: ['critical'] }, () => {
  it('opens the stored id and the typed code on the one item', () => {
    expect(parseRecord('I-12')?.id).toBe('I-12');
    expect(parseRecord('B-12')?.id).toBe('I-12');
    expect(parseRecord('rotp/B-12')).toMatchObject({ id: 'I-12', board: 'rotp', code: 'B-12' });
  });
});
