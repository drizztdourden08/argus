/* @layer renderer-app @kind logic */
import { NO_LOAD, STALE_AGENT_MS } from '../RegisteredStrip.constants';
import type { AgentLoad, RegistryAgent } from '../RegisteredStrip.type';

const isWorking = (agent: RegistryAgent): boolean => agent.status === 'working' && !agent.archivedAt && !agent.deletedAt;

const isQuiet = (agent: RegistryAgent, now: number): boolean => {
  const updated = agent.updatedAt ? Date.parse(agent.updatedAt) : Number.NaN;
  return agent.live?.stalled === true || (Number.isFinite(updated) && now - updated > STALE_AGENT_MS);
};

const agentLoads = (agents: RegistryAgent[], now: number): Map<string, AgentLoad> => {
  const loads = new Map<string, AgentLoad>();
  for (const agent of agents.filter(isWorking)) {
    const known = loads.get(agent.thread) ?? NO_LOAD;
    const quiet = isQuiet(agent, now);
    loads.set(agent.thread, { working: known.working + (quiet ? 0 : 1), stalled: known.stalled + (quiet ? 1 : 0) });
  }
  return loads;
};

export { agentLoads };
