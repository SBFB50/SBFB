import { useState } from 'react';
import { Image, Search, Database, Loader2, Eye } from 'lucide-react';
import Card from '../components/Card';
import LoadingSpinner from '../components/LoadingSpinner';
import ScoreBar from '../components/ScoreBar';
import { useCaseStore } from '../stores/caseStore';
import { useSearchImagesByText, useIndexCaseImages } from '../hooks/useApi';
import * as clientApi from '../api/client';

function NoCaseMessage() {
  return (
    <div className="flex items-center justify-center h-64">
      <p className="text-[var(--text-muted)]">Select a case to search images.</p>
    </div>
  );
}

interface SearchResult {
  evidence_id: string;
  path: string;
  case_id: string;
  description: string;
  distance?: number;
  similarity?: number;
}

export default function ImageSearch() {
  const { caseId } = useCaseStore();
  const searchMutation = useSearchImagesByText();
  const indexMutation = useIndexCaseImages();
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [similarResults, setSimilarResults] = useState<SearchResult[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [loadingSimilar, setLoadingSimilar] = useState(false);

  if (!caseId) return <NoCaseMessage />;

  const handleSearch = () => {
    if (!query.trim()) return;
    searchMutation.mutate(
      { query: query.trim(), nResults: 12 },
      { onSuccess: (data) => { setResults(data); setSimilarResults([]); setSelectedId(null); } }
    );
  };

  const handleFindSimilar = async (evidenceId: string) => {
    setSelectedId(evidenceId);
    setLoadingSimilar(true);
    try {
      const data = await clientApi.getSimilarImages(caseId!, evidenceId, 6);
      setSimilarResults(data);
    } catch {
      setSimilarResults([]);
    } finally {
      setLoadingSimilar(false);
    }
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Image size={22} className="text-[var(--accent)]" />
          <h2 className="text-lg font-bold text-[var(--text-primary)]">Image Search</h2>
        </div>
        <button
          onClick={() => indexMutation.mutate()}
          disabled={indexMutation.isPending}
          className="flex items-center gap-2 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] text-xs font-medium rounded-lg hover:border-[var(--accent)] transition-colors disabled:opacity-50"
        >
          {indexMutation.isPending ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <Database size={14} />
          )}
          {indexMutation.isPending ? 'Indexing...' : 'Index Images'}
        </button>
      </div>

      {/* Index result message */}
      {indexMutation.isSuccess && indexMutation.data && (
        <div className="bg-[var(--accent-green)]/10 border border-[var(--accent-green)]/30 rounded-lg px-4 py-2 text-xs text-[var(--accent-green)]">
          Indexed {(indexMutation.data as Record<string, number>).indexed} / {(indexMutation.data as Record<string, number>).total} images
        </div>
      )}

      {/* Search bar */}
      <Card>
        <div className="flex gap-2 p-4">
          <div className="relative flex-1">
            <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              placeholder="Describe the image you're looking for... (CLIP text-to-image)"
              className="w-full pl-10 pr-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
            />
          </div>
          <button
            onClick={handleSearch}
            disabled={searchMutation.isPending || !query.trim()}
            className="flex items-center gap-2 px-4 py-2.5 bg-[var(--accent)] text-white text-sm font-medium rounded-lg hover:bg-[var(--accent-hover)] transition-colors disabled:opacity-50"
          >
            {searchMutation.isPending ? (
              <Loader2 size={16} className="animate-spin" />
            ) : (
              <Search size={16} />
            )}
            Search
          </button>
        </div>
      </Card>

      {/* Results grid */}
      {results.length > 0 && (
        <div>
          <p className="text-xs text-[var(--text-muted)] mb-3">{results.length} results for "{query}"</p>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
            {results.map((result) => {
              const similarity = result.similarity ?? (result.distance != null ? Math.max(0, 1 - result.distance) * 100 : 0);
              const isSelected = result.evidence_id === selectedId;

              return (
                <div
                  key={result.evidence_id}
                  className={`bg-[var(--bg-card)] border rounded-lg overflow-hidden transition-all ${
                    isSelected ? 'border-[var(--accent)] ring-1 ring-[var(--accent)]' : 'border-[var(--border)] hover:border-[var(--accent-hover)]'
                  }`}
                >
                  {/* Image placeholder — actual images would need a file serving endpoint */}
                  <div className="aspect-video bg-[var(--bg-primary)] flex items-center justify-center">
                    <Image size={32} className="text-[var(--text-muted)]" />
                  </div>

                  <div className="p-3">
                    <p className="text-xs text-[var(--text-primary)] font-medium truncate mb-1">
                      {result.description || result.evidence_id.slice(0, 12)}
                    </p>
                    <div className="mb-2">
                      <ScoreBar value={similarity} max={100} label="Similarity" />
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-[9px] text-[var(--text-muted)] font-mono">
                        {result.evidence_id.slice(0, 8)}
                      </span>
                      <button
                        onClick={() => handleFindSimilar(result.evidence_id)}
                        className="flex items-center gap-1 text-[10px] text-[var(--accent)] hover:text-[var(--accent-hover)] transition-colors"
                      >
                        <Eye size={10} />
                        Similar
                      </button>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Similar images panel */}
      {selectedId && (
        <div>
          <p className="text-xs text-[var(--text-muted)] mb-3">
            Visually similar to {selectedId.slice(0, 8)} (DINOv2)
          </p>
          {loadingSimilar ? (
            <LoadingSpinner text="Finding similar images..." />
          ) : similarResults.length === 0 ? (
            <p className="text-xs text-[var(--text-muted)]">No similar images found.</p>
          ) : (
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
              {similarResults.map((result) => {
                const similarity = result.similarity ?? (result.distance != null ? Math.max(0, 1 - result.distance) * 100 : 0);
                return (
                  <div
                    key={result.evidence_id}
                    className="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg overflow-hidden"
                  >
                    <div className="aspect-square bg-[var(--bg-primary)] flex items-center justify-center">
                      <Image size={24} className="text-[var(--text-muted)]" />
                    </div>
                    <div className="p-2">
                      <p className="text-[10px] text-[var(--text-primary)] truncate">{result.description || result.evidence_id.slice(0, 12)}</p>
                      <p className="text-[9px] text-[var(--text-muted)] font-mono mt-0.5">{similarity.toFixed(0)}% match</p>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      {/* Empty state */}
      {results.length === 0 && !searchMutation.isPending && (
        <div className="flex flex-col items-center justify-center h-48 gap-3">
          <Image size={40} className="text-[var(--text-muted)]" />
          <p className="text-sm text-[var(--text-muted)]">
            Search for images by describing them in natural language.
          </p>
          <p className="text-xs text-[var(--text-muted)]">
            Make sure to index images first using the "Index Images" button.
          </p>
        </div>
      )}
    </div>
  );
}
