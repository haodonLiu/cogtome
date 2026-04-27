import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, getNodeHeaderStyle, getIconBadgeStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

interface IfNodeData {
  condition?: string;
  expanded?: boolean;
  body?: any[];
}

export const IfNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = (data || {}) as IfNodeData;
  const { condition = '', expanded = false } = nodeData;
  const config = NODE_TYPE_CONFIGS.if;
  const color = config.color;

  return (
    <div style={getNodeShellStyle('if', !!selected, color)}>
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
        <span style={{ color: 'var(--node-text)', fontWeight: 600 }}>if</span>
      </div>

      <div style={{ background: 'var(--node-bg-subtle)', borderRadius: 4, padding: 8, marginBottom: 8, marginTop: 4 }}>
        <div style={{ color: 'var(--node-text-muted)', fontSize: 10, marginBottom: 4 }}>condition</div>
        <div style={{ color, fontSize: 12 }}>{condition || '(none)'}</div>
      </div>

      {expanded && (
        <div style={{ color: 'var(--node-text-muted)', fontSize: 11, fontStyle: 'italic' }}>
          (body: {nodeData.body?.length || 0} nodes)
        </div>
      )}

      <NodeHandle
        type="source"
        position={Position.Right}
        color="#22c55e"
        size={10}
        id="output"
      />
    </div>
  );
});

IfNode.displayName = 'IfNode';
