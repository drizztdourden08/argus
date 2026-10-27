/* @layer renderer-app @kind component */
import { Card, Flex, Markdown, Text } from '@drizztdourden08/tessera';
import { RESULT_TONES } from '../tests.constants';
import type { ResultBlockProps } from '../tests-drawer-props.type';
import { ResultMeta } from './ResultMeta';

const ResultBlock = (props: ResultBlockProps) => {
  const { test } = props;
  const text = test.result?.text;
  return (
    <Card role="group" aria-label="Result" title="Result" tone={RESULT_TONES[test.status] ?? 'neutral'} actions={<ResultMeta test={test} />}>
      {text ? <Markdown size="sm">{text}</Markdown> : <Flex justify="center"><Text tone="muted">Not run yet.</Text></Flex>}
    </Card>
  );
};

export { ResultBlock };
