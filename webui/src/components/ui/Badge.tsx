import { ReactNode } from 'react';

type BadgeVariant = 'accent' | 'success' | 'warning' | 'danger' | 'secondary';
type BadgeSize = 'sm' | 'md';

interface BadgeProps {
  variant?: BadgeVariant;
  size?: BadgeSize;
  children: ReactNode;
  style?: React.CSSProperties;
  className?: string;
}

const variantStyles: Record<BadgeVariant, { bg: string; color: string }> = {
  accent: { bg: 'var(--accent-light)', color: 'var(--accent)' },
  success: { bg: 'var(--success-bg)', color: 'var(--success)' },
  warning: { bg: 'var(--warning-bg)', color: 'var(--warning)' },
  danger: { bg: 'var(--danger-bg)', color: 'var(--danger)' },
  secondary: { bg: 'var(--bg-active)', color: 'var(--text-secondary)' },
};

const sizeStyles: Record<BadgeSize, { padding: string; fontSize: string }> = {
  sm: { padding: '2px 6px', fontSize: '11px' },
  md: { padding: '4px 10px', fontSize: '13px' },
};

export function Badge({
  variant = 'accent',
  size = 'sm',
  children,
  style,
  className,
}: BadgeProps) {
  const v = variantStyles[variant];
  const s = sizeStyles[size];

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        backgroundColor: v.bg,
        color: v.color,
        padding: s.padding,
        fontSize: s.fontSize,
        fontWeight: 500,
        borderRadius: 'var(--radius-sm)',
        ...style,
      }}
      className={className}
    >
      {children}
    </span>
  );
}