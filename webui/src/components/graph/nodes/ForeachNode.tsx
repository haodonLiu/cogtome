import { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

interface ForeachNodeData {
  over?: string;
  maxIterations?: number;
  expanded?: boolean;
  body?: any[];
}

export const ForeachNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as ForeachNodeData;
  const { over = '', maxIterations = 50, expanded = false } = nodeData;

  return (
    <div
      style={{
        background: '#1a1a2e',
        border: selected ? '2px solid #06b6d4' : '2px solid #3b3b5c',
        borderRadius: 8,
        padding: 12,
        minWidth: 180,
        fontFamily: 'monospace',
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: '#06b6d4', width: 10, height: 10, border: 'none' }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#06b6d4',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#000',
          }}
        >
          #
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>foreach</span>
      </div>

      <div style={{ background: '#0f0f1a', borderRadius: 4, padding: 8, marginBottom: 4 }}>
        <div style={{ color: '#64748b', fontSize: 10, marginBottom: 4 }}>over</div>
        <div style={{ color: '#67e8f9', fontSize: 12 }}>{over || '(none)'}</div>
      </div>

      <div style={{ color: '#64748b', fontSize: 11 }}>
        max: {maxIterations}
      </div>

      {expanded && (
        <div style={{ color: '#64748b', fontSize: 11, fontStyle: 'italic', marginTop: 4 }}>
          (body: {nodeData.body?.length || 0} nodes)
        </div>
      )}

      <Handle
        type="source"
        position={Position.Right}
        style={{ background: '#22c55e', width: 10, height: 10, border: 'none' }}
      />
    </div>
  );
});

ForeachNode.displayName = 'ForeachNode';
