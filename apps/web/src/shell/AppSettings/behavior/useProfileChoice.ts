/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import { useCallback, useState } from 'react';
import { BRAND_PREFIX } from '../../../avatar';
import { fileToIcon } from '../../../board-icon';
import { writeErrorText } from '../../../data/api';
import { setProfile } from '../../../data/posts';
import type { ProfileFields } from '../../../data/posts';
import { INITIALS_AVATAR, resolveOwnerFace, useOwnerProfile } from '../../../owner';
import { INITIALS_LENGTH, PROFILE_SAVED } from '../AppSettings.constants';
import type { AvatarMode, ProfileChoice } from '../AppSettings.type';

const MODE_OF_KIND: Record<string, AvatarMode> = { default: 'default', initials: 'initials', brand: 'brand', image: 'upload' };

const useProfileChoice = (): ProfileChoice => {
  const profile = useOwnerProfile();
  const face = resolveOwnerFace(profile);
  const [picked, setPicked] = useState<AvatarMode | null>(null);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const save = useCallback(async (fields: ProfileFields) => {
    setBusy(true);
    try {
      const refusal = await setProfile(fields);
      setProblem(refusal);
      if (!refusal) toast({ variant: 'success', message: PROFILE_SAVED });
    } catch (error) {
      setProblem(writeErrorText(error));
    } finally {
      setBusy(false);
    }
  }, []);
  const reset = useCallback(async () => {
    setPicked(null);
    await save({ avatar: null });
  }, [save]);
  const chooseMode = useCallback(async (mode: AvatarMode) => {
    if (mode === 'default') return reset();
    setPicked(mode);
    if (mode === 'initials') await save({ avatar: INITIALS_AVATAR });
  }, [reset, save]);
  const pickBrand = useCallback((app: BrandApp) => save({ avatar: `${BRAND_PREFIX}${app}` }), [save]);
  const upload = useCallback(async (files: File[]) => {
    if (!files[0]) return;
    const result = await fileToIcon(files[0]);
    if (result.ok) await save({ avatar: result.url });
    else setProblem(result.reason);
  }, [save]);
  const saveName = useCallback((value: string) => save({ name: value.trim() || null }), [save]);
  const saveInitials = useCallback((value: string) => save({ initials: value.trim().slice(0, INITIALS_LENGTH) || null }), [save]);
  const brand = face.kind === 'brand' ? face.app : null;
  const mode = picked ?? MODE_OF_KIND[face.kind] ?? 'default';
  return { ...profile, brand, mode, busy, problem, chooseMode, pickBrand, upload, reset, saveName, saveInitials };
};

export { useProfileChoice };
