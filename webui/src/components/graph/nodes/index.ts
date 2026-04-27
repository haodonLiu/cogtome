export { UnitNode } from './UnitNode';
export { StartNode } from './StartNode';
export { IfNode } from './IfNode';
export { MatchNode } from './MatchNode';
export { ForeachNode } from './ForeachNode';
export { ForkNode } from './ForkNode';
export { JoinNode } from './JoinNode';
export { ReturnNode } from './ReturnNode';
export { MotifNode } from './MotifNode';

import { UnitNode } from './UnitNode';
import { StartNode } from './StartNode';
import { IfNode } from './IfNode';
import { MatchNode } from './MatchNode';
import { ForeachNode } from './ForeachNode';
import { ForkNode } from './ForkNode';
import { JoinNode } from './JoinNode';
import { ReturnNode } from './ReturnNode';
import { MotifNode } from './MotifNode';

export const nodeTypes = {
  start: StartNode,
  unit: UnitNode,
  if: IfNode,
  match: MatchNode,
  foreach: ForeachNode,
  fork: ForkNode,
  join: JoinNode,
  return: ReturnNode,
  motif: MotifNode,
};