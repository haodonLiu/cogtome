import { HashRouter, Routes, Route } from 'react-router-dom';
import { Layout } from './components/Layout';
import { StructureList } from './components/StructureList';
import { StructureEditor } from './components/StructureEditor';
import { MotifList } from './components/MotifList';
import { MotifViewer } from './components/MotifViewer';
import { UnitEditor } from './components/editors/UnitEditor';

function App() {
  return (
    <HashRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
      <Layout>
        <Routes>
          <Route path="/" element={<StructureList />} />
          <Route path="/structures" element={<StructureList />} />
          <Route path="/structures/new" element={<StructureEditor />} />
          <Route path="/structures/:name" element={<StructureEditor />} />
          <Route path="/motifs" element={<MotifList />} />
          <Route path="/motifs/:name" element={<MotifViewer />} />
          <Route path="/units/:name" element={<UnitEditor />} />
        </Routes>
      </Layout>
    </HashRouter>
  );
}

export default App;
