/* @layer renderer-app @kind component */
import { Flex, FilterBar } from '@drizztdourden08/tessera';
import { BoardFocus } from '../../BoardFocus';
import type { ClauseFilterRowProps } from '../BoardTabsRow.type';

const ClauseFilterRow = (props: ClauseFilterRowProps) => {
  const { boardKey, search, placeholder, onSearch, bar } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <BoardFocus boardKey={boardKey} />
      <FilterBar
        search={search}
        onSearchChange={onSearch}
        searchPlaceholder={placeholder}
        searchLabel={placeholder}
        schema={bar.schema}
        clauses={bar.clauses}
        onChange={bar.onChange}
      />
    </Flex>
  );
};

export { ClauseFilterRow };
