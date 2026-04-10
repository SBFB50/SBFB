import { useState } from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Play, Loader2, CheckCircle, Beaker, Send } from 'lucide-react';
import { submitTestTask } from '../../api/compute';

const TEST_TASKS = [
  {
    id: 'sentiment',
    label: 'Analyse de sentiment',
    type: 'sentiment',
    prompt: 'Analysez le sentiment politique de cette declaration: "Le gouvernement a annonce une reforme majeure des retraites qui provoque des manifestations dans toute la France. Les syndicats denoncent un passage en force."',
    priority: 3,
    color: '#22c55e',
  },
  {
    id: 'contradiction',
    label: 'Detection de contradiction',
    type: 'contradiction_detection',
    prompt: 'Comparez ces deux positions du meme depute:\nPosition A (2024): "Je suis fermement oppose a toute augmentation d\'impots sur les classes moyennes."\nPosition B (2025): "Cette hausse de la CSG est necessaire et je la voterai sans hesitation."\nY a-t-il contradiction? Expliquez.',
    priority: 2,
    color: '#ef4444',
  },
  {
    id: 'summary',
    label: 'Resume de scrutin',
    type: 'deep_analysis',
    prompt: 'Resumez ce scrutin parlementaire fictif: Projet de loi sur la transition energetique — 342 pour, 215 contre, 20 abstentions. Les groupes Renaissance et MoDem ont vote pour, LFI et RN contre, LR divise. Quels sont les enjeux principaux et les lignes de fracture?',
    priority: 5,
    color: '#3b82f6',
  },
  {
    id: 'classification',
    label: 'Classification thematique',
    type: 'classification',
    prompt: 'Classifiez cette declaration dans une ou plusieurs categories (economie, social, ecologie, securite, education, sante, international, justice, democratie): "Nous devons investir massivement dans le nucleaire tout en developpant les energies renouvelables pour atteindre la neutralite carbone en 2050."',
    priority: 7,
    color: '#a855f7',
  },
  {
    id: 'biography',
    label: 'Generation de biographie',
    type: 'biography',
    prompt: 'Generez une biographie politique factuelle et synthetique (5 lignes max) pour un depute fictif: Jean-Pierre Durand, depute LREM de la 3eme circonscription du Rhone depuis 2017, ancien maire de Villeurbanne, membre de la commission des finances.',
    priority: 8,
    color: '#06b6d4',
  },
];

export function TestPanel({ onTaskSent }: { onTaskSent?: () => void }) {
  const [sending, setSending] = useState<string | null>(null);
  const [sent, setSent] = useState<Set<string>>(new Set());
  const [lastResult, setLastResult] = useState<string>('');

  const handleSend = async (task: typeof TEST_TASKS[0]) => {
    setSending(task.id);
    try {
      const result = await submitTestTask(task.type, task.prompt, task.priority);
      setSent(prev => new Set(prev).add(task.id));
      setLastResult(`Task ${result.task_id?.slice(0, 8)}... cree (model: ${result.model || 'auto'})`);
      onTaskSent?.();
    } catch (err: any) {
      setLastResult(`Erreur: ${err?.response?.data?.detail || err.message}`);
    } finally {
      setSending(null);
    }
  };

  const handleSendAll = async () => {
    setSending('all');
    for (const task of TEST_TASKS) {
      try {
        await submitTestTask(task.type, task.prompt, task.priority);
        setSent(prev => new Set(prev).add(task.id));
      } catch {
        // continue
      }
    }
    setLastResult(`${TEST_TASKS.length} taches envoyees — le SelfWorker va les traiter`);
    setSending(null);
    onTaskSent?.();
  };

  return (
    <Card>
      <CardHeader className="border-b border-[var(--border)] py-2 px-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm flex items-center gap-2">
            <Beaker size={16} className="text-orange-400" />
            Tester le pipeline GPU
          </CardTitle>
          <button
            onClick={handleSendAll}
            disabled={sending !== null}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white rounded text-xs font-medium transition-colors"
          >
            {sending === 'all' ? <Loader2 size={12} className="animate-spin" /> : <Play size={12} />}
            Envoyer les 5 taches
          </button>
        </div>
      </CardHeader>
      <CardContent className="p-3 space-y-2">
        {TEST_TASKS.map(task => (
          <div
            key={task.id}
            className="flex items-center justify-between bg-[var(--bg-primary)] border border-[var(--border)] rounded px-3 py-2"
          >
            <div className="flex items-center gap-2 min-w-0">
              <div className="w-2 h-2 rounded-full shrink-0" style={{ backgroundColor: task.color }} />
              <div className="min-w-0">
                <p className="text-xs font-medium text-[var(--text-primary)]">{task.label}</p>
                <p className="text-[10px] text-[var(--text-muted)] truncate max-w-[400px]">
                  {task.type} — prio {task.priority}
                </p>
              </div>
            </div>
            <button
              onClick={() => handleSend(task)}
              disabled={sending !== null}
              className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] font-medium transition-colors shrink-0 ${
                sent.has(task.id)
                  ? 'bg-emerald-600/20 text-emerald-400 border border-emerald-600/30'
                  : 'bg-[var(--bg-hover)] hover:bg-blue-600/20 text-[var(--text-secondary)] hover:text-blue-400'
              }`}
            >
              {sending === task.id ? (
                <Loader2 size={10} className="animate-spin" />
              ) : sent.has(task.id) ? (
                <CheckCircle size={10} />
              ) : (
                <Send size={10} />
              )}
              {sent.has(task.id) ? 'Envoye' : 'Envoyer'}
            </button>
          </div>
        ))}

        {lastResult && (
          <p className="text-[10px] text-[var(--text-muted)] bg-[var(--bg-hover)] rounded px-2 py-1.5 font-mono">
            {lastResult}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
