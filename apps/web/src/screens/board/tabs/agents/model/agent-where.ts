/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { kindOf } from '../../../../../data/registry';
import { folderOf } from './folder-of';
import { BUILD_FALLBACK_CODE, MASTER_TIP, MASTER_WORD, NO_WORKTREE_TEXT, NO_WORKTREE_TIP, WORKTREE_FALLBACK_CODE } from './model.constants';
import type { WhereLook } from './model.type';

const sourceTip = (agent: Agent): string => (agent.whereSource ? ` (${agent.whereSource})` : '');

const agentWhere = (agent: Agent): WhereLook => {
  const place = agent.where ?? null;
  if (agent.master === true || place === MASTER_WORD) {
    return { kind: 'master', text: MASTER_WORD, tip: MASTER_TIP, link: `${kindOf('build')?.code ?? BUILD_FALLBACK_CODE}-${MASTER_WORD}` };
  }
  if (place) {
    const code = kindOf('worktree')?.code ?? WORKTREE_FALLBACK_CODE;
    return { kind: 'worktree', text: `${code} ${folderOf(place)}`, tip: `${place}${sourceTip(agent)}`, link: `${code}-${folderOf(place)}` };
  }
  return { kind: 'unknown', text: NO_WORKTREE_TEXT, tip: NO_WORKTREE_TIP };
};

export { agentWhere };
