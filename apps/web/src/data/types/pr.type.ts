/* @layer renderer-app @kind types */
import type { Replay } from '../remote';

type PullRequestStatus = 'open' | 'merged' | 'closed';

interface PullRequest {
  title: string;
  body?: string;
  head: string;
  base: string;
  headCommit?: string;
  status: PullRequestStatus;
  item?: string | null;
  checks?: string[];
  createdBy?: string;
  createdAt?: string;
  mergedAt?: string | null;
  mergeCommit?: string | null;
  replay?: Replay | null;
}

export type { PullRequest, PullRequestStatus };
