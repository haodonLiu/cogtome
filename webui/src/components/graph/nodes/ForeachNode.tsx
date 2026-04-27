import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, getNodeHeaderStyle, getIconBadgeStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

interface ForeachNodeData {
  over?: string;
  maxIterations?: number;
  expanded?: boolean;
  body?: any[];
}

export const ForeachNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as ForeachNodeData;
  const { over = '', maxIterations = 50, expanded = false } = nodeData;
  const config = NODE_TYPE_CONFIGS.foreach;
  const color = config.color;

  return (
    <div style={getNodeShellStyle('foreach', !!selected, color)}>
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
        <span style={{ color: 'var(--node-text)', fontWeight: 600 }}>foreach</span>
      </div>

      <div style={{ background: 'var(--node-bg-subtle)', borderRadius: 4, padding: 8, marginBottom: 4, marginTop: 4 }}>
        <div style={{ color: 'var(--node-text-muted)', fontSize: 10, marginBottom: 4 }}>over</div>
        <div style={{ color, fontSize: 12 }}>{over || '(none)'}</div>
      </div>

      <div style={{ color: 'var(--node-text-muted)', fontSize: 11 }}>
        max: {maxIterations}
      </div>

      {expanded && (
        <div style={{ color: 'var(--node-text-muted)', fontSize: 11, fontStyle: 'italic', marginTop: 4 }}>
          (body: {nodeData.body?.length || 0} nodes)
        </div>
      )}

      <NodeHandle
        type="source"
        position={Position.Right}
        color="#22c55e"
        size={10}
      />
    </div>
  );
});

ForeachNode.displayName = 'ForeachNode';
