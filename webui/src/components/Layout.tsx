import { Link, useLocation } from 'react-router-dom';

export function Layout({ children }: { children: React.ReactNode }) {
  const location = useLocation();

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <h1 style={styles.title}>COGTOME</h1>
        <nav style={styles.nav}>
          <Link
            to="/structures"
            style={{
              ...styles.navLink,
              ...(location.pathname === '/structures' ? styles.navLinkActive : {}),
            }}
          >
            Structures
          </Link>
          <Link
            to="/motifs"
            style={{
              ...styles.navLink,
              ...(location.pathname === '/motifs' ? styles.navLinkActive : {}),
            }}
          >
            Motifs
          </Link>
        </nav>
      </header>
      <main style={styles.main}>{children}</main>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    minHeight: '100vh',
    backgroundColor: '#1a1a2e',
    color: '#eee',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '1rem 2rem',
    backgroundColor: '#16213e',
    borderBottom: '1px solid #0f3460',
  },
  title: {
    margin: 0,
    fontSize: '1.5rem',
    fontWeight: 'bold',
    color: '#e94560',
  },
  nav: {
    display: 'flex',
    gap: '1rem',
  },
  navLink: {
    color: '#aaa',
    textDecoration: 'none',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    transition: 'all 0.2s',
  },
  navLinkActive: {
    backgroundColor: '#0f3460',
    color: '#fff',
  },
  main: {
    padding: '2rem',
  },
};
