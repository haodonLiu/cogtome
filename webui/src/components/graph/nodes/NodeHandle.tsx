import { memo } from 'react';
import { Handle, Position, HandleProps } from '@xyflow/react';

export interface NodeHandleProps extends HandleProps {
  color?: string;
  size?: number;
}

export const NodeHandle = memo(({
  type = 'source',
  position = Position.Right,
  color = 'var(--node-border)',
  size = 10,
  id,
  style,
  ...rest
}: NodeHandleProps) => {
  return (
    <Handle
      type={type}
      position={position}
      id={id}
      style={{
        background: color,
        width: size,
        height: size,
        border: 'none',
        transition: 'transform 0.15s ease, background 0.15s ease',
        ...style,
      }}
      className="node-handle"
      {...rest}
    />
  );
});

NodeHandle.displayName = 'NodeHandle';
