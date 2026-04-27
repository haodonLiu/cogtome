import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, getNodeHeaderStyle, getIconBadgeStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

export const JoinNode = memo(({ selected }: NodeProps) => {
  const config = NODE_TYPE_CONFIGS.join;
  const color = config.color;

  return (
    <div style={getNodeShellStyle('join', !!selected, color)}>
      {config.hasTopBar && <div style={getTopBarStyle(color)} />}

      <NodeHandle
        type="target"
        position={Position.Left}
        id="branch1"
        color={color}
        size={8}
        style={{ top: '25%' }}
      />
      <NodeHandle
        type="target"
        position={Position.Left}
        id="branch2"
        color={color}
        size={8}
        style={{ top: '50%' }}
      />
      <NodeHandle
        type="target"
        position={Position.Left}
        id="branch3"
        color={color}
        size={8}
        style={{ top: '75%' }}
      />

      <div style={getNodeHeaderStyle()}>
        <div style={getIconBadgeStyle(color)}>
          {config.icon}
        </div>
        <span style={{ color: 'var(--node-text)', fontWeight: 600 }}>join</span>
      </div>

      <div style={{ color: 'var(--node-text-muted)', fontSize: 10 }}>
        sync branches
      </div>

      <NodeHandle
        type="source"
        position={Position.Right}
        color={color}
        size={10}
      />
    </div>
  );
});

JoinNode.displayName = 'JoinNode';
