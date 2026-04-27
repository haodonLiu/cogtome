interface SpinnerProps {
  size?: number;
  style?: React.CSSProperties;
  className?: string;
}

export function Spinner({ size = 18, style, className }: SpinnerProps) {
  return (
    <div
      style={{
        width: size,
        height: size,
        border: '2px solid var(--border)',
        borderTopColor: 'var(--accent)',
        borderRadius: '50%',
        animation: 'spin 0.8s linear infinite',
        ...style,
      }}
      className={className}
    />
  );
}