import { useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMonitor, useChecks, useUptime } from '../api'
import StatusBadge from '../components/StatusBadge'
import UptimeChart from '../components/UptimeChart'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

export default function MonitorDetail() {
  const { id } = useParams<{ id: string }>()
  const { data: monitor } = useMonitor(id!)
  const { data: checks = [] } = useChecks(id!, 100)
  const [period, setPeriod] = useState('24')
  const { data: uptime } = useUptime(id!, +period)

  if (!monitor) return <p className="text-muted-foreground">Loading...</p>

  const pct = uptime?.uptime_percent ?? 100
  const pctColor = pct >= 99 ? 'text-green' : pct >= 95 ? 'text-yellow' : 'text-red'

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div>
          <h1 className="text-xl font-bold">{monitor.name}</h1>
          <p className="text-muted-foreground text-sm">{monitor.url}</p>
        </div>
        <StatusBadge status={monitor.current_status} />
      </div>

      {uptime && (
        <Card className="mb-4">
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle>Uptime</CardTitle>
              <Select value={period} onValueChange={val => val && setPeriod(val)}>
                <SelectTrigger className="w-[140px]"><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="24">Last 24h</SelectItem>
                  <SelectItem value="168">Last 7 days</SelectItem>
                  <SelectItem value="720">Last 30 days</SelectItem>
                  <SelectItem value="2160">Last 90 days</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardHeader>
          <CardContent>
            <div className={`text-3xl font-bold ${pctColor}`}>{uptime.uptime_percent}%</div>
            <div className="text-xs text-muted-foreground mt-1">{uptime.total_checks} checks · {uptime.healthy_checks} up · {uptime.sick_checks} degraded · {uptime.dead_checks} down</div>
          </CardContent>
        </Card>
      )}

      <Card className="mb-4">
        <CardHeader>
          <CardTitle>Response Time</CardTitle>
        </CardHeader>
        <CardContent>
          <UptimeChart monitorId={monitor.id} />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Recent Checks</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Time</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Code</TableHead>
                <TableHead>Response</TableHead>
                <TableHead>Error</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {checks.map(c => (
                <TableRow key={c.id}>
                  <TableCell className="text-xs text-muted-foreground">{new Date(c.checked_at + 'Z').toLocaleString()}</TableCell>
                  <TableCell><StatusBadge status={c.status} /></TableCell>
                  <TableCell className="text-muted-foreground text-xs">{c.status_code ?? '—'}</TableCell>
                  <TableCell className="text-muted-foreground">{c.response_time_ms != null ? `${c.response_time_ms}ms` : '—'}</TableCell>
                  <TableCell className="text-red text-xs max-w-[200px] truncate">{c.error || '—'}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
