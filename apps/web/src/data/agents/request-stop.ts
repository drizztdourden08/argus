/* @layer renderer-app @kind logic */
import { postEntry } from '../posts';
import { ownerEntry } from '../posts';
import type { Agent, StopSent } from './agents.type';

const requestStop = async (key: string, id: string, agent: Agent): Promise<StopSent> => {
  const entry = ownerEntry({
    kind: 'edit',
    target: { c: 'agents', id },
    set: { status: 'stop-requested' },
    prev: { status: agent.status },
    baseRev: agent.rev,
  });
  const result = await postEntry(key, entry);
  return { entryId: String(entry.id), result };
};

export { requestStop };
