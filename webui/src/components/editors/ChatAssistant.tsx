import React, { useState, useRef, useEffect } from 'react';

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

interface ChatAssistantProps {
  context?: {
    type: 'unit' | 'motif' | 'structure';
    name: string;
    yaml?: string;
  };
}

export function ChatAssistant({ context }: ChatAssistantProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || loading) return;
    const userMessage = { role: 'user' as const, content: input };
    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setLoading(true);

    try {
      const systemPrompt = buildSystemPrompt(context);
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: 'claude-3-5-sonnet-20241022',
          messages: [
            { role: 'system', content: systemPrompt },
            ...messages.map((m) => ({ role: m.role, content: m.content })),
            { role: 'user', content: input },
          ],
          max_tokens: 1024,
          stream: false,
        }),
      });
      const data = await response.json();
      const assistantMessage = {
        role: 'assistant' as const,
        content: data.content || data.error || 'No response',
      };
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (e) {
      setMessages((prev) => [...prev, { role: 'assistant', content: `Error: ${e}` }]);
    }
    setLoading(false);
  };

  return (
    <div style={{
      background: '#0f0f1a',
      borderTop: '1px solid #3b3b5c',
      display: 'flex',
      flexDirection: 'column',
      height: 300,
      fontFamily: 'monospace',
    }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', padding: '8px 16px', borderBottom: '1px solid #3b3b5c' }}>
        <span style={{ color: '#7c3aed', fontSize: 14 }}>🤖</span>
        <span style={{ color: '#e2e8f0', marginLeft: 8, fontSize: 13 }}>Assistant</span>
      </div>

      {/* Messages */}
      <div style={{ flex: 1, overflowY: 'auto', padding: 16 }}>
        {messages.length === 0 && (
          <div style={{ color: '#64748b', fontSize: 12 }}>
            Ask me about editing this {context?.type || 'resource'}.
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} style={{ marginBottom: 12 }}>
            <div style={{ color: msg.role === 'user' ? '#7c3aed' : '#22c55e', fontSize: 11, marginBottom: 2 }}>
              {msg.role === 'user' ? 'You' : 'Assistant'}
            </div>
            <div style={{ color: '#e2e8f0', fontSize: 13, whiteSpace: 'pre-wrap' }}>{msg.content}</div>
          </div>
        ))}
        {loading && <div style={{ color: '#64748b', fontSize: 12 }}>Thinking...</div>}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <div style={{ display: 'flex', gap: 8, padding: 12, borderTop: '1px solid #3b3b5c' }}>
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSend()}
          placeholder={`Ask about ${context?.type || 'this'}...`}
          style={{
            flex: 1,
            background: '#1a1a2e',
            border: '1px solid #3b3b5c',
            borderRadius: 4,
            padding: '8px 12px',
            color: '#e2e8f0',
            fontFamily: 'monospace',
            fontSize: 13,
          }}
        />
        <button
          onClick={handleSend}
          disabled={loading}
          style={{
            background: '#7c3aed',
            color: '#fff',
            border: 'none',
            padding: '8px 16px',
            borderRadius: 4,
            cursor: loading ? 'not-allowed' : 'pointer',
            opacity: loading ? 0.5 : 1,
          }}
        >
          Send
        </button>
      </div>
    </div>
  );
}

function buildSystemPrompt(context?: ChatAssistantProps['context']): string {
  if (!context) return 'You are a helpful assistant for COGTOME.';
  return `You are a COGTOME expert helping edit a ${context.type} called "${context.name}".
${context.yaml ? `Current content:\n${context.yaml}` : ''}
COGTOME info:
- Units: atomic executables, stdin/stdout JSON
- Motifs: flow of Units, supports if/foreach control flow
- Structures: composed of Motifs and Units
- Variables: \${params.x}, \${steps.name.output.field}
- Expression functions: filter, map with == != > < && ||`;
}
