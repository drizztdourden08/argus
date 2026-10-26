/* @layer renderer-app @kind types */
import type { PostResult } from '../../../../data/api';
import type { TestOutcome } from '../../../../data/posts';
import type { WithId } from '../../../../data/react';
import type { RunEntry } from '../../../../data/runs';
import type { TestItem } from '../../../../data/types';
import type { BundleGroup, ClosedGroup, LaunchInfo, TestDoc } from './model';

interface BundleView {
  group: BundleGroup;
  launch: LaunchInfo;
}

interface TestsData {
  ready: boolean;
  tests: TestDoc[];
  bundles: BundleView[];
  closed: ClosedGroup[];
  runs: RunEntry[];
  items: WithId<TestItem>[];
}

interface OwnerPost {
  busy: boolean;
  run: (send: () => Promise<PostResult>, done: string) => Promise<string | null>;
}

interface ProofUpload {
  uploading: boolean;
  attach: (testId: string, file: File, caption: string) => Promise<string | null>;
}

interface RecordResult {
  busy: boolean;
  record: (testId: string, outcome: TestOutcome, text: string, shot: File | null) => Promise<string | null>;
}

interface Outcome {
  open: TestOutcome | null;
  text: string;
  shot: File | null;
  error: string | null;
  busy: boolean;
  choose: (outcome: TestOutcome) => void;
  setText: (text: string) => void;
  setShot: (file: File) => void;
  removeShot: () => void;
  cancel: () => void;
  send: () => void;
}

interface TestActions {
  busy: boolean;
  rerun: (testId: string) => Promise<void>;
  hand: (tag: string | null) => Promise<void>;
}

interface Folds {
  isOpen: (key: string, fallback: boolean) => boolean;
  set: (key: string, open: boolean) => void;
}

export type {
  BundleView, Folds, Outcome, OwnerPost, ProofUpload, RecordResult, TestActions, TestsData,
};
