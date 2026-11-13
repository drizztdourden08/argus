/* @layer renderer-app @kind component */
import { Flex, InlineEdit, Text } from '@drizztdourden08/tessera';
import { findBoard, useBoards } from '../../../../data/boards';
import { useRename } from '../behavior/useRename';
import type { BoardTitleProps } from '../BoardHeader.type';

const BoardTitle = (props: BoardTitleProps) => {
  const { boardKey, name } = props;
  const info = findBoard(useBoards(), boardKey);
  const id = info?.id ?? boardKey;
  const rename = useRename(id);
  return (
    <Flex align="center" gap="sm">
      <InlineEdit
        label="Board name"
        value={info?.name ?? name}
        onCommit={rename.commit}
        validate={rename.validate}
        renderValue={(text) => <Text as="h1" className="board-header__title" title={id}>{text}</Text>}
      />
      {rename.error ? <Text variant="caption" tone="danger" role="alert">{rename.error}</Text> : null}
    </Flex>
  );
};

export { BoardTitle };
