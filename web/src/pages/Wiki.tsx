import { useState } from 'react';
import { BookOpen, RefreshCw, FileText, ChevronRight } from 'lucide-react';
import Card from '../components/Card';
import LoadingSpinner from '../components/LoadingSpinner';
import Badge from '../components/Badge';
import { useCaseStore } from '../stores/caseStore';
import { useWikiPages, useWikiPage, useRebuildWiki } from '../hooks/useApi';

function NoCaseMessage() {
  return (
    <div className="flex items-center justify-center h-64">
      <p className="text-[var(--text-muted)]">Select a case to view the wiki.</p>
    </div>
  );
}

/** Minimal markdown-to-JSX renderer for wiki content. */
function MarkdownContent({ content }: { content: string }) {
  const lines = content.split('\n');
  const elements: JSX.Element[] = [];
  let inList = false;
  let listItems: string[] = [];

  const flushList = () => {
    if (listItems.length > 0) {
      elements.push(
        <ul key={`list-${elements.length}`} className="list-disc list-inside space-y-1 mb-3 text-sm text-[var(--text-secondary)]">
          {listItems.map((item, i) => <li key={i}>{renderInline(item)}</li>)}
        </ul>
      );
      listItems = [];
      inList = false;
    }
  };

  const renderInline = (text: string) => {
    // Bold
    const parts = text.split(/\*\*(.+?)\*\*/g);
    return parts.map((part, i) =>
      i % 2 === 1
        ? <strong key={i} className="text-[var(--text-primary)] font-semibold">{part}</strong>
        : <span key={i}>{part}</span>
    );
  };

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    if (trimmed.startsWith('### ')) {
      flushList();
      elements.push(
        <h3 key={i} className="text-base font-semibold text-[var(--text-primary)] mt-4 mb-2">
          {trimmed.slice(4)}
        </h3>
      );
    } else if (trimmed.startsWith('## ')) {
      flushList();
      elements.push(
        <h2 key={i} className="text-lg font-bold text-[var(--text-primary)] mt-5 mb-2 border-b border-[var(--border)] pb-1">
          {trimmed.slice(3)}
        </h2>
      );
    } else if (trimmed.startsWith('# ')) {
      flushList();
      elements.push(
        <h1 key={i} className="text-xl font-bold text-[var(--text-primary)] mt-6 mb-3">
          {trimmed.slice(2)}
        </h1>
      );
    } else if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
      inList = true;
      listItems.push(trimmed.slice(2));
    } else if (trimmed === '') {
      flushList();
    } else {
      flushList();
      elements.push(
        <p key={i} className="text-sm text-[var(--text-secondary)] mb-2 leading-relaxed">
          {renderInline(trimmed)}
        </p>
      );
    }
  }
  flushList();

  return <div className="prose-dark">{elements}</div>;
}

export default function Wiki() {
  const { caseId } = useCaseStore();
  const pagesQuery = useWikiPages();
  const rebuildWiki = useRebuildWiki();
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const pageQuery = useWikiPage(selectedPath);

  if (!caseId) return <NoCaseMessage />;

  const pagesData = pagesQuery.data;
  const pages: Array<Record<string, unknown>> = Array.isArray(pagesData)
    ? pagesData
    : (pagesData as Record<string, unknown>)?.items as Array<Record<string, unknown>> ?? [];

  // Group by page_type
  const groups: Record<string, Array<Record<string, unknown>>> = {};
  for (const page of pages) {
    const type = String(page.page_type || 'other');
    if (!groups[type]) groups[type] = [];
    groups[type].push(page);
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <BookOpen size={22} className="text-[var(--accent)]" />
          <h2 className="text-lg font-bold text-[var(--text-primary)]">Case Wiki</h2>
          <span className="text-xs text-[var(--text-muted)]">{pages.length} pages</span>
        </div>
        <button
          onClick={() => rebuildWiki.mutate()}
          disabled={rebuildWiki.isPending}
          className="flex items-center gap-2 px-3 py-1.5 bg-[var(--accent)] text-white text-xs font-medium rounded-lg hover:bg-[var(--accent-hover)] transition-colors disabled:opacity-50"
        >
          <RefreshCw size={14} className={rebuildWiki.isPending ? 'animate-spin' : ''} />
          {rebuildWiki.isPending ? 'Rebuilding...' : 'Rebuild Wiki'}
        </button>
      </div>

      <div className="grid grid-cols-12 gap-4">
        {/* Left panel: page list */}
        <div className="col-span-4 xl:col-span-3">
          <Card>
            {pagesQuery.isLoading ? (
              <LoadingSpinner text="Loading pages..." />
            ) : pages.length === 0 ? (
              <p className="text-sm text-[var(--text-muted)] p-4">
                No wiki pages yet. Start an investigation to generate wiki content.
              </p>
            ) : (
              <div className="space-y-3">
                {Object.entries(groups).map(([type, typePages]) => (
                  <div key={type}>
                    <p className="text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider px-3 mb-1">
                      {type}
                    </p>
                    {typePages.map((page) => {
                      const path = String(page.page_path || page.path || '');
                      const title = String(page.title || path.replace(/\.md$/, '').replace(/_/g, ' '));
                      const isSelected = path === selectedPath;
                      return (
                        <button
                          key={path}
                          onClick={() => setSelectedPath(path)}
                          className={`w-full flex items-center gap-2 px-3 py-2 text-left text-xs rounded-lg transition-colors ${
                            isSelected
                              ? 'bg-[var(--accent)]/10 text-[var(--accent)]'
                              : 'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]'
                          }`}
                        >
                          <FileText size={14} className="shrink-0" />
                          <span className="truncate flex-1">{title}</span>
                          {isSelected && <ChevronRight size={12} />}
                        </button>
                      );
                    })}
                  </div>
                ))}
              </div>
            )}
          </Card>
        </div>

        {/* Right panel: page content */}
        <div className="col-span-8 xl:col-span-9">
          <Card>
            {!selectedPath ? (
              <div className="flex items-center justify-center h-64">
                <p className="text-sm text-[var(--text-muted)]">Select a page to view its content.</p>
              </div>
            ) : pageQuery.isLoading ? (
              <LoadingSpinner text="Loading page..." />
            ) : pageQuery.data ? (
              <div className="p-4">
                <MarkdownContent content={String(pageQuery.data.content || '')} />
              </div>
            ) : (
              <div className="flex items-center justify-center h-64">
                <p className="text-sm text-[var(--accent-red)]">Failed to load page.</p>
              </div>
            )}
          </Card>
        </div>
      </div>
    </div>
  );
}
