/* @layer renderer-app @kind logic */
import type { ViewChoice } from '../BoardTabsRow.type';

const pickChoice = <Id extends string>(options: readonly ViewChoice<Id>[], id: string | null): Id | undefined =>
  options.find((option) => option.id === id)?.id;

export { pickChoice };
