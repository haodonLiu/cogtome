export { UnitNode } from './UnitNode';
export { IfNode } from './IfNode';
export { ForeachNode } from './ForeachNode';
export { ReturnNode } from './ReturnNode';
export { MotifNode } from './MotifNode';

import { UnitNode } from './UnitNode';
import { IfNode } from './IfNode';
import { ForeachNode } from './ForeachNode';
import { ReturnNode } from './ReturnNode';
import { MotifNode } from './MotifNode';

export const nodeTypes = {
  unit: UnitNode,
  if: IfNode,
  foreach: ForeachNode,
  return: ReturnNode,
  motif: MotifNode,
};
