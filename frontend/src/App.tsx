import { Routes, Route, Navigate } from 'react-router-dom'
import { TooltipProvider } from '@/components/ui/tooltip'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import Monitors from './pages/Monitors'
import MonitorDetail from './pages/MonitorDetail'
import Notifications from './pages/Notifications'
import Settings from './pages/Settings'
import ChangePassword from './pages/ChangePassword'
import Login from './pages/Login'
import PublicStatus from './pages/PublicStatus'
import Docs from './pages/Docs'
import { useAuth } from './api'

function Protected({ children }: { children: React.ReactNode }) {
  return useAuth() ? <>{children}</> : <Navigate to="/login" />
}

export default function App() {
  return (
    <TooltipProvider>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/change-password" element={<Protected><ChangePassword /></Protected>} />
        <Route path="/status" element={<PublicStatus />} />
        <Route path="/docs" element={<Docs />} />
        <Route path="/" element={<Protected><Layout /></Protected>}>
          <Route index element={<Dashboard />} />
          <Route path="monitors" element={<Monitors />} />
          <Route path="monitors/:id" element={<MonitorDetail />} />
          <Route path="notifications" element={<Notifications />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </TooltipProvider>
  )
}
