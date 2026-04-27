import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, getNodeHeaderStyle, getIconBadgeStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

interface UnitNodeData {
  name?: string;
  inputs?: Record<string, string>;
  outputs?: Array<{ id: string; name: string; type: string }>;
}

export const UnitNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as UnitNodeData;
  const { name, inputs = {}, outputs = [] } = nodeData;
  const config = NODE_TYPE_CONFIGS.unit;
  const color = config.color;

  return (
    <div style={getNodeShellStyle('unit', !!selected, color)}>
      {config.hasTopBar && <div style={getTopBarStyle(color)} />}

      {Object.keys(inputs).length > 0 && (
        <NodeHandle
          type="target"
          position={Position.Left}
          color={color}
          size={10}
        />
      )}

      <div style={getNodeHeaderStyle()}>
        <div style={getIconBadgeStyle(color)}>
          {config.icon}
        </div>
        <span style={{ color: 'var(--node-text)', fontWeight: 600 }}>{name || 'unit'}</span>
      </div>

      <div style={{ marginBottom: 4 }}>
        {Object.entries(inputs).map(([key, val]) => (
          <div key={key} style={{ color: 'var(--node-text-muted)', fontSize: 11, marginBottom: 2 }}>
            {key}: <span style={{ color: '#7c3aed' }}>{String(val).slice(0, 20)}</span>
          </div>
        ))}
      </div>

      {outputs.length > 0 && (
        <NodeHandle
          type="source"
          position={Position.Right}
          color="#22c55e"
          size={10}
        />
      )}
    </div>
  );
});

UnitNode.displayName = 'UnitNode';
