/* @layer renderer-app @kind component */
import { Stack, Stepper } from '@drizztdourden08/tessera';
import { useWizardView } from '../behavior/useWizardView';
import { wizardSteps } from '../model';
import { WIZARD_LABEL } from '../switch.constants';
import type { ProcessDocProps } from '../switch.type';
import { StepBody } from './StepBody';

const ProcessWizard = (props: ProcessDocProps) => {
  const { doc } = props;
  const model = wizardSteps(doc);
  const { shown, show } = useWizardView(model.currentId);
  const canSelect = (id: string) => model.steps.find((step) => step.id === id)?.state !== 'upcoming';
  return (
    <Stack gap="sm">
      <Stepper
        steps={model.steps}
        currentId={model.currentId}
        complete={model.complete}
        orientation="horizontal"
        size="sm"
        reserve="summaries"
        canSelect={canSelect}
        onSelect={show}
        label={WIZARD_LABEL}
      />
      {shown !== undefined && <StepBody doc={doc} stage={shown} />}
    </Stack>
  );
};

export { ProcessWizard };
