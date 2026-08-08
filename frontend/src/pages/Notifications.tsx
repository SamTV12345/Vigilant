import { useState } from 'react'
import { useNotifications, useCreateNotification, useDeleteNotification } from '../api'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'

const CHANNELS = ['slack', 'email', 'telegram', 'webhook', 'twilio', 'pushover', 'gotify', 'zulip', 'matrix', 'webex']

export default function Notifications() {
  const { data: notifications = [] } = useNotifications()
  const create = useCreateNotification()
  const del = useDeleteNotification()
  const [show, setShow] = useState(false)
  const [name, setName] = useState('')
  const [type, setType] = useState('slack')
  const [config, setConfig] = useState('{}')

  function handleCreate(e: React.FormEvent) {
    e.preventDefault()
    let cfg: unknown
    try { cfg = JSON.parse(config) } catch { alert('Invalid JSON'); return }
    create.mutate({ name, type, config: cfg as any }, {
      onSuccess: () => { setShow(false); setName(''); setConfig('{}') },
    })
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-xl font-bold">Notifications</h1>
        <Button onClick={() => setShow(true)}>+ Add Channel</Button>
      </div>

      <Dialog open={show} onOpenChange={setShow}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>New Notification Channel</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleCreate} className="flex flex-col gap-3">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="name">Name</Label>
              <Input id="name" required value={name} onChange={e => setName(e.target.value)} />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="type">Type</Label>
              <Select value={type} onValueChange={val => val && setType(val)}>
                <SelectTrigger id="type"><SelectValue /></SelectTrigger>
                <SelectContent>
                  {CHANNELS.map(c => <SelectItem key={c} value={c}>{c}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="config">Config (JSON)</Label>
              <Textarea id="config" value={config} onChange={e => setConfig(e.target.value)}
                className="font-mono text-sm h-24 resize-y" />
            </div>
            <div className="flex gap-2 mt-2">
              <Button type="submit" disabled={create.isPending}>{create.isPending ? '...' : 'Create'}</Button>
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
              <TableHead>Active</TableHead>
              <TableHead>Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {notifications.map(n => (
              <TableRow key={n.id}>
                <TableCell>{n.name}</TableCell>
                <TableCell><Badge variant="outline">{n.type}</Badge></TableCell>
                <TableCell>{n.active ? '✅' : '❌'}</TableCell>
                <TableCell>
                  <Button variant="destructive" size="xs" onClick={() => { if (confirm('Delete?')) del.mutate(n.id) }}>Delete</Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
