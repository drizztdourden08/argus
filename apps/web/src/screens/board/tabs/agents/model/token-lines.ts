/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';

const count = (value: number | undefined): string => Number(value ?? 0).toLocaleString('en-US');

const tokenLines = (agent: Agent): string[] => {
  const live = agent.live;
  if (!live) return [];
  const tokens = live.tokens ?? {};
  const lines = [
    `Turns ${count(live.turns)}, tool calls ${count(live.toolCalls)}`,
    `Tokens: ${count(tokens.input)} in, ${count(tokens.output)} out, ${count(tokens.cacheRead)} cache read`,
  ];
  return live.modelUsed ? [...lines, `Running on ${live.modelUsed}`] : lines;
};

export { tokenLines };
