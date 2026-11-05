/* @layer renderer-app @kind test */
import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setProfile } from '../../src/data/posts';
import { useProfileChoice } from '../../src/shell/AppSettings/behavior/useProfileChoice';

const profile = vi.hoisted(() => ({ value: { name: 'You', initials: 'ME', avatar: null as string | null } }));

vi.mock('../../src/data/posts', () => ({ setProfile: vi.fn() }));
vi.mock('../../src/owner/behavior/useOwnerProfile', () => ({ useOwnerProfile: () => profile.value }));
vi.mock('@drizztdourden08/tessera', async (original) => ({ ...(await original<object>()), toast: vi.fn() }));

beforeEach(() => {
  profile.value = { name: 'You', initials: 'ME', avatar: null };
  vi.mocked(setProfile).mockReset();
  vi.mocked(setProfile).mockResolvedValue(null);
});

describe('I-111 useProfileChoice', { tags: ['critical'] }, () => {
  it('shows the mode of the saved avatar', () => {
    profile.value = { ...profile.value, avatar: 'initials' };
    expect(renderHook(useProfileChoice).result.current.mode).toBe('initials');
    profile.value = { ...profile.value, avatar: 'brand:argus' };
    const { result } = renderHook(useProfileChoice);
    expect(result.current).toMatchObject({ mode: 'brand', brand: 'argus' });
  });

  it('saves Initials at once and Default as a reset', async () => {
    const { result } = renderHook(useProfileChoice);
    await act(() => result.current.chooseMode('initials'));
    expect(setProfile).toHaveBeenLastCalledWith({ avatar: 'initials' });
    await act(() => result.current.chooseMode('default'));
    expect(setProfile).toHaveBeenLastCalledWith({ avatar: null });
  });

  it('opens the brand picker without saving', async () => {
    const { result } = renderHook(useProfileChoice);
    await act(() => result.current.chooseMode('brand'));
    expect(result.current.mode).toBe('brand');
    expect(setProfile).not.toHaveBeenCalled();
  });

  it('trims the name and cuts the initials to two, empty resets', async () => {
    const { result } = renderHook(useProfileChoice);
    await act(() => result.current.saveName('  Ada  '));
    expect(setProfile).toHaveBeenLastCalledWith({ name: 'Ada' });
    await act(() => result.current.saveName('  '));
    expect(setProfile).toHaveBeenLastCalledWith({ name: null });
    await act(() => result.current.saveInitials('abc'));
    expect(setProfile).toHaveBeenLastCalledWith({ initials: 'ab' });
  });

  it('keeps the refusal under the control', async () => {
    vi.mocked(setProfile).mockResolvedValue('initials must be two letters or digits');
    const { result } = renderHook(useProfileChoice);
    await act(() => result.current.saveInitials('?'));
    expect(result.current.problem).toBe('initials must be two letters or digits');
  });
});
