import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, getNodeHeaderStyle, getIconBadgeStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

interface MatchNodeData {
  condition?: string;
}

export const MatchNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = (data || {}) as MatchNodeData;
  const { condition = '' } = nodeData;
  const config = NODE_TYPE_CONFIGS.match;
  const color = config.color;

  const shellStyle = getNodeShellStyle('match', !!selected, color);

  return (
    <div
      style={{
        ...shellStyle,
        padding: '16px 20px',
        clipPath: 'polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%)',
      }}
    >
      {config.hasTopBar && <div style={getTopBarStyle(color)} />}

      <NodeHandle
        type="target"
        position={Position.Left}
        color={color}
        size={10}
        id="input"
      />

      <div style={getNodeHeaderStyle()}>
        <div style={getIconBadgeStyle(color)}>
          {config.icon}
        </div>
        <span style={{ color: 'var(--node-text)', fontWeight: 600 }}>match</span>
      </div>

      <div style={{ color: 'var(--node-text-muted)', fontSize: 11, marginTop: 4 }}>
        on: <span style={{ color }}>{condition || '...'}</span>
      </div>

      <NodeHandle
        type="source"
        position={Position.Right}
        id="case1"
        color={color}
        size={8}
        style={{ top: '30%' }}
      />
      <NodeHandle
        type="source"
        position={Position.Right}
        id="case2"
        color={color}
        size={8}
        style={{ top: '70%' }}
      />
    </div>
  );
});

MatchNode.displayName = 'MatchNode';
