import { useState } from 'react';
import { FileText, Plus, Send, X } from 'lucide-react';
import Card from '../components/Card';
import DataTable from '../components/DataTable';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import { useEvidence, useSubmitEvidence } from '../hooks/useApi';

export default function Evidence() {
  const { caseId } = useCaseStore();
  const evidenceQuery = useEvidence();
  const submitEvidence = useSubmitEvidence();
  const [showForm, setShowForm] = useState(false);
  const [content, setContent] = useState('');
  const [source, setSource] = useState('');

  if (!caseId) {
    return <NoCaseMessage />;
  }

  const evidence = Array.isArray(evidenceQuery.data) ? evidenceQuery.data : [];

  const handleSubmit = () => {
    if (!content.trim()) return;
    submitEvidence.mutate(
      { content: content.trim(), source: source.trim() || undefined },
      {
        onSuccess: () => {
          setContent('');
          setSource('');
          setShowForm(false);
        },
      }
    );
  };

  const columns = [
    {
      key: 'type',
      label: 'Type',
      className: 'w-24',
      render: (row: Record<string, unknown>) => <Badge type={String(row.type || row.evidence_type || 'text')} />,
    },
    {
      key: 'content',
      label: 'Content',
      render: (row: Record<string, unknown>) => (
        <span className="text-[var(--text-primary)] line-clamp-2 text-xs">
          {String(row.content || row.text || row.summary || '').slice(0, 200)}
        </span>
      ),
    },
    {
      key: 'source',
      label: 'Source',
      className: 'w-32',
      render: (row: Record<string, unknown>) => (
        <span className="text-xs">{String(row.source || '-')}</span>
      ),
    },
    {
      key: 'created_at',
      label: 'Date',
      className: 'w-40',
      render: (row: Record<string, unknown>) => (
        <span className="text-xs text-[var(--text-muted)]">
          {row.created_at ? new Date(String(row.created_at)).toLocaleString() : '-'}
        </span>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Evidence</h2>
          <p className="text-sm text-[var(--text-muted)]">{evidence.length} items</p>
        </div>
        <button
          onClick={() => setShowForm(!showForm)}
          className="flex items-center gap-2 px-4 py-2 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white rounded-lg text-sm font-medium transition-colors"
        >
          {showForm ? <X size={16} /> : <Plus size={16} />}
          {showForm ? 'Cancel' : 'Add Evidence'}
        </button>
      </div>

      {showForm && (
        <Card className="border-[var(--accent)]/30">
          <div className="space-y-3">
            <textarea
              placeholder="Paste evidence text here..."
              value={content}
              onChange={e => setContent(e.target.value)}
              className="w-full h-32 px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] resize-none"
            />
            <div className="flex gap-3">
              <input
                type="text"
                placeholder="Source (optional)"
                value={source}
                onChange={e => setSource(e.target.value)}
                className="flex-1 px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)]"
              />
              <button
                onClick={handleSubmit}
                disabled={!content.trim() || submitEvidence.isPending}
                className="flex items-center gap-2 px-4 py-2 bg-[var(--accent-green)] hover:bg-[var(--accent-green)]/80 text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
              >
                <Send size={14} />
                {submitEvidence.isPending ? 'Submitting...' : 'Submit'}
              </button>
            </div>
          </div>
        </Card>
      )}

      <Card>
        {evidenceQuery.isLoading ? (
          <LoadingSpinner text="Loading evidence..." />
        ) : (
          <DataTable
            columns={columns}
            data={evidence as Record<string, unknown>[]}
            searchable
            searchKeys={['content', 'text', 'source', 'type']}
            emptyMessage="No evidence yet. Add text evidence to get started."
          />
        )}
      </Card>
    </div>
  );
}

function NoCaseMessage() {
  return (
    <div className="flex flex-col items-center justify-center h-full text-center">
      <FileText size={48} className="text-[var(--text-muted)] mb-4" />
      <p className="text-[var(--text-secondary)]">Select a case to view evidence.</p>
    </div>
  );
}
