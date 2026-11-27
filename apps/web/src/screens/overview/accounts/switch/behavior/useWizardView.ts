/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { SwitchStage } from '../../../../../data/types';
import { STAGES } from '../switch.constants';
import type { WizardPick, WizardView } from '../switch.type';

const useWizardView = (currentId: SwitchStage | undefined): WizardView => {
  const [picked, setPicked] = useState<WizardPick | null>(null);
  const shown = picked !== null && picked.from === currentId ? picked.stage : currentId;
  const show = useCallback((id: string) => {
    const stage = STAGES.find((candidate) => candidate === id);
    if (stage !== undefined) setPicked({ stage, from: currentId });
  }, [currentId]);
  return { shown, show };
};

export { useWizardView };
