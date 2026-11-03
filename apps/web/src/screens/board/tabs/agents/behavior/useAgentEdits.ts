/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { RefObject } from 'react';
import type { Agent } from '../../../../../data/agents';
import { SOFT_INVERSE, editDoc, sendOp, softOp } from '../../../../../data/kanban';
import { SOFT_LABEL, useWriter } from '../../../shared';
import type { AgentEdits } from '../AgentsTab.type';
import { agentItems } from '../model';

const useAgentEdits = (boardKey: string, latest: RefObject<Map<string, Agent>>): AgentEdits => {
  const write = useWriter();
  return useMemo(() => ({
    setField: (id, field, value) => {
      const agent = latest.current.get(id);
      if (!agent) return;
      void write(() => editDoc(boardKey, { c: 'agents', id, rev: agent.rev }, { field, prev: agent[field] ?? '', value }), `Saved ${id}.`);
    },
    setItems: (id, items) => {
      const agent = latest.current.get(id);
      if (!agent) return;
      const fields = { kind: 'edit', target: { c: 'agents', id }, set: { items }, prev: { items: agentItems(agent) }, baseRev: agent.rev };
      void write(() => sendOp(boardKey, fields), items.length === 0 ? `${id} has no item now.` : `Items of ${id}: ${items.join(', ')}.`);
    },
    setMaster: (id, master) => {
      const agent = latest.current.get(id);
      if (!agent) return;
      const fields = { kind: 'edit', target: { c: 'agents', id }, set: { master }, prev: { master: agent.master === true }, baseRev: agent.rev };
      void write(() => sendOp(boardKey, fields), master ? `${id} works on the master checkout.` : `${id} no longer works on the master checkout.`);
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
