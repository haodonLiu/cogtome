import { TextareaHTMLAttributes, forwardRef } from 'react';

interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  error?: boolean;
  mono?: boolean;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ error, mono, style, className, ...props }, ref) => {
    return (
      <textarea
        ref={ref}
        style={{
          width: '100%',
          padding: '8px 12px',
          fontSize: '14px',
          fontFamily: mono ? 'var(--font-mono)' : 'var(--font-sans)',
          backgroundColor: 'var(--bg-page)',
          color: 'var(--text-primary)',
          border: `1px solid ${error ? 'var(--danger)' : 'var(--border)'}`,
          borderRadius: 'var(--radius-sm)',
          outline: 'none',
          resize: 'vertical',
          minHeight: '80px',
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
      />
    );
  }
);

Textarea.displayName = 'Textarea';
