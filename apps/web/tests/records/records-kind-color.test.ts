/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { RegistryKind } from '../../src/data/registry';
import { kindMark } from '../../src/shell/records/behavior/kind-color';

const kindWith = (look: RegistryKind['look']): RegistryKind => ({ look }) as RegistryKind;

describe('I-85 the colour of a type mark', { tags: ['critical'] }, () => {
  it('uses the category colour of the type, before any tone', () => {
    expect(kindMark(kindWith({ color: 'violet', tone: 'danger' }))).toEqual({ color: 'violet' });
  });

  it('passes the status tone when the type has no colour', () => {
    for (const tone of ['danger', 'warning', 'info', 'success']) expect(kindMark(kindWith({ tone }))).toEqual({ tone });
  });

  it('leaves the mark in the default ink for a type with neither, or a tone the mark has not', () => {
    expect(kindMark(kindWith({}))).toEqual({});
    expect(kindMark(kindWith({ tone: 'neutral' }))).toEqual({});
  });
});
