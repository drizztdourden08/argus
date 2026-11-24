/* @layer renderer-app @kind component */
import { List, ListItem, Small, Tag } from '@drizztdourden08/tessera';
import { FaultTip } from '../../RegisteredStrip';
import type { BoardFaultsProps } from '../BoardHeader.type';
import './BoardFaults.css';
import { isLeadRole } from '../../../../data/content';

const BoardFaults = (props: BoardFaultsProps) => (
  <List aria-label="Faults">
    {props.faults.map((row) => (
      <ListItem key={row.id} bare className="board-faults__row">
        <Tag variant="urgency" color="danger">{row.session ?? row.thread}</Tag>
        {row.fault ? <Small className="board-faults__text"><FaultTip fault={row.fault} main={isLeadRole(row.role)} /></Small> : null}
      </ListItem>
    ))}
  </List>
);

export { BoardFaults };
