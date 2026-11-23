/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { configKind } from '../model';
import type { ConfigRuleFieldProps } from '../config-editor.type';
import { ConfigRuleParam } from './ConfigRuleParam';

const ConfigRuleParams = (props: ConfigRuleFieldProps) => {
  const { rule, onChange } = props;
  return (
    <Flex gap="md" wrap>
      {configKind(rule.kind).params.map((field) => <ConfigRuleParam key={field.key} rule={rule} field={field} onChange={onChange} />)}
    </Flex>
  );
};

export { ConfigRuleParams };
