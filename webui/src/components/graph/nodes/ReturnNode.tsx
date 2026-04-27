import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, getNodeHeaderStyle, getIconBadgeStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

interface ReturnNodeData {
  mappings?: Record<string, string>;
}

export const ReturnNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as ReturnNodeData;
  const { mappings = {} } = nodeData;
  const config = NODE_TYPE_CONFIGS.return;
  const color = config.color;

  return (
    <div style={getNodeShellStyle('return', !!selected, color)}>
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
        <span style={{ color: 'var(--node-text)', fontWeight: 600 }}>return</span>
      </div>

      {Object.entries(mappings).map(([key, val]) => (
        <div key={key} style={{ color: '#22c55e', fontSize: 11, marginBottom: 2 }}>
          {key}: <span style={{ color: '#7c3aed' }}>{String(val).slice(0, 20)}</span>
        </div>
      ))}

      {Object.keys(mappings).length === 0 && (
        <div style={{ color: 'var(--node-text-muted)', fontSize: 11, fontStyle: 'italic' }}>(no mappings)</div>
      )}
    </div>
  );
});

ReturnNode.displayName = 'ReturnNode';
