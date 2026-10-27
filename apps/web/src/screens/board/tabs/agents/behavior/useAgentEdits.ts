/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { RefObject } from 'react';
import type { Agent } from '../../../../../data/agents';
import { SOFT_INVERSE, editDoc, sendOp, softOp } from '../../../../../data/kanban';
import { SOFT_LABEL, useWriter } from '../../../shared';
import type { AgentEdits } from '../AgentsTab.type';

const useAgentEdits = (boardKey: string, latest: RefObject<Map<string, Agent>>): AgentEdits => {
  const write = useWriter();
  return useMemo(() => ({
    setField: (id, field, value) => {
      const agent = latest.current.get(id);
      if (!agent) return;
      void write(() => editDoc(boardKey, { c: 'agents', id, rev: agent.rev }, { field, prev: agent[field] ?? '', value }), `Saved ${id}.`);
    },
    setItem: (id, item) => {
      const agent = latest.current.get(id);
      if (!agent) return;
      const fields = { kind: 'edit', target: { c: 'agents', id }, set: { item }, prev: { item: agent.item ?? null }, baseRev: agent.rev };
      void write(() => sendOp(boardKey, fields), `Item of ${id} is now ${item ?? 'none'}.`);
    },
    soft: (kind, id) =>
      void write(
        () => softOp(boardKey, { kind, id, c: 'agents' }),
        `${SOFT_LABEL[kind]} ${id}.`,
        (undoes) => void write(() => softOp(boardKey, { kind: SOFT_INVERSE[kind], id, c: 'agents', undoes }), `Undone: ${id}.`),
      ),
  }), [boardKey, latest, write]);
};

export { useAgentEdits };
