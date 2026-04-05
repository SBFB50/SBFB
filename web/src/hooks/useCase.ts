import { useQuery } from '@tanstack/react-query';
import { useCaseStore } from '../stores/caseStore';
import { getCase, getCaseStats } from '../api/client';

export function useActiveCase() {
  const { caseId, caseName } = useCaseStore();

  const caseQuery = useQuery({
    queryKey: ['case', caseId],
    queryFn: () => getCase(caseId!),
    enabled: !!caseId,
  });

  const statsQuery = useQuery({
    queryKey: ['caseStats', caseId],
    queryFn: () => getCaseStats(caseId!),
    enabled: !!caseId,
  });

  return {
    caseId,
    caseName,
    caseData: caseQuery.data,
    stats: statsQuery.data,
    isLoading: caseQuery.isLoading || statsQuery.isLoading,
  };
}
