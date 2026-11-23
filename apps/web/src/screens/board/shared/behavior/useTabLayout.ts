/* @layer renderer-app @kind logic */
import { useStored } from '../../../../stored';

const useTabLayout = <T extends string>(tab: string, boardKey: string, choices: readonly T[], fallback: T): [T, (layout: T) => void] => {
  const [stored, write] = useStored(`argus.tab.${tab}.layout.${boardKey}`);
  const layout = choices.find((choice) => choice === stored) ?? fallback;
  return [layout, write];
};

export { useTabLayout };
