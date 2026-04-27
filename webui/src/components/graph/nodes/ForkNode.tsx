import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, getNodeHeaderStyle, getIconBadgeStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

export const ForkNode = memo(({ selected }: NodeProps) => {
  const config = NODE_TYPE_CONFIGS.fork;
  const color = config.color;
  const shellStyle = getNodeShellStyle('fork', !!selected, color);

  return (
    <div
      style={{
        ...shellStyle,
        borderTop: `3px solid ${color}`,
      }}
    >
      {config.hasTopBar && <div style={getTopBarStyle(color)} />}

      <NodeHandle
        type="target"
        position={Position.Left}
        color={color}
        size={10}
      />

      <div style={getNodeHeaderStyle()}>
        <div style={getIconBadgeStyle(color)}>
          {config.icon}
        </div>
        <span style={{ color: 'var(--node-text)', fontWeight: 600 }}>fork</span>
      </div>

      <div style={{ color: 'var(--node-text-muted)', fontSize: 10 }}>
        parallel branches
      </div>

      <NodeHandle
        type="source"
        position={Position.Right}
        id="branch1"
        color={color}
        size={8}
        style={{ top: '25%' }}
      />
      <NodeHandle
        type="source"
        position={Position.Right}
        id="branch2"
        color={color}
        size={8}
        style={{ top: '50%' }}
      />
      <NodeHandle
        type="source"
        position={Position.Right}
        id="branch3"
        color={color}
        size={8}
        style={{ top: '75%' }}
      />
    </div>
  );
});

ForkNode.displayName = 'ForkNode';
