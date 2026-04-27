interface ErrorBannerProps {
  message: string;
  onDismiss?: () => void;
  style?: React.CSSProperties;
  className?: string;
}

export function ErrorBanner({
  message,
  onDismiss,
  style,
  className,
}: ErrorBannerProps) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        backgroundColor: 'var(--danger-bg)',
        color: 'var(--danger)',
        padding: '12px 16px',
        borderRadius: 'var(--radius-md)',
        fontSize: '14px',
        border: '1px solid #fecaca',
        ...style,
      }}
      className={className}
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <span style={{ flex: 1 }}>{message}</span>
      {onDismiss && (
        <button
          onClick={onDismiss}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--danger)',
            cursor: 'pointer',
            padding: '4px',
            fontSize: '18px',
            lineHeight: 1,
          }}
        >
          ×
        </button>
      )}
    </div>
  );
}
