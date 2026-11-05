/* @layer renderer-app @kind test */
import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { MemoryRouter, useLocation } from 'react-router';
import { describe, expect, it } from 'vitest';
import { useSwitchBoard } from '../../src/shell/BoardSwitcher';

const wrapper = (entry: string) => ({ children }: { children: ReactNode }) => <MemoryRouter initialEntries={[entry]}>{children}</MemoryRouter>;

const useProbe = () => ({ go: useSwitchBoard(), location: useLocation() });

describe('I-106 useSwitchBoard', { tags: ['critical'] }, () => {
  it('pushes the other board and keeps the hash', () => {
    const { result } = renderHook(useProbe, { wrapper: wrapper('/p/rotp/#tests') });
    act(() => result.current.go('argus'));
    expect(result.current.location.pathname).toBe('/p/argus/');
    expect(result.current.location.hash).toBe('#tests');
  });

  it('lets the last switch win', () => {
    const { result } = renderHook(useProbe, { wrapper: wrapper('/p/rotp/') });
    act(() => {
      result.current.go('argus');
      result.current.go('brock');
    });
    expect(result.current.location.pathname).toBe('/p/brock/');
  });
});
