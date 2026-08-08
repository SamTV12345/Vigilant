import { useNavigate } from 'react-router-dom'
import { useMonitors, useStatus } from '../api'
import StatusBadge from '../components/StatusBadge'
import { Card, CardContent } from '@/components/ui/card'

function Stat({ label, value, color }: { label: string; value: string | number; color?: string }) {
  return (
    <Card>
      <CardContent className="min-w-[100px]">
        <div className="text-xs text-muted-foreground">{label}</div>
        <div className={`text-xl font-bold ${color || ''}`}>{value}</div>
      </CardContent>
    </Card>
  )
}

export default function Dashboard() {
  const { data: monitors = [] } = useMonitors()
  const { data: status } = useStatus()
  const navigate = useNavigate()

  const counts = { dead: 0, sick: 0, partial: 0, healthy: 0 }
  for (const m of monitors) counts[m.current_status as keyof typeof counts]++

  return (
    <div>
      <h1 className="text-xl font-bold mb-4">Dashboard</h1>

      <div className="flex flex-wrap gap-2 mb-6">
        <Card>
          <CardContent className="min-w-[120px]">
            <div className="text-xs text-muted-foreground">Overall</div>
            <div className="text-lg mt-1"><StatusBadge status={status?.status || 'healthy'} /></div>
          </CardContent>
        </Card>
        <Stat label="Healthy" value={counts.healthy} color="text-green" />
        <Stat label="Sick" value={counts.sick} color="text-yellow" />
        <Stat label="Partial" value={counts.partial} color="text-partial" />
        <Stat label="Dead" value={counts.dead} color="text-red" />
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        {monitors.map(m => (
          <Card
            key={m.id}
            className="cursor-pointer hover:border-primary transition-colors"
            onClick={() => navigate(`/monitors/${m.id}`)}
          >
            <CardContent>
              <div className="flex items-center justify-between mb-2">
                <strong>{m.name}</strong>
                <StatusBadge status={m.current_status} />
              </div>
              <div className="text-sm text-muted-foreground truncate">{m.url}</div>
              <div className="text-xs text-muted-foreground mt-1">{m.type} · every {m.interval_secs}s · {m.active ? 'Active' : 'Paused'}</div>
            </CardContent>
          </Card>
        ))}
        {monitors.length === 0 && (
          <p className="text-muted-foreground col-span-full">No monitors yet. <a href="/monitors" className="text-primary underline">Add one</a>.</p>
        )}
      </div>
    </div>
  )
}
