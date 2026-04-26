import { HashRouter, Routes, Route } from 'react-router-dom';
import { Layout } from './components/Layout';
import { StructureList } from './components/StructureList';
import { StructureEditor } from './components/StructureEditor';
import { MotifList } from './components/MotifList';
import { MotifViewer } from './components/MotifViewer';

function App() {
  return (
    <HashRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<StructureList />} />
          <Route path="/structures" element={<StructureList />} />
          <Route path="/structures/:name" element={<StructureEditor />} />
          <Route path="/motifs" element={<MotifList />} />
          <Route path="/motifs/:name" element={<MotifViewer />} />
        </Routes>
      </Layout>
    </HashRouter>
  );
}

export default App;
