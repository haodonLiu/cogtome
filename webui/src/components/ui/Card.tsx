import { ReactNode, MouseEvent } from 'react';

interface CardProps {
  children: ReactNode;
  hoverable?: boolean;
  onClick?: () => void;
  padding?: 'none' | 'sm' | 'md' | 'lg';
  style?: React.CSSProperties;
  className?: string;
}

const paddingMap = {
  none: '0',
  sm: '12px',
  md: '16px',
  lg: '24px',
};

export function Card({
  children,
  hoverable,
  onClick,
  padding = 'md',
  style,
  className,
}: CardProps) {
  return (
    <div
      onClick={onClick}
      style={{
        backgroundColor: 'var(--bg-card)',
        border: '1px solid var(--border)',
        borderRadius: 'var(--radius-lg)',
        padding: paddingMap[padding],
        boxShadow: 'var(--shadow-sm)',
        cursor: onClick ? 'pointer' : 'auto',
        transition: 'border-color 0.15s, box-shadow 0.15s',
        ...style,
      }}
      className={className}
      onMouseEnter={(e: MouseEvent<HTMLDivElement>) => {
        if (hoverable) {
          e.currentTarget.style.borderColor = 'var(--accent)';
          e.currentTarget.style.boxShadow = '0 4px 12px rgba(0,0,0,0.1)';
        }
      }}
      onMouseLeave={(e: MouseEvent<HTMLDivElement>) => {
        if (hoverable) {
          e.currentTarget.style.borderColor = 'var(--border)';
          e.currentTarget.style.boxShadow = 'var(--shadow-sm)';
        }
      }}
    >
      {children}
    </div>
  );
}