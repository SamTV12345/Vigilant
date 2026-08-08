import { useState } from 'react'
import { useStatus, useDailyUptime, useIncidents } from '../api'
import StatusBadge from '../components/StatusBadge'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

const BANNER: Record<string, { bg: string; text: string; emoji: string }> = {
  healthy: { bg: 'bg-green-600', text: 'All Systems Operational', emoji: '✅' },
  sick: { bg: 'bg-yellow-600', text: 'Degraded Performance', emoji: '⚠️' },
  partial: { bg: 'bg-partial', text: 'Partial Outage', emoji: '🔶' },
  dead: { bg: 'bg-red-600', text: 'Major Outage', emoji: '🔴' },
}

// --- Sparkline + uptime% cell ---
function UptimeCell({ monitorId }: { monitorId: string }) {
  const { data: days = [] } = useDailyUptime(monitorId, 90)
  const [hovered, setHovered] = useState<number | null>(null)

  if (days.length === 0) {
    return (
      <div className="flex items-center gap-3">
        <div className="flex gap-px items-end h-8 opacity-30"><div className="text-xs text-muted-foreground">No data</div></div>
        <span className="text-muted-foreground text-xs w-14 text-right tabular-nums">—</span>
      </div>
    )
  }

  const totalHealthy = days.reduce((s, d) => s + d.healthy, 0)
  const totalChecks = days.reduce((s, d) => s + d.healthy + d.sick + d.dead, 0)
  const uptimePct = totalChecks > 0 ? (totalHealthy / totalChecks) * 100 : 100

  return (
    <div className="flex items-center gap-3">
      <div className="relative flex gap-px items-end h-8" onMouseLeave={() => setHovered(null)}>
        {days.map((d, i) => {
          const h = Math.max(2, (d.uptime_percent / 100) * 32)
          const color = d.uptime_percent >= 99 ? 'bg-green' : d.uptime_percent >= 95 ? 'bg-yellow' : 'bg-red'
          return (
            <div key={i} className="relative" onMouseEnter={() => setHovered(i)}>
              <div
                className={`${color} rounded-xs w-[3px] opacity-80 hover:opacity-100 transition-opacity`}
                style={{ height: `${h}px` }}
              />
              {d.dead > 0 && (
                <div className="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-red" />
              )}
              {hovered === i && (
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 z-50">
                  <div className="bg-foreground text-background text-xs rounded-lg px-3 py-2 shadow-lg whitespace-nowrap">
                    <div className="font-medium mb-0.5">{d.date}</div>
                    <div className={d.uptime_percent >= 99 ? 'text-green-400' : d.uptime_percent >= 95 ? 'text-yellow-400' : 'text-red-400'}>
                      {d.uptime_percent.toFixed(1)}% uptime
                    </div>
                    <div className="opacity-70 mt-0.5">
                      {d.healthy} healthy · {d.sick} degraded · {d.dead} outage
                    </div>
                    {d.dead > 0 && <div className="text-red-400 mt-0.5 text-[11px]">⚠ Outage this day</div>}
                  </div>
                  <div className="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-foreground" />
                </div>
              )}
            </div>
          )
        })}
      </div>
      <span className="text-muted-foreground text-xs w-14 text-right tabular-nums font-medium">
        {uptimePct.toFixed(2)}%
      </span>
    </div>
  )
}

// --- Subscribe modal ---
function SubscribeModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [email, setEmail] = useState('')
  const [status, setStatus] = useState<'idle' | 'loading' | 'ok' | 'error'>('idle')
  const [msg, setMsg] = useState('')

  async function handleSubscribe(e: React.FormEvent) {
    e.preventDefault()
    setStatus('loading')
    try {
      const r = await fetch('/api/subscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email }),
      })
      const data = await r.json()
      if (r.ok) {
        setStatus('ok')
        setMsg(data.message || 'Subscribed!')
        setEmail('')
      } else {
        setStatus('error')
        setMsg(data.error || 'Failed')
      }
    } catch {
      setStatus('error')
      setMsg('Network error')
    }
  }

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Subscribe to Updates</DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          {/* Email */}
          <div className="bg-background border border-border rounded-lg p-3">
            <span className="font-medium text-sm">📧 Email</span>
            <span className="text-muted-foreground block text-xs mt-0.5 mb-2">Get notified when incidents happen</span>
            <form onSubmit={handleSubscribe} className="flex gap-2">
              <Input
                type="email"
                required
                placeholder="you@example.com"
                value={email}
                onChange={e => setEmail(e.target.value)}
              />
              <Button type="submit" disabled={status === 'loading'} size="sm">
                {status === 'loading' ? '...' : 'Subscribe'}
              </Button>
            </form>
            {status === 'ok' && <p className="text-green text-xs mt-1.5">{msg}</p>}
            {status === 'error' && <p className="text-red text-xs mt-1.5">{msg}</p>}
          </div>

          {/* RSS */}
          <a href="/api/feed/atom" target="_blank" rel="noopener"
            className="block bg-background border border-border rounded-lg p-3 hover:border-primary text-foreground text-sm no-underline">
            <span className="font-medium">📡 Atom Feed</span>
            <span className="text-muted-foreground block text-xs mt-0.5">Subscribe via any feed reader</span>
          </a>

          {/* Other channels */}
          <div className="bg-background border border-border rounded-lg p-3 text-sm">
            <span className="font-medium">💬 Slack / Teams / Webhook</span>
            <span className="text-muted-foreground block text-xs mt-0.5">Configure in the admin panel under Notifications</span>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}

