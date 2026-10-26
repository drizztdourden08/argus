/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { recordResult } from '../../../../../data/posts';
import type { TestOutcome } from '../../../../../data/posts';
import { noteProblem } from '../model';
import type { RecordResult } from '../tests.type';
import { useOwnerPost } from './useOwnerPost';
import { useProofUpload } from './useProofUpload';

const useRecordResult = (boardKey: string): RecordResult => {
  const { busy, run } = useOwnerPost();
  const { attach } = useProofUpload(boardKey);
  const record = useCallback(async (testId: string, outcome: TestOutcome, text: string, shot: File | null): Promise<string | null> => {
    const problem = noteProblem(outcome, text);
    if (problem) return problem;
    const failure = await run(() => recordResult(boardKey, testId, outcome, text.trim()), `${testId} recorded as ${outcome}.`);
    if (failure) return failure;
    if (shot) await attach(testId, shot, text);
    return null;
  }, [boardKey, run, attach]);
  return { busy, record };
};

export { useRecordResult };
