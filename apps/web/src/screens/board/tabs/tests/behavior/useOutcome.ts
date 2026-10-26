/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { TestOutcome } from '../../../../../data/posts';
import { imageProblem } from '../model';
import type { Outcome } from '../tests.type';
import { useRecordResult } from './useRecordResult';

const useOutcome = (boardKey: string, testId: string): Outcome => {
  const { busy, record } = useRecordResult(boardKey);
  const [open, setOpen] = useState<TestOutcome | null>(null);
  const [text, setText] = useState('');
  const [shot, setShotFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reset = () => {
    setOpen(null);
    setText('');
    setShotFile(null);
    setError(null);
  };
  const postPass = () => {
    reset();
    void record(testId, 'pass', '', null).then((failure) => failure && toast({ variant: 'danger', message: failure }));
  };
  const choose = (outcome: TestOutcome) => {
    if (outcome === 'pass') postPass();
    else setOpen(outcome);
  };
  const send = () => {
    if (!open) return;
    setError(null);
    void record(testId, open, text, shot).then((failure) => (failure ? setError(failure) : reset()));
  };
  const setShot = (file: File) => {
    const problem = imageProblem(file);
    setError(problem);
    if (!problem) setShotFile(file);
  };
  return { open, text, shot, error, busy, choose, setText, setShot, removeShot: () => setShotFile(null), cancel: reset, send };
};

export { useOutcome };
