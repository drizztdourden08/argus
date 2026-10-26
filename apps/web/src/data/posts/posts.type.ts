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

type OwnerFields = {
  kind: string;
  text?: string;
  replyTo?: string | null;
  item?: string | null;
  thread?: string | null;
};

export type { OwnerFields, ProofResult, SendDownInput, TestOutcome };
