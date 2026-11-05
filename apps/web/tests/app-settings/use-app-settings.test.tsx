/* @layer renderer-app @kind test */
import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { AppSettingsContext } from '../../src/shell/AppSettings/behavior/app-settings-context';
import { useAppSettings } from '../../src/shell/AppSettings/behavior/useAppSettings';
import { useAppSettingsState } from '../../src/shell/AppSettings/behavior/useAppSettingsState';

const api = { open: true, section: 'system' as const, openSettings: vi.fn(), close: vi.fn(), setSection: vi.fn() };

const wrapper = ({ children }: { children: ReactNode }) => <AppSettingsContext.Provider value={api}>{children}</AppSettingsContext.Provider>;

describe('I-111 useAppSettings', { tags: ['critical'] }, () => {
  it('starts closed on Profile and opens the section asked for', () => {
    const { result } = renderHook(useAppSettingsState);
    expect(result.current).toMatchObject({ open: false, section: 'profile' });
    act(() => result.current.openSettings('system'));
    expect(result.current).toMatchObject({ open: true, section: 'system' });
  });

  it('reopens on the last section when none is given', () => {
    const { result } = renderHook(useAppSettingsState);
    act(() => result.current.openSettings('appearance'));
    act(() => result.current.close());
    expect(result.current.open).toBe(false);
    act(() => result.current.openSettings());
    expect(result.current).toMatchObject({ open: true, section: 'appearance' });
  });

  it('changes the section while open', () => {
    const { result } = renderHook(useAppSettingsState);
    act(() => result.current.openSettings());
    act(() => result.current.setSection('notifications'));
    expect(result.current.section).toBe('notifications');
  });

  it('reads the api inside a provider and refuses outside one', () => {
    expect(renderHook(useAppSettings, { wrapper }).result.current).toBe(api);
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
    expect(() => renderHook(useAppSettings)).toThrow(/AppSettingsProvider/);
  });
});
