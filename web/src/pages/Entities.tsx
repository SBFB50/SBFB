import { useMemo, useState } from 'react';
import { Users } from 'lucide-react';
import Card from '../components/Card';
import DataTable from '../components/DataTable';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import { useEntities } from '../hooks/useApi';

const entityTypes = ['all', 'person', 'location', 'organization', 'event', 'vehicle', 'weapon', 'phone', 'email', 'document'];

export default function Entities() {
  const { caseId } = useCaseStore();
  const entitiesQuery = useEntities();
  const [selectedType, setSelectedType] = useState('all');

  if (!caseId) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <Users size={48} className="text-[var(--text-muted)] mb-4" />
        <p className="text-[var(--text-secondary)]">Select a case to view entities.</p>
      </div>
    );
  }

  const allEntities = Array.isArray(entitiesQuery.data) ? entitiesQuery.data : [];

  const filtered = useMemo(() => {
    if (selectedType === 'all') return allEntities;
    return allEntities.filter((e: Record<string, unknown>) =>
      String(e.type || e.entity_type || '').toLowerCase() === selectedType
    );
  }, [allEntities, selectedType]);

  const typeCounts = useMemo(() => {
    const counts: Record<string, number> = { all: allEntities.length };
    allEntities.forEach((e: Record<string, unknown>) => {
      const t = String(e.type || e.entity_type || 'unknown').toLowerCase();
      counts[t] = (counts[t] || 0) + 1;
    });
    return counts;
  }, [allEntities]);

  const columns = [
    {
      key: 'type',
      label: 'Type',
      className: 'w-28',
      render: (row: Record<string, unknown>) => (
        <Badge type={String(row.type || row.entity_type || 'unknown')} />
      ),
    },
    {
      key: 'name',
      label: 'Name',
      render: (row: Record<string, unknown>) => (
        <span className="text-[var(--text-primary)] font-medium">
          {String(row.name || row.value || row.label || '-')}
        </span>
      ),
    },
    {
      key: 'description',
      label: 'Description',
      render: (row: Record<string, unknown>) => (
        <span className="text-xs text-[var(--text-secondary)] line-clamp-1">
          {String(row.description || row.details || row.context || '-').slice(0, 120)}
        </span>
      ),
    },
    {
      key: 'confidence',
      label: 'Confidence',
      className: 'w-24',
      render: (row: Record<string, unknown>) => {
        const conf = Number(row.confidence ?? row.score ?? 0);
        const pct = conf <= 1 ? conf * 100 : conf;
        return (
          <span className="text-xs font-mono" style={{
            color: pct >= 70 ? 'var(--accent-green)' : pct >= 40 ? 'var(--accent-yellow)' : 'var(--accent-red)'
          }}>
            {Math.round(pct)}%
          </span>
        );
      },
    },
    {
      key: 'mentions',
      label: 'Mentions',
      className: 'w-20',
      render: (row: Record<string, unknown>) => (
        <span className="text-xs text-[var(--text-muted)]">{String(row.mentions ?? row.count ?? '-')}</span>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-[var(--text-primary)]">Entities</h2>
        <p className="text-sm text-[var(--text-muted)]">{allEntities.length} extracted entities</p>
      </div>

      {/* Type filter tabs */}
      <div className="flex flex-wrap gap-2">
        {entityTypes.map(type => (
          <button
            key={type}
            onClick={() => setSelectedType(type)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              selectedType === type
                ? 'bg-[var(--accent)] text-white'
                : 'bg-[var(--bg-card)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]'
            }`}
          >
            {type.charAt(0).toUpperCase() + type.slice(1)}
            {typeCounts[type] !== undefined && (
              <span className="ml-1.5 opacity-70">({typeCounts[type]})</span>
            )}
          </button>
        ))}
      </div>

      <Card>
        {entitiesQuery.isLoading ? (
          <LoadingSpinner text="Loading entities..." />
        ) : (
          <DataTable
            columns={columns}
            data={filtered as Record<string, unknown>[]}
            searchable
            searchKeys={['name', 'value', 'label', 'description', 'type']}
            emptyMessage="No entities found for this filter."
          />
        )}
      </Card>
    </div>
  );
}
