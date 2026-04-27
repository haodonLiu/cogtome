import React from 'react';

export type NodeTypeKey = 'start' | 'unit' | 'motif' | 'if' | 'match' | 'foreach' | 'fork' | 'join' | 'return';

export interface NodeTypeConfig {
  color: string;
  label: string;
  icon: React.ReactNode;
  hasTopBar?: boolean;
}

export const NODE_TYPE_CONFIGS: Record<NodeTypeKey, NodeTypeConfig> = {
  start: {
    color: '#22c55e',
    label: 'Start',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <polygon points="4,2 14,8 4,14" fill="currentColor" />
      </svg>
    ),
  },
  unit: {
    color: '#7c3aed',
    label: 'Unit',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <rect x="2" y="2" width="5" height="5" rx="1" fill="currentColor" />
        <rect x="9" y="2" width="5" height="5" rx="1" fill="currentColor" opacity="0.6" />
        <rect x="2" y="9" width="5" height="5" rx="1" fill="currentColor" opacity="0.6" />
        <rect x="9" y="9" width="5" height="5" rx="1" fill="currentColor" />
      </svg>
    ),
  },
  motif: {
    color: '#a855f7',
    label: 'Motif',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <path d="M8 1L14 5V11L8 15L2 11V5L8 1Z" fill="currentColor" opacity="0.3" />
        <path d="M8 1L14 5L8 9L2 5L8 1Z" fill="currentColor" opacity="0.6" />
        <path d="M8 9L14 5V11L8 15V9Z" fill="currentColor" />
      </svg>
    ),
  },
  if: {
    color: '#f59e0b',
    label: 'If',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <rect x="3" y="1" width="10" height="14" rx="2" fill="currentColor" opacity="0.2" />
        <rect x="3" y="1" width="10" height="14" rx="2" stroke="currentColor" strokeWidth="1.5" />
        <text x="8" y="11" textAnchor="middle" fontSize="7" fontWeight="bold" fill="currentColor">1</text>
      </svg>
    ),
  },
  match: {
    color: '#ec4899',
    label: 'Match',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <polygon points="8,1 15,4.5 15,11.5 8,15 1,11.5 1,4.5" fill="currentColor" opacity="0.3" />
        <polygon points="8,1 15,4.5 8,8 1,4.5" fill="currentColor" opacity="0.7" />
        <polygon points="8,8 15,4.5 15,11.5 8,15" fill="currentColor" />
      </svg>
    ),
  },
  foreach: {
    color: '#06b6d4',
    label: 'Foreach',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <path
          d="M8 2C5.24 2 3 4.24 3 7C3 9.76 5.24 12 8 12C10.76 12 13 9.76 13 7"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          fill="none"
        />
        <polygon points="13,5 13,9 9,7" fill="currentColor" />
        <path
          d="M5 7L3 5M5 7L3 9"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
  },
  fork: {
    color: '#8b5cf6',
    label: 'Fork',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="4" r="2" fill="currentColor" />
        <circle cx="4" cy="13" r="2" fill="currentColor" opacity="0.5" />
        <circle cx="12" cy="13" r="2" fill="currentColor" opacity="0.5" />
        <path d="M8 6L8 9M8 9L4 11M8 9L12 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    ),
  },
  join: {
    color: '#8b5cf6',
    label: 'Join',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <circle cx="4" cy="4" r="2" fill="currentColor" opacity="0.5" />
        <circle cx="12" cy="4" r="2" fill="currentColor" opacity="0.5" />
        <circle cx="8" cy="13" r="2" fill="currentColor" />
        <path d="M4 6L8 9L12 6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M8 9L8 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    ),
  },
  return: {
    color: '#22c55e',
    label: 'Return',
    hasTopBar: true,
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <path
          d="M6 4L2 8L6 12"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <path
          d="M2 8H10C11.66 8 13 6.66 13 5V4"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
  },
};

export function getNodeShellStyle(_type: NodeTypeKey, selected: boolean, color: string) {
  const borderColor = selected ? color : 'var(--node-border)';
  const borderWidth = selected ? 2 : 1;
  const border = `${borderWidth}px solid ${borderColor}`;

  return {
    background: 'var(--node-bg)',
    border,
    borderRadius: 'var(--node-radius)',
    padding: '12px',
    minWidth: 140,
    fontFamily: 'var(--font-mono)',
    position: 'relative' as const,
    overflow: 'hidden',
    boxShadow: selected ? `0 0 0 1px ${color}40, var(--node-shadow)` : 'var(--node-shadow)',
  };
}

export function getTopBarStyle(color: string) {
  return {
    position: 'absolute' as const,
    top: 0,
    left: 0,
    right: 0,
    height: '3px',
    background: color,
    borderRadius: '8px 8px 0 0',
  };
}

export function getNodeHeaderStyle() {
  return {
    display: 'flex',
    alignItems: 'center' as const,
    gap: 8,
    marginBottom: 4,
  };
}

export function getIconBadgeStyle(color: string) {
  return {
    width: 20,
    height: 20,
    borderRadius: 4,
    background: color,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: '#fff',
    flexShrink: 0,
  };
}
