/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { RegistryKind } from '../../src/data/registry';
import { kindColor } from '../../src/shell/records/behavior/kind-color';

const kindWith = (look: RegistryKind['look']): RegistryKind => ({ look }) as RegistryKind;

describe('I-85 the colour of a type mark', { tags: ['critical'] }, () => {
  it('uses the category colour of the type', () => {
    expect(kindColor(kindWith({ color: 'violet' }))).toBe('violet');
  });

  it('maps a tone to the category colour that stands for it, when the type has no colour', () => {
    expect(kindColor(kindWith({ tone: 'danger' }))).toBe('rose');
    expect(kindColor(kindWith({ tone: 'warning' }))).toBe('amber');
    expect(kindColor(kindWith({ tone: 'info' }))).toBe('blue');
    expect(kindColor(kindWith({ tone: 'success' }))).toBe('green');
  });

  it('leaves the mark in the default ink for a type with neither', () => {
    expect(kindColor(kindWith({}))).toBeUndefined();
    expect(kindColor(kindWith({ tone: 'unknown' }))).toBeUndefined();
  });
});
