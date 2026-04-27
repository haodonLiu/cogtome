import { memo } from 'react';
import { Position, NodeProps } from '@xyflow/react';
import { NodeHandle } from './NodeHandle';
import { getNodeShellStyle, getTopBarStyle, NODE_TYPE_CONFIGS } from './nodeStyles';

export const StartNode = memo(({ selected }: NodeProps) => {
  const config = NODE_TYPE_CONFIGS.start;
  const color = config.color;

  return (
    <div style={getNodeShellStyle('start', !!selected, color)}>
      {config.hasTopBar && <div style={getTopBarStyle(color)} />}

      <div
        style={{
          width: 32,
          height: 32,
          borderRadius: '50%',
          background: color,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          margin: '0 auto 6px',
          color: '#000',
        }}
      >
        {config.icon}
      </div>
      <span style={{ color: 'var(--node-text)', fontWeight: 600, fontSize: 13, display: 'block', textAlign: 'center' }}>
        {config.label}
      </span>

      <NodeHandle
        type="source"
        position={Position.Right}
        color={color}
        size={10}
      />
    </div>
  );
});

StartNode.displayName = 'StartNode';
