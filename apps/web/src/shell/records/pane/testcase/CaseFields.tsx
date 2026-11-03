/* @layer renderer-app @kind component */
import { FieldFrame } from '../fields/FieldFrame';
import { FieldGroup } from '../fields/FieldGroup';
import { FieldView } from '../fields/FieldView';
import { caseFields } from './case-fields';
import { caseGroups } from './case-groups';
import type { CaseFieldsProps } from './testcase.type';

const CaseFields = (props: CaseFieldsProps) => {
  const values = caseFields(props.test);
  return (
    <>
      {caseGroups().map((group) => (
        <FieldGroup
          key={group.id}
          group={group}
          render={(name) => (
            <FieldFrame name={name}>
              <FieldView name={name} value={values[name]} />
            </FieldFrame>
          )}
        />
      ))}
    </>
  );
};

export { CaseFields };
