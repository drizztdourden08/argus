/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { setTitleMax } from '../../../../data/posts';
import type { PostResult } from '../../../../data/api';
import { useDoc } from '../../../../data/react';
import type { BoardMeta } from '../../../../data/types';
import { TITLE_LIMIT_DEFAULT, TITLE_LIMIT_MAX, TITLE_LIMIT_MIN, TITLE_LIMIT_SAVED } from '../BoardSettings.constants';
import type { TitleLimitChoice } from '../BoardSettings.type';
import { useSettingSave } from './useSettingSave';

const postLimit = (key: string, value: string | null): Promise<PostResult> => setTitleMax(key, value === null ? null : Number(value));

const useTitleLimit = (boardKey: string): TitleLimitChoice => {
  const saved = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc?.titleMax ?? TITLE_LIMIT_DEFAULT;
  const [value, setValue] = useState(saved);
  useEffect(() => setValue(saved), [saved]);
  const { busy, problem, save } = useSettingSave({ boardKey, post: postLimit, saved: TITLE_LIMIT_SAVED });
  const commit = useCallback(async () => {
    const next = Math.min(TITLE_LIMIT_MAX, Math.max(TITLE_LIMIT_MIN, Math.round(value)));
    setValue(next);
    if (next !== saved) await save(String(next));
  }, [value, saved, save]);
  return { value, busy, problem, changed: value !== saved, setValue, commit };
};

export { useTitleLimit };
