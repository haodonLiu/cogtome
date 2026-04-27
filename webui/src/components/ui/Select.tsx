import { SelectHTMLAttributes, forwardRef } from 'react';

interface SelectOption {
  value: string;
  label: string;
}

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  options: SelectOption[];
  error?: boolean;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ options, error, style, className, ...props }, ref) => {
    return (
      <select
        ref={ref}
        style={{
          width: '100%',
          padding: '8px 12px',
          fontSize: '14px',
          fontFamily: 'var(--font-sans)',
          backgroundColor: 'var(--bg-page)',
          color: 'var(--text-primary)',
          border: `1px solid ${error ? 'var(--danger)' : 'var(--border)'}`,
          borderRadius: 'var(--radius-sm)',
          outline: 'none',
          cursor: 'pointer',
          transition: 'border-color 0.15s',
          boxSizing: 'border-box',
          ...style,
        }}
        className={className}
        onFocus={(e) => {
          e.target.style.borderColor = 'var(--accent)';
        }}
        onBlur={(e) => {
          e.target.style.borderColor = error ? 'var(--danger)' : 'var(--border)';
        }}
        {...props}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    );
  }
);

Select.displayName = 'Select';