# UI Component Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create unified ui/ component library and migrate all usage, eliminating CSS/inline style duplication.

**Architecture:** 11 React components with consistent API (variant/size/disabled props), TypeScript types, CSS variable theming. Migration via component replacement with visual verification.

**Tech Stack:** React 18, TypeScript, CSS Variables, @xyflow/react (unchanged)

---

## File Structure

```
src/components/ui/
├── index.ts          # Unified exports
├── Button.tsx       # Already exists
├── Input.tsx        # NEW
├── Textarea.tsx     # NEW
├── Select.tsx       # NEW
├── Badge.tsx        # NEW
├── Card.tsx         # NEW
├── Modal.tsx        # NEW
├── Spinner.tsx      # NEW
├── EmptyState.tsx   # NEW
├── ErrorBanner.tsx  # NEW
└── FormField.tsx    # NEW
```

---

## Phase 1: Core Components

### Task 1: Create Input Component

**Files:**
- Create: `webui/src/components/ui/Input.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create Input.tsx**

```tsx
import { InputHTMLAttributes, forwardRef } from 'react';

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
  mono?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ error, mono, style, className, ...props }, ref) => {
    return (
      <input
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

Input.displayName = 'Input';
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { Input } from './Input';
```

- [ ] **Step 3: Verify build**

Run: `cd /home/haodont/cogtome/webui && npm run build 2>&1 | head -20`
Expected: No errors related to Input

---

### Task 2: Create Textarea Component

**Files:**
- Create: `webui/src/components/ui/Textarea.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create Textarea.tsx**

```tsx
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
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { Textarea } from './Textarea';
```

- [ ] **Step 3: Verify build**

Run: `cd /home/haodont/cogtome/webui && npm run build 2>&1 | head -20`
Expected: No errors

---

### Task 3: Create Badge Component

**Files:**
- Create: `webui/src/components/ui/Badge.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create Badge.tsx**

```tsx
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
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { Badge } from './Badge';
```

- [ ] **Step 3: Verify build**

---

### Task 4: Create Spinner Component

**Files:**
- Create: `webui/src/components/ui/Spinner.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create Spinner.tsx**

```tsx
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
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { Spinner } from './Spinner';
```

- [ ] **Step 3: Verify build**

---

### Task 5: Create Card Component

**Files:**
- Create: `webui/src/components/ui/Card.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create Card.tsx**

```tsx
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
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { Card } from './Card';
```

- [ ] **Step 3: Verify build**

---

### Task 6: Create EmptyState Component

**Files:**
- Create: `webui/src/components/ui/EmptyState.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create EmptyState.tsx**

```tsx
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
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { EmptyState } from './EmptyState';
```

- [ ] **Step 3: Verify build**

---

### Task 7: Create ErrorBanner Component

**Files:**
- Create: `webui/src/components/ui/ErrorBanner.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create ErrorBanner.tsx**

```tsx
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
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { ErrorBanner } from './ErrorBanner';
```

- [ ] **Step 3: Verify build**

---

### Task 8: Create FormField Component

**Files:**
- Create: `webui/src/components/ui/FormField.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create FormField.tsx**

```tsx
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
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { FormField } from './FormField';
```

- [ ] **Step 3: Verify build**

---

### Task 9: Create Select Component

**Files:**
- Create: `webui/src/components/ui/Select.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create Select.tsx**

```tsx
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
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { Select } from './Select';
```

- [ ] **Step 3: Verify build**

---

### Task 10: Create Modal Component

**Files:**
- Create: `webui/src/components/ui/Modal.tsx`
- Modify: `webui/src/components/ui/index.ts`

- [ ] **Step 1: Create Modal.tsx**

```tsx
import { ReactNode, useEffect } from 'react';
import { createPortal } from 'react-dom';

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  footer?: ReactNode;
  children: ReactNode;
  width?: string;
  style?: React.CSSProperties;
  className?: string;
}

export function Modal({
  open,
  onClose,
  title,
  footer,
  children,
  width = '480px',
  style,
  className,
}: ModalProps) {
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && open) onClose();
    };
    window.addEventListener('keydown', handleEsc);
    return () => window.removeEventListener('keydown', handleEsc);
  }, [open, onClose]);

  if (!open) return null;

  return createPortal(
    <div
      onClick={onClose}
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(0,0,0,0.5)',
        backdropFilter: 'blur(4px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      className={className}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          backgroundColor: 'var(--bg-card)',
          borderRadius: 'var(--radius-lg)',
          width,
          maxWidth: '90vw',
          maxHeight: '90vh',
          display: 'flex',
          flexDirection: 'column',
          boxShadow: '0 20px 40px rgba(0,0,0,0.2)',
          ...style,
        }}
      >
        {title && (
          <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '16px 20px',
            borderBottom: '1px solid var(--border)',
          }}>
            <h3 style={{
              margin: 0,
              fontSize: '18px',
              fontWeight: 600,
              color: 'var(--text-primary)',
            }}>
              {title}
            </h3>
            <button
              onClick={onClose}
              style={{
                background: 'none',
                border: 'none',
                fontSize: '24px',
                color: 'var(--text-tertiary)',
                cursor: 'pointer',
                padding: '0 4px',
                lineHeight: 1,
              }}
            >
              ×
            </button>
          </div>
        )}
        <div style={{
          flex: 1,
          overflow: 'auto',
          padding: '20px',
        }}>
          {children}
        </div>
        {footer && (
          <div style={{
            padding: '16px 20px',
            borderTop: '1px solid var(--border)',
            display: 'flex',
            justifyContent: 'flex-end',
            gap: 12,
          }}>
            {footer}
          </div>
        )}
      </div>
    </div>,
    document.body
  );
}
```

- [ ] **Step 2: Export from index.ts**

Add to `webui/src/components/ui/index.ts`:
```ts
export { Modal } from './Modal';
```

- [ ] **Step 3: Verify build**

---

## Phase 2: List Page Migration

### Task 11: Migrate StructureList

**Files:**
- Modify: `webui/src/components/StructureList.tsx`

- [ ] **Step 1: Read StructureList.tsx to identify inline styles to replace**

- [ ] **Step 2: Add imports**

```tsx
import { Card, Badge, Spinner, EmptyState, ErrorBanner, Button } from './ui';
```

- [ ] **Step 3: Replace inline styles.card with Card component**

Replace:
```tsx
// OLD
<div style={styles.card}>
```
With:
```tsx
<Card hoverable style={{ /* extracted card styles */ }}>
```

- [ ] **Step 4: Replace spinner with Spinner component**

Replace inline spinner div with `<Spinner />`

- [ ] **Step 5: Replace error banner with ErrorBanner component**

- [ ] **Step 6: Replace empty state with EmptyState component**

- [ ] **Step 7: Replace buttons with Button component**

- [ ] **Step 8: Verify build and visual**

---

### Task 12: Migrate MotifList

**Files:**
- Modify: `webui/src/components/MotifList.tsx`

(Same pattern as StructureList)

- [ ] **Step 1: Read MotifList.tsx**
- [ ] **Step 2: Add imports**
- [ ] **Step 3-8: Apply same replacements as StructureList**

---

### Task 13: Migrate UnitList

**Files:**
- Modify: `webui/src/components/UnitList.tsx`

(Same pattern)

- [ ] **Step 1: Read UnitList.tsx**
- [ ] **Step 2: Add imports**
- [ ] **Step 3-8: Apply same replacements**

---

## Phase 3: Editor Migration

### Task 14: Migrate PropertyPanel

**Files:**
- Modify: `webui/src/components/editors/PropertyPanel.tsx`

- [ ] **Step 1: Replace .panel-empty with EmptyState**
- [ ] **Step 2: Replace Delete buttons with Button variant="danger"**
- [ ] **Step 3: Verify build**

---

### Task 15: Migrate UnitEditor

**Files:**
- Modify: `webui/src/components/editors/UnitEditor.tsx`

- [ ] **Step 1: Replace template modal buttons with Button component**
- [ ] **Step 2: Replace editor toolbar buttons with Button component**
- [ ] **Step 3: Replace test button with Button**
- [ ] **Step 4: Verify build**

---

### Task 16: Migrate MotifEditor

**Files:**
- Modify: `webui/src/components/editors/MotifEditor.tsx`

- [ ] **Step 1: Replace toolbar buttons with Button component**
- [ ] **Step 2: Verify build**

---

### Task 17: Migrate StructureEditor

**Files:**
- Modify: `webui/src/components/editors/StructureEditor.tsx`

- [ ] **Step 1: Replace toolbar buttons with Button component**
- [ ] **Step 2: Verify build**

---

## Phase 4: CSS Cleanup

### Task 18: Remove Duplicate CSS Classes

**Files:**
- Modify: `webui/src/index.css`

- [ ] **Step 1: Remove .btn-primary, .btn-secondary, .btn-ghost, .btn-danger**
- [ ] **Step 2: Remove .input, .input.textarea (keep if still needed)**
- [ ] **Step 3: Remove .card duplicate styles**
- [ ] **Step 4: Verify build with no visual regressions**

---

## Verification

- [ ] Run `npm run build` - must pass
- [ ] Run `npm run dev` - test list pages, editors, modals
- [ ] Verify orange accent color (#ff6b00) on all buttons
- [ ] Verify font-weight 600 on buttons

---

## Self-Review Checklist

- [ ] All 11 components created and exported
- [ ] All list pages migrated (StructureList, MotifList, UnitList)
- [ ] All editors migrated (PropertyPanel, UnitEditor, MotifEditor, StructureEditor)
- [ ] CSS cleanup completed
- [ ] Build passes
- [ ] No TypeScript errors
