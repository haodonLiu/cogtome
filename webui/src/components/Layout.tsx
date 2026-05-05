import React from 'react';
import { Link, useLocation } from 'react-router-dom';

const navItems = [
  { to: '/structures', match: '/structures', label: 'Structures', icon: <><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></> },
  { to: '/motifs', match: '/motifs', label: 'Motifs', icon: <><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></> },
  { to: '/units', match: '/units', label: 'Units', icon: <><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="2" x2="9" y2="4"/><line x1="15" y1="2" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="22"/><line x1="15" y1="20" x2="15" y2="22"/><line x1="20" y1="9" x2="22" y2="9"/><line x1="20" y1="14" x2="22" y2="14"/><line x1="2" y1="9" x2="4" y2="9"/><line x1="2" y1="14" x2="4" y2="14"/></> },
  { to: '/traces', match: '/traces', label: 'Traces', icon: <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/> },
];

export function Layout({ children }: { children: React.ReactNode }) {
  const location = useLocation();

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="app-brand">
          <svg width="26" height="26" viewBox="0 0 28 28" fill="none">
            <rect width="28" height="28" rx="7" fill="#4f46e5"/>
            <path d="M8 14h12M14 8v12" stroke="white" strokeWidth="2.5" strokeLinecap="round"/>
          </svg>
          <h1 className="app-brand-title">COGTOME</h1>
        </div>
        <nav className="app-nav">
          {navItems.map(item => {
            const isActive = item.to === '/structures'
              ? (location.pathname === '/' || location.pathname.startsWith('/structures'))
              : location.pathname.startsWith(item.match);
            return (
              <Link
                key={item.to}
                to={item.to}
                className={`app-nav-link${isActive ? ' active' : ''}`}
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  {item.icon}
                </svg>
                <span className="app-nav-label">{item.label}</span>
              </Link>
            );
          })}
        </nav>
      </header>
      <main className="app-main">
        <div className="app-content">{children}</div>
      </main>
    </div>
  );
}
