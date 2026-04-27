import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, getNodeHeaderStyle, getIconBadgeStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

interface MotifNodeData {
  name?: string;
  expanded?: boolean;
}

export const MotifNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as MotifNodeData;
  const { name = '', expanded = false } = nodeData;
  const config = NODE_TYPE_CONFIGS.motif;
  const color = config.color;

  return (
    <div style={getNodeShellStyle('motif', !!selected, color)}>
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
        <span style={{ color: 'var(--node-text)', fontWeight: 600 }}>{name || 'motif'}</span>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span
          style={{
            color: expanded ? '#22c55e' : 'var(--node-text-muted)',
            fontSize: 11,
          }}
        >
          {expanded ? '▼ expanded' : '▶ collapsed'}
        </span>
      </div>

      <NodeHandle
        type="source"
        position={Position.Right}
        color="#22c55e"
        size={10}
      />
    </div>
  );
});

MotifNode.displayName = 'MotifNode';
