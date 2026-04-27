import { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const StartNode = memo(({ selected }: NodeProps) => {

  return (
    <div
      style={{
        background: '#1e1e2e',
        border: selected ? '2px solid #22c55e' : '2px solid #22c55e80',
        borderRadius: 20,
        padding: 12,
        minWidth: 80,
        fontFamily: 'monospace',
        fontSize: 12,
        textAlign: 'center',
      }}
    >
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: '50%',
          background: '#22c55e',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 12,
          fontWeight: 700,
          color: '#000',
          margin: '0 auto 4px',
        }}
      >
        ▶
      </div>
      <span style={{ color: '#22c55e', fontWeight: 600 }}>Start</span>

      {/* Output handle */}
      <Handle
        type="source"
        position={Position.Right}
        style={{
          background: '#22c55e',
          width: 10,
          height: 10,
          border: 'none',
        }}
      />
    </div>
  );
});

StartNode.displayName = 'StartNode';