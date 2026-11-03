/* @layer renderer-app @kind types */
type TestStatus = 'pending' | 'pass' | 'fail' | 'partial' | 'blocked' | 'moved';

type TestMode = 'auto' | 'manual';

interface TestResult {
  status: string;
  text: string;
  by: string;
  at: string;
}

interface TestProof {
  n: number;
  caption: string;
  url: string;
  at?: string;
}

interface TestSetup {
  label: string;
  text: string;
}

interface Test {
  item: string | null;
  thread: string | null;
  title: string;
  covers: string | null;
  build?: string | null;
  mode: TestMode;
  run: string | null;
  context: string | null;
  changed?: string[];
  setup?: TestSetup[];
  steps?: string[];
  pass?: string[];
  fail?: string[];
  notes?: string[];
  ac: number[];
  status: TestStatus;
  result: TestResult | null;
  runs?: TestResult[];
  proofs?: TestProof[];
  rank?: number;
  rev?: number;
  createdBy?: string | null;
  updatedBy?: string | null;
  subject?: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt?: string | null;
  deletedAt?: string | null;
}

export type { Test, TestResult };
