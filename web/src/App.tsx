import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClientProvider } from '@tanstack/react-query';
import { queryClient } from './api/client';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Evidence from './pages/Evidence';
import Entities from './pages/Entities';
import Hypotheses from './pages/Hypotheses';
import Graph from './pages/Graph';
import Timeline from './pages/Timeline';
import Investigation from './pages/Investigation';
import Benchmark from './pages/Benchmark';
import Suspects from './pages/Suspects';
import Wiki from './pages/Wiki';
import Reports from './pages/Reports';
import ImageSearch from './pages/ImageSearch';
import GovernmentPage from './pages/GovernmentPage';

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/evidence" element={<Evidence />} />
            <Route path="/entities" element={<Entities />} />
            <Route path="/hypotheses" element={<Hypotheses />} />
            <Route path="/graph" element={<Graph />} />
            <Route path="/timeline" element={<Timeline />} />
            <Route path="/investigation" element={<Investigation />} />
            <Route path="/suspects" element={<Suspects />} />
            <Route path="/wiki" element={<Wiki />} />
            <Route path="/reports" element={<Reports />} />
            <Route path="/images" element={<ImageSearch />} />
            <Route path="/benchmark" element={<Benchmark />} />
            <Route path="/government" element={<GovernmentPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