function timeAgo(ts: string): string {
  const diff = Date.now() - new Date(ts + 'Z').getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs}h ago`
  const days = Math.floor(hrs / 24)
  return `${days}d ago`
}

export default function PublicStatus() {
  const { data: status } = useStatus()
  const { data: incidents = [] } = useIncidents(20)
  const [subscribeOpen, setSubscribeOpen] = useState(false)

  const s = status?.status || 'healthy'
  const b = BANNER[s] || BANNER.healthy

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="max-w-3xl mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 rounded bg-primary flex items-center justify-center text-primary-foreground font-bold text-xs">V</div>
            <span className="font-bold text-foreground">Vigilant Status</span>
          </div>
          <Button onClick={() => setSubscribeOpen(true)}>
            Subscribe to Updates
          </Button>
        </div>
      </header>

      {/* Status Banner */}
      <div className={`${b.bg} text-white text-center py-12 px-4`}>
        <div className="text-4xl mb-2">{b.emoji}</div>
        <h1 className="text-2xl font-bold">{b.text}</h1>
      </div>

      {/* Services */}
      <div className="max-w-3xl mx-auto px-4 py-8">
        <h2 className="text-base font-semibold text-muted-foreground uppercase tracking-wide mb-4">Services</h2>

        {status?.monitors && status.monitors.length > 0 ? (
          <div className="flex flex-col gap-2">
            {status.monitors.map(m => (
              <div key={m.id} className="bg-card border border-border rounded-lg px-4 py-3">
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3">
                      <span className="font-medium text-foreground text-sm truncate">{m.name}</span>
                      <StatusBadge status={m.status} />
                    </div>
                    <div className="text-muted-foreground text-xs mt-1 truncate hidden sm:block">{m.url}</div>
                  </div>

                  <div className="flex items-center gap-4 ml-4">
                    <UptimeCell monitorId={m.id} />
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-muted-foreground text-sm">No services monitored.</p>
        )}
      </div>

      {/* Uptime Legend */}
      <div className="max-w-3xl mx-auto px-4 pb-8">
        <div className="flex flex-wrap gap-4 text-xs text-muted-foreground">
          <span className="flex items-center gap-1"><span className="w-3 h-3 rounded-xs bg-green inline-block" /> Operational</span>
          <span className="flex items-center gap-1"><span className="w-3 h-3 rounded-xs bg-yellow inline-block" /> Degraded</span>
          <span className="flex items-center gap-1"><span className="w-3 h-3 rounded-xs bg-red inline-block" /> Outage</span>
        </div>
      </div>

      {/* Incident History */}
      {incidents.length > 0 && (
        <div className="max-w-3xl mx-auto px-4 pb-8">
          <h2 className="text-base font-semibold text-muted-foreground uppercase tracking-wide mb-4">Past Incidents</h2>
          <div className="flex flex-col gap-1">
            {incidents.map(inc => {
              const monitor = status?.monitors?.find(m => m.id === inc.monitor_id)
              const isResolved = inc.resolved_at !== null
              return (
                <div key={inc.id} className="bg-card border border-border rounded-lg px-4 py-3">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="flex items-center gap-2">
                        <span className={`w-2 h-2 rounded-full ${isResolved ? 'bg-green' : 'bg-yellow'}`} />
                        <span className="text-sm font-medium text-foreground">
                          {monitor?.name || inc.monitor_id}
                        </span>
                      </div>
                      <div className="text-xs text-muted-foreground mt-1">
                        {inc.status} · {timeAgo(inc.started_at)}
                        {inc.resolved_at && ` · resolved ${timeAgo(inc.resolved_at)}`}
                      </div>
                    </div>
                    <StatusBadge status={isResolved ? 'healthy' : 'sick'} />
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Footer */}
      <footer className="border-t border-border mt-8">
        <div className="max-w-3xl mx-auto px-4 py-6 flex items-center justify-between text-xs text-muted-foreground">
          <Button variant="link" onClick={() => setSubscribeOpen(true)} className="h-auto p-0 text-xs">
            Subscribe to Updates
          </Button>
          <span>
            <a href="/docs" className="text-muted-foreground hover:text-foreground no-underline mr-4">Docs</a>
            Powered by{' '}
            <a href="https://github.com/SamTV12345/Vigilant" className="text-primary hover:underline">Vigilant</a>
          </span>
        </div>
      </footer>

      <SubscribeModal open={subscribeOpen} onClose={() => setSubscribeOpen(false)} />
    </div>
  )
}
