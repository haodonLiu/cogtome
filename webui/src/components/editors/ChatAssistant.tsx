import { useState, useRef, useEffect } from 'react';
import { Button } from '../ui';

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
    <div className="assistant-panel">
      {/* Header */}
      <div className="assistant-header">
        <span className="assistant-icon">🤖</span>
        <span className="assistant-title">Assistant</span>
      </div>

      {/* Messages */}
      <div className="assistant-messages">
        {messages.length === 0 && (
          <div className="assistant-empty">
            Ask me about editing this {context?.type || 'resource'}.
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className="assistant-message">
            <div className={`assistant-role assistant-role--${msg.role}`}>
              {msg.role === 'user' ? 'You' : 'Assistant'}
            </div>
            <div className="assistant-content">{msg.content}</div>
          </div>
        ))}
        {loading && <div className="assistant-loading">Thinking...</div>}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <div className="assistant-input-area">
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSend()}
          placeholder={`Ask about ${context?.type || 'this'}...`}
          className="assistant-input"
        />
        <Button
          onClick={handleSend}
          disabled={loading}
          variant="primary"
        >
          Send
        </Button>
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
