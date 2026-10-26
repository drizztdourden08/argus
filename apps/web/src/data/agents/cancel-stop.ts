/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { postEntry } from '../posts';
import { ownerEntry } from '../posts';
import type { Agent } from './agents.type';

const cancelStop = (key: string, id: string, agent: Agent, undoes?: string): Promise<PostResult> =>
  postEntry(key, ownerEntry({
    kind: 'edit',
    target: { c: 'agents', id },
    set: { status: 'working' },
    prev: { status: agent.status },
    baseRev: agent.rev,
    undoes,
  }));

export { cancelStop };
