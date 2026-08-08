import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMonitors, useCreateMonitor, useUpdateMonitor, useDeleteMonitor, MonitorInput, Monitor } from '../api'
import StatusBadge from '../components/StatusBadge'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

const defaults: MonitorInput = { name: '', type: 'http', url: '', interval_secs: 60, timeout_secs: 10, method: 'GET' }

export default function Monitors() {
  const { data: monitors = [] } = useMonitors()
  const create = useCreateMonitor()
  const update = useUpdateMonitor()
  const del = useDeleteMonitor()
  const [show, setShow] = useState(false)
  const [editId, setEditId] = useState<string | null>(null)
  const [form, setForm] = useState<MonitorInput>({ ...defaults })
  const nav = useNavigate()

  function openCreate() { setEditId(null); setForm({ ...defaults }); setShow(true) }
  function openEdit(m: Monitor) {
    setEditId(m.id)
    setForm({ name: m.name, type: m.type, url: m.url, interval_secs: m.interval_secs, timeout_secs: m.timeout_secs, method: m.method })
    setShow(true)
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (editId) {
      update.mutate({ id: editId, data: form }, { onSuccess: () => { setShow(false); setEditId(null) } })
    } else {
      create.mutate(form, { onSuccess: () => { setShow(false); setForm({ ...defaults }) } })
    }
  }

  const isPending = create.isPending || update.isPending

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-xl font-bold">Monitors</h1>
        <Button onClick={openCreate}>+ Add Monitor</Button>
      </div>

      <Dialog open={show} onOpenChange={setShow}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editId ? 'Edit Monitor' : 'New Monitor'}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit} className="flex flex-col gap-3">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="name">Name</Label>
              <Input id="name" required value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="type">Type</Label>
              <Select value={form.type} onValueChange={val => val && setForm({ ...form, type: val })}>
                <SelectTrigger id="type"><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="http">HTTP(S)</SelectItem>
                  <SelectItem value="tcp">TCP</SelectItem>
                  <SelectItem value="icmp">ICMP Ping</SelectItem>
                  <SelectItem value="dns">DNS</SelectItem>
                  <SelectItem value="script">Script</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="url">URL</Label>
              <Input id="url" required placeholder="https://example.com" value={form.url} onChange={e => setForm({ ...form, url: e.target.value })} />
            </div>
            <div className="flex gap-3">
              <div className="flex-1 flex flex-col gap-1.5">
                <Label htmlFor="interval">Interval (s)</Label>
                <Input id="interval" type="number" value={form.interval_secs} onChange={e => setForm({ ...form, interval_secs: +e.target.value })} />
              </div>
              <div className="flex-1 flex flex-col gap-1.5">
                <Label htmlFor="timeout">Timeout (s)</Label>
                <Input id="timeout" type="number" value={form.timeout_secs} onChange={e => setForm({ ...form, timeout_secs: +e.target.value })} />
              </div>
            </div>
            {form.type === 'http' && (
              <div className="flex flex-col gap-1.5">
                <Label htmlFor="method">HTTP Method</Label>
                <Select value={form.method} onValueChange={val => val && setForm({ ...form, method: val })}>
                  <SelectTrigger id="method"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="GET">GET</SelectItem>
                    <SelectItem value="HEAD">HEAD</SelectItem>
                    <SelectItem value="POST">POST</SelectItem>
                    <SelectItem value="PUT">PUT</SelectItem>
                    <SelectItem value="PATCH">PATCH</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}
            <div className="flex gap-2 mt-2">
              <Button type="submit" disabled={isPending}>{isPending ? '...' : editId ? 'Save' : 'Create'}</Button>
              <Button type="button" variant="outline" onClick={() => setShow(false)}>Cancel</Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      <div className="rounded-lg border overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>URL</TableHead>
              <TableHead>Interval</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {monitors.map(m => (
              <TableRow key={m.id}>
                <TableCell>
                  <a onClick={e => { e.preventDefault(); nav(`/monitors/${m.id}`) }} className="text-primary cursor-pointer hover:underline">{m.name}</a>
                </TableCell>
                <TableCell className="text-muted-foreground">{m.type}</TableCell>
                <TableCell className="max-w-[300px] truncate text-muted-foreground">{m.url}</TableCell>
                <TableCell className="text-muted-foreground">{m.interval_secs}s</TableCell>
                <TableCell><StatusBadge status={m.current_status} /></TableCell>
                <TableCell>
                  <div className="flex gap-1">
                    <Button variant="outline" size="xs" onClick={() => openEdit(m)}>Edit</Button>
                    <Button variant="destructive" size="xs" onClick={() => { if (confirm('Delete?')) del.mutate(m.id) }}>Delete</Button>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
