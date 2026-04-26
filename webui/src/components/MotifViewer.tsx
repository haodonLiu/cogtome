import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { getMotif } from '../api/client';

export function MotifViewer() {
  const { name } = useParams<{ name: string }>();
  const [content, setContent] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!name) return;
    getMotif(name)
      .then(setContent)
      .catch((e) => setError((e as Error).message));
  }, [name]);

  if (error) {
    return (
      <div style={styles.container}>
        <p style={styles.error}>Error: {error}</p>
      </div>
    );
  }

  if (!content) {
    return (
      <div style={styles.container}>
        <p>Loading...</p>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <h2 style={styles.title}>{name}</h2>
      <pre style={styles.code}>{content}</pre>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    maxWidth: '900px',
    margin: '0 auto',
  },
  title: {
    marginBottom: '1rem',
    fontSize: '1.5rem',
  },
  code: {
    backgroundColor: '#16213e',
    border: '1px solid #0f3460',
    borderRadius: '8px',
    padding: '1.5rem',
    overflow: 'auto',
    fontFamily: 'monospace',
    fontSize: '0.875rem',
    lineHeight: 1.5,
    color: '#a0a0a0',
    maxHeight: '70vh',
  },
  error: {
    color: '#e94560',
  },
};
