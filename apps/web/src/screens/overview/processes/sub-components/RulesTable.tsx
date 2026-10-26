/* @layer renderer-app @kind component */
import { Disclosure, Stack, Table, TableCell, TableRow, TableSection, Text } from '@drizztdourden08/tessera';
import { formatMemory, formatMinutes } from '../../format';
import { matchText } from '../model';
import { RULE_HEADINGS } from '../processes.constants';
import type { RulesTableProps } from '../processes.type';

const yesNo = (value: boolean | undefined): string => (value ? 'yes' : '-');

const RulesTable = (props: RulesTableProps) => {
  const { rules } = props;
  return (
    <Disclosure summary={`Rules${rules.length > 0 ? ` (${rules.length})` : ''}`}>
      <Stack gap="sm">
        {rules.length === 0
          ? <Text tone="muted">No rules read yet.</Text>
          : (
            <Table striped aria-label="Process rules">
              <TableSection head>
                <TableRow>{RULE_HEADINGS.map((heading) => <TableCell key={heading} header>{heading}</TableCell>)}</TableRow>
              </TableSection>
              <TableSection>
                {rules.map((rule) => (
                  <TableRow key={rule.name}>
                    <TableCell>{rule.name}</TableCell>
                    <TableCell>{matchText(rule.match)}</TableCell>
                    <TableCell align="right">{formatMemory(rule.maxMemMB)}</TableCell>
                    <TableCell align="right">{formatMemory(rule.maxGroupMemMB)}</TableCell>
                    <TableCell align="right">{formatMinutes(rule.maxAgeMin)}</TableCell>
                    <TableCell align="right">{formatMinutes(rule.idleMin)}</TableCell>
                    <TableCell>{yesNo(rule.orphan)}</TableCell>
                    <TableCell>{yesNo(rule.hung)}</TableCell>
                    <TableCell>{yesNo(rule.killable)}</TableCell>
                    <TableCell>{rule.note ?? ''}</TableCell>
                  </TableRow>
                ))}
              </TableSection>
            </Table>
          )}
        <Text variant="caption" tone="muted">
          The rules are read-only here. The Overlord changes them with coord proc rule set (coord skill, processes.md); the next sample applies the change.
        </Text>
      </Stack>
    </Disclosure>
  );
};

export { RulesTable };
