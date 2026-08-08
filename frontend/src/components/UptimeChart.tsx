import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'
import { useChecks } from '../api'

export default function UptimeChart({ monitorId }: { monitorId: string }) {
  const { data: checks = [] } = useChecks(monitorId, 200)

  const data = checks
    .reverse()
    .map(c => ({
      time: new Date(c.checked_at + 'Z').toLocaleTimeString(),
      ms: c.response_time_ms || null,
      status: c.status === 'healthy' ? 1 : c.status === 'sick' ? 0.5 : 0,
    }))

  return (
    <div style={{ height: 200 }}>
      <ResponsiveContainer>
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
          <XAxis dataKey="time" stroke="#94a3b8" fontSize={11} interval="preserveStartEnd" />
          <YAxis stroke="#94a3b8" fontSize={11} label={{ value: 'ms', position: 'insideTopLeft', fill: '#94a3b8' }} />
          <Tooltip
            contentStyle={{ background: '#1e293b', border: '1px solid #334155', borderRadius: 6 }}
            labelStyle={{ color: '#f1f5f9' }}
          />
          <Line type="monotone" dataKey="ms" stroke="#3b82f6" dot={false} strokeWidth={2} />
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
