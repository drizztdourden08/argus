/* @layer renderer-app @kind component */
import { Callout, CodeBlock, Text } from '@drizztdourden08/tessera';
import { useStepLog } from '../behavior/useStepLog';
import { LOG_EMPTY, LOG_READING } from '../changes.constants';
import type { StepLogBodyProps } from '../changes.type';

const StepLogBody = (props: StepLogBodyProps) => {
  const { boardKey, target } = props;
  const { data, error } = useStepLog(boardKey, target);
  if (error && data === null) return <Callout tone="danger">{`The log could not be read: ${error}.`}</Callout>;
  if (data === null) return <Text tone="muted">{LOG_READING}</Text>;
  if (data === '') return <Text tone="muted">{LOG_EMPTY}</Text>;
  return <CodeBlock code={data} language="text" wrap />;
};

export { StepLogBody };
