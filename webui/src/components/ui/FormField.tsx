import { ReactNode } from 'react';

interface FormFieldProps {
  label?: string;
  error?: string;
  hint?: string;
  children: ReactNode;
  style?: React.CSSProperties;
  className?: string;
}

export function FormField({
  label,
  error,
  hint,
  children,
  style,
  className,
}: FormFieldProps) {
  return (
    <div style={{ marginBottom: 16, ...style }} className={className}>
      {label && (
        <label style={{
          display: 'block',
          marginBottom: 6,
          fontSize: '13px',
          fontWeight: 500,
          color: 'var(--text-secondary)',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
        }}>
          {label}
        </label>
      )}
      {children}
      {error && (
        <span style={{
          display: 'block',
          marginTop: 4,
          fontSize: '12px',
          color: 'var(--danger)',
        }}>
          {error}
        </span>
      )}
      {hint && !error && (
        <span style={{
          display: 'block',
          marginTop: 4,
          fontSize: '12px',
          color: 'var(--text-tertiary)',
        }}>
          {hint}
        </span>
      )}
    </div>
  );
}