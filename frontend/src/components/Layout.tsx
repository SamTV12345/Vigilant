import { NavLink, Outlet, useNavigate } from 'react-router-dom'
import { clearToken } from '../api'
import { Button } from '@/components/ui/button'

const link = ({ isActive }: { isActive: boolean }) =>
  `block px-3 py-2 rounded text-sm transition-colors ${isActive ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-border'}`

export default function Layout() {
  const nav = useNavigate()
  return (
    <div className="flex h-screen bg-background">
      <aside className="w-56 flex flex-col gap-1 p-4 bg-card border-r border-border">
        <h2 className="text-lg font-bold mb-3">⚡ Vigilant</h2>
        <NavLink to="/" end className={link}>Dashboard</NavLink>
        <NavLink to="/monitors" className={link}>Monitors</NavLink>
        <NavLink to="/notifications" className={link}>Notifications</NavLink>
        <NavLink to="/settings" className={link}>Settings</NavLink>
        <a href="/docs" className="block px-3 py-2 rounded text-sm text-muted-foreground hover:text-foreground hover:bg-border no-underline">Docs ↗</a>
        <div className="flex-1" />
        <Button variant="ghost" onClick={() => { clearToken(); nav('/login') }} className="justify-start text-muted-foreground hover:text-foreground">
          Logout
        </Button>
      </aside>
      <main className="flex-1 overflow-auto p-6"><Outlet /></main>
    </div>
  )
}
