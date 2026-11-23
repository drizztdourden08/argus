/* @layer renderer-app @kind test */
import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { useTabLayout } from '../../src/screens/board/shared';

const CHOICES = ['list', 'cards'] as const;

beforeEach(() => localStorage.clear());

describe('I-215 useTabLayout', { tags: ['critical'] }, () => {
  it('falls back when nothing is stored or the stored value is not a choice', () => {
    expect(renderHook(() => useTabLayout('agents', 'argus', CHOICES, 'list')).result.current[0]).toBe('list');
    localStorage.setItem('argus.tab.agents.layout.argus', 'grid');
    expect(renderHook(() => useTabLayout('agents', 'argus', CHOICES, 'list')).result.current[0]).toBe('list');
  });

  it('stores the choice per board', () => {
    const { result } = renderHook(() => useTabLayout('agents', 'argus', CHOICES, 'list'));
    act(() => result.current[1]('cards'));
    expect(result.current[0]).toBe('cards');
    expect(localStorage.getItem('argus.tab.agents.layout.argus')).toBe('cards');
    expect(renderHook(() => useTabLayout('agents', 'other', CHOICES, 'list')).result.current[0]).toBe('list');
  });
});
