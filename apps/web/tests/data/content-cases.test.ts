/* @layer renderer-app @kind test */
import caseFile from '@argus/registry/content-cases.json';
import { describe, expect, it } from 'vitest';
import { checkField } from '../../src/data/content';
import type { RefusalKey } from '../../src/data/content';
import { ITEM_REGISTRY, fieldOf } from '../../src/data/registry';

interface CaseRow {
  name: string;
  field: string;
  value: unknown;
  context?: { images?: number[]; ids?: string[]; threads?: string[] };
  ok: boolean;
  result?: unknown;
  key?: RefusalKey;
}

const rowsOf = (raw: unknown): CaseRow[] => (raw as { cases: CaseRow[] }).cases;

const ROWS = rowsOf(caseFile);

describe('K2 the content rules agree with the Rust checker on the shared case file', { tags: ['critical'] }, () => {
  it('holds every case the Rust side runs', () => {
    expect(ROWS.length).toBeGreaterThan(100);
  });

  it.each(ROWS.map((row) => [row.name, row] as const))('%s', (_name, row) => {
    const verdict = checkField(row.field, row.value, row.context ?? {});
    if (!row.ok) {
      expect(verdict.ok).toBe(false);
      expect(verdict.ok ? null : verdict.key).toBe(row.key);
      return;
    }
    expect(verdict.ok).toBe(true);
    if ('result' in row) expect(verdict.ok ? verdict.value : null).toEqual(row.result);
  });

  it('covers every content kind and names only fields the registry has', () => {
    const kinds = new Set(ROWS.flatMap((row) => fieldOf(row.field)?.content ?? []));
    const known = ROWS.filter((row) => row.key !== 'unknown-field').every((row) => fieldOf(row.field) !== undefined);
    expect(known).toBe(true);
    expect([...kinds].sort()).toEqual(['date', 'enum', 'flag', 'line', 'person', 'records', 'ref', 'refs', 'rich', 'slug']);
    expect(Object.keys(ITEM_REGISTRY.content).sort()).toEqual(['date', 'enum', 'flag', 'line', 'person', 'records', 'ref', 'refs', 'rich', 'slug']);
  });
});

describe('K4 the content messages tell the user what to do', { tags: ['critical'] }, () => {
  it('words a title over the limit like the server does', () => {
    const verdict = checkField('title', 'a'.repeat(81));
    expect(verdict).toEqual({ ok: false, key: 'too-long', message: 'title is 81 chars, the limit is 80. Shorten it.' });
  });

  it('refuses a lone paragraph with the bullet example', () => {
    const verdict = checkField('description', `${'word '.repeat(30)}end`);
    expect(verdict.ok ? '' : verdict.message).toContain('a lone paragraph is not accepted. Example');
  });

  it('counts 500 characters of bullet text and refuses 501 with the count', () => {
    const ten = Array.from({ length: 10 }, () => `- ${'x'.repeat(50)}`).join('\n');
    expect(checkField('description', ten).ok).toBe(true);
    const over = checkField('description', `${ten}\n- x`);
    expect(over.ok ? '' : over.message).toContain('501');
  });

  it('accepts bullets under a lead line and an image the record has', () => {
    expect(checkField('description', 'Make the lobby clearer\n- seats show who joined\n- approve button is visible').ok).toBe(true);
    expect(checkField('description', '- see\n![lobby](img:1)', { images: [1] }).ok).toBe(true);
  });

  it('answers an unknown field name that is an object key with an unknown field', () => {
    const verdict = checkField('constructor', 'x');
    expect(verdict.ok ? '' : verdict.key).toBe('unknown-field');
  });
});
