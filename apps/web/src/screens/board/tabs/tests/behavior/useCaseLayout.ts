/* @layer renderer-app @kind logic */
import { useStored } from '../../../shared';
import { LAYOUT_KEY } from '../tests.constants';
import type { CaseLayout } from '../tests.type';

const useCaseLayout = (): [CaseLayout, (layout: CaseLayout) => void] => {
  const [stored, write] = useStored(LAYOUT_KEY);
  return [stored === 'cards' ? 'cards' : 'rows', write];
};

export { useCaseLayout };
