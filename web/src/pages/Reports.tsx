import { useState } from 'react';
import { FileOutput, Download, Loader2, ChevronDown } from 'lucide-react';
import Card from '../components/Card';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import { useReports, useGenerateReport } from '../hooks/useApi';
import { api } from '../api/client';

function NoCaseMessage() {
  return (
    <div className="flex items-center justify-center h-64">
      <p className="text-[var(--text-muted)]">Select a case to manage reports.</p>
    </div>
  );
}

const REPORT_TYPES = [
  { value: 'full', label: 'Full Report', desc: 'Complete investigation report with all evidence and analysis' },
  { value: 'summary', label: 'Summary', desc: 'Executive summary with key findings' },
  { value: 'timeline', label: 'Timeline', desc: 'Chronological event timeline' },
] as const;

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function Reports() {
  const { caseId } = useCaseStore();
  const reportsQuery = useReports();
  const generateReport = useGenerateReport();
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [downloading, setDownloading] = useState<string | null>(null);

  if (!caseId) return <NoCaseMessage />;

  const reports: Array<Record<string, unknown>> = Array.isArray(reportsQuery.data) ? reportsQuery.data : [];

  const handleGenerate = (reportType: string) => {
    generateReport.mutate(reportType);
    setDropdownOpen(false);
  };

  const handleDownload = async (reportId: string, filename?: string) => {
    setDownloading(reportId);
    try {
      const response = await api.get(`/reports/${reportId}/download`, { responseType: 'blob' });
      const url = window.URL.createObjectURL(new Blob([response.data]));
      const link = document.createElement('a');
      link.href = url;
      link.download = filename || `report_${reportId}.pdf`;
      document.body.appendChild(link);
      link.click();
      link.remove();
      window.URL.revokeObjectURL(url);
    } catch {
      // Error toast handled by global error handler
    } finally {
      setDownloading(null);
    }
  };

  const statusColor = (status: string) => {
    if (status === 'completed') return 'success';
    if (status === 'generating') return 'info';
    if (status === 'error') return 'danger';
    return 'default';
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <FileOutput size={22} className="text-[var(--accent)]" />
          <h2 className="text-lg font-bold text-[var(--text-primary)]">Reports</h2>
          <span className="text-xs text-[var(--text-muted)]">{reports.length} reports</span>
        </div>

        {/* Generate dropdown */}
        <div className="relative">
          <button
            onClick={() => setDropdownOpen(!dropdownOpen)}
            disabled={generateReport.isPending}
            className="flex items-center gap-2 px-3 py-1.5 bg-[var(--accent)] text-white text-xs font-medium rounded-lg hover:bg-[var(--accent-hover)] transition-colors disabled:opacity-50"
          >
            {generateReport.isPending ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <FileOutput size={14} />
            )}
            Generate Report
            <ChevronDown size={12} />
          </button>
          {dropdownOpen && (
            <div className="absolute right-0 top-full mt-1 w-64 bg-[var(--bg-card)] border border-[var(--border)] rounded-lg shadow-lg z-50 overflow-hidden">
              {REPORT_TYPES.map(({ value, label, desc }) => (
                <button
                  key={value}
                  onClick={() => handleGenerate(value)}
                  className="w-full text-left px-4 py-3 hover:bg-[var(--bg-hover)] transition-colors border-b border-[var(--border)] last:border-0"
                >
                  <p className="text-sm font-medium text-[var(--text-primary)]">{label}</p>
                  <p className="text-[10px] text-[var(--text-muted)] mt-0.5">{desc}</p>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Reports table */}
      <Card>
        {reportsQuery.isLoading ? (
          <LoadingSpinner text="Loading reports..." />
        ) : reports.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-48 gap-3">
            <FileOutput size={32} className="text-[var(--text-muted)]" />
            <p className="text-sm text-[var(--text-muted)]">No reports generated yet.</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[var(--border)]">
                  <th className="text-left px-4 py-3 text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">Type</th>
                  <th className="text-left px-4 py-3 text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">Status</th>
                  <th className="text-left px-4 py-3 text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">Created</th>
                  <th className="text-left px-4 py-3 text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">Size</th>
                  <th className="text-right px-4 py-3 text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">Action</th>
                </tr>
              </thead>
              <tbody>
                {reports.map((report) => {
                  const id = String(report.id);
                  const status = String(report.status || 'unknown');
                  const reportType = String(report.report_type || 'full');
                  const createdAt = String(report.created_at || '');
                  const fileSize = report.file_size as number | undefined;
                  const filePath = String(report.file_path || '');
                  const filename = filePath.split('/').pop() || `report_${id}.pdf`;

                  return (
                    <tr key={id} className="border-b border-[var(--border)] last:border-0 hover:bg-[var(--bg-hover)] transition-colors">
                      <td className="px-4 py-3">
                        <Badge type={reportType} />
                      </td>
                      <td className="px-4 py-3">
                        <Badge type={statusColor(status)}>
                          {status === 'generating' && <Loader2 size={10} className="animate-spin mr-1" />}
                          {status}
                        </Badge>
                      </td>
                      <td className="px-4 py-3 text-xs text-[var(--text-secondary)] font-mono">
                        {createdAt.slice(0, 19).replace('T', ' ')}
                      </td>
                      <td className="px-4 py-3 text-xs text-[var(--text-secondary)] font-mono">
                        {fileSize ? formatBytes(fileSize) : '-'}
                      </td>
                      <td className="px-4 py-3 text-right">
                        {status === 'completed' && (
                          <button
                            onClick={() => handleDownload(id, filename)}
                            disabled={downloading === id}
                            className="inline-flex items-center gap-1.5 px-2.5 py-1 bg-[var(--accent-green)]/20 text-[var(--accent-green)] text-xs font-medium rounded hover:bg-[var(--accent-green)]/30 transition-colors disabled:opacity-50"
                          >
                            {downloading === id ? (
                              <Loader2 size={12} className="animate-spin" />
                            ) : (
                              <Download size={12} />
                            )}
                            Download
                          </button>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </Card>
    </div>
  );
}
