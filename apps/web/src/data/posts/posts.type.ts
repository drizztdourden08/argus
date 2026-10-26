/* @layer renderer-app @kind types */
type TestOutcome = 'pass' | 'fail' | 'partial' | 'blocked';

interface ProofResult {
  test: string;
  proof: { n: number; url: string; caption: string };
}

interface SendDownInput {
  project: string;
  text: string;
  decision?: boolean;
}

export type { ProofResult, SendDownInput, TestOutcome };
