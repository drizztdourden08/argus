/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { errorMessage } from '../../../../data/api';
import { renameBoard } from '../../../../data/boards';
import { NAME_EMPTY, NAME_MAX, NAME_TOO_LONG } from '../BoardHeader.constants';
import type { Rename } from '../BoardHeader.type';

const checkName = (text: string): string | undefined => {
  const trimmed = text.trim();
  if (trimmed.length === 0) return NAME_EMPTY;
  return trimmed.length > NAME_MAX ? NAME_TOO_LONG : undefined;
};

const useRename = (id: string): Rename => {
  const [error, setError] = useState<string | null>(null);
  const commit = useCallback((text: string) => {
    setError(null);
    renameBoard(id, text.trim()).catch((failure: unknown) => setError(errorMessage(failure)));
  }, [id]);
  return { error, commit, validate: checkName };
};

export { useRename };
