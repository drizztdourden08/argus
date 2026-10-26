/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { imageProblem } from '../../../../../data/api';
import type { TestOutcome } from '../../../../../data/posts';
import type { NoteDraft } from '../tests.type';
import { useRecordResult } from './useRecordResult';

const useNoteDraft = (boardKey: string, testId: string, onDone: () => void): NoteDraft => {
  const { busy, record } = useRecordResult(boardKey);
  const [text, setText] = useState('');
  const [shot, setShotFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);
  const setShot = (file: File) => {
    const problem = imageProblem(file);
    setError(problem);
    if (!problem) setShotFile(file);
  };
  const post = (outcome: TestOutcome) => {
    setError(null);
    void record(testId, outcome, text, shot).then((failure) => (failure ? setError(failure) : onDone()));
  };
  return { text, shot, error, busy, setText, setShot, removeShot: () => setShotFile(null), post };
};

export { useNoteDraft };
