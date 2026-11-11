/* @layer renderer-app @kind component */
import { Inline, Tag } from '@drizztdourden08/tessera';
import type { RuleChipsProps } from '../accounts.type';
import { ruleChips } from '../model';

const RuleChips = (props: RuleChipsProps) => (
  <Inline gap="xs" wrap>
    {ruleChips(props.account.rules).map((chip) => <Tag key={chip}>{chip}</Tag>)}
  </Inline>
);

export { RuleChips };
