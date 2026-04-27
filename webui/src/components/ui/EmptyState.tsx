import { ReactNode } from 'react';

interface EmptyStateProps {
  title: string;
  description?: string;
  icon?: ReactNode;
  action?: ReactNode;
  style?: React.CSSProperties;
  className?: string;
}

export function EmptyState({
  title,
  description,
  icon,
  action,
  style,
  className,
}: EmptyStateProps) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '64px 24px',
        textAlign: 'center',
        ...style,
      }}
      className={className}
    >
      {icon && (
        <div style={{ marginBottom: 16, color: 'var(--text-tertiary)' }}>
          {icon}
        </div>
      )}
      <h3 style={{
        margin: '0 0 8px 0',
        fontSize: '16px',
        fontWeight: 600,
        color: 'var(--text-secondary)',
      }}>
        {title}
      </h3>
      {description && (
        <p style={{
          margin: 0,
          fontSize: '14px',
          color: 'var(--text-tertiary)',
        }}>
          {description}
        </p>
      )}
      {action && (
        <div style={{ marginTop: 16 }}>
          {action}
        </div>
      )}
    </div>
  );
}
