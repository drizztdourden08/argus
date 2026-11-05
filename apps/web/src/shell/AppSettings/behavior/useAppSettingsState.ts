/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useState } from 'react';
import { FIRST_SECTION } from '../AppSettings.constants';
import type { AppSettingsApi, AppSettingsState, SectionId } from '../AppSettings.type';

const useAppSettingsState = (): AppSettingsApi => {
  const [state, setState] = useState<AppSettingsState>({ open: false, section: FIRST_SECTION });
  const openSettings = useCallback((section?: SectionId) => {
    setState((last) => ({ open: true, section: section ?? last.section }));
  }, []);
  const close = useCallback(() => setState((last) => ({ ...last, open: false })), []);
  const setSection = useCallback((section: SectionId) => setState((last) => ({ ...last, section })), []);
  return useMemo(() => ({ ...state, openSettings, close, setSection }), [state, openSettings, close, setSection]);
};

export { useAppSettingsState };
