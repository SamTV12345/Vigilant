import { useState } from 'react'
import { useNotifications, useCreateNotification, useUpdateNotification, useDeleteNotification, Notification } from '../api'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'

const CHANNELS = ['slack', 'email', 'telegram', 'webhook', 'twilio', 'pushover', 'gotify', 'zulip', 'matrix', 'webex']

export default function Notifications() {
  const { data: notifications = [] } = useNotifications()
  const create = useCreateNotification()
  const update = useUpdateNotification()
  const del = useDeleteNotification()
  const [show, setShow] = useState(false)
  const [editId, setEditId] = useState<string | null>(null)
  const [showConfig, setShowConfig] = useState<string | null>(null) // notification id to expand config
  const [name, setName] = useState('')
  const [type, setType] = useState('slack')
  const [active, setActive] = useState(true)
  const [config, setConfig] = useState('{}')

  function openCreate() {
    setEditId(null); setName(''); setType('slack'); setActive(true); setConfig('{}'); setShow(true)
  }

  function openEdit(n: Notification) {
    setEditId(n.id)
    setName(n.name)
    setType(n.type)
    setActive(n.active)
    try {
      setConfig(JSON.stringify(JSON.parse(typeof n.config === 'string' ? n.config : JSON.stringify(n.config)), null, 2))
    } catch {
      setConfig(typeof n.config === 'string' ? n.config : JSON.stringify(n.config))
    }
    setShow(true)
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    let cfg: unknown
    try { cfg = JSON.parse(config) } catch { alert('Invalid JSON'); return }
    if (editId) {
      update.mutate({ id: editId, data: { name, type, config: cfg as any, active } }, {
        onSuccess: () => { setShow(false); setEditId(null) }
      })
    } else {
      create.mutate({ name, type, config: cfg as any }, {
        onSuccess: () => { setShow(false); setName(''); setConfig('{}') }
      })
    }
  }

  const isPending = create.isPending || update.isPending

  const toggleActive = (n: Notification) => {
    update.mutate({ id: n.id, data: { active: !n.active } })
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-xl font-bold">Notifications</h1>
        <Button onClick={openCreate}>+ Add Channel</Button>
      </div>

      <Dialog open={show} onOpenChange={setShow}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editId ? 'Edit Channel' : 'New Notification Channel'}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit} className="flex flex-col gap-3">
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
            {editId && (
              <div className="flex items-center justify-between">
                <Label htmlFor="active-toggle">Active</Label>
                <Switch id="active-toggle" checked={active} onCheckedChange={setActive} />
              </div>
            )}
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="config">Config (JSON)</Label>
              <Textarea id="config" value={config} onChange={e => setConfig(e.target.value)}
                className="font-mono text-sm h-24 resize-y" />
            </div>
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
              <TableHead>Active</TableHead>
              <TableHead>Config</TableHead>
              <TableHead>Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {notifications.map(n => (
              <>
                <TableRow key={n.id}>
                  <TableCell>{n.name}</TableCell>
                  <TableCell><Badge variant="outline">{n.type}</Badge></TableCell>
                  <TableCell>
                    <button
                      onClick={() => toggleActive(n)}
                      className="cursor-pointer"
                      title="Toggle active"
                    >
                      {n.active ? '✅' : '❌'}
                    </button>
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="xs"
                      onClick={() => setShowConfig(showConfig === n.id ? null : n.id)}
                    >
                      {showConfig === n.id ? 'Hide' : 'View'}
                    </Button>
                  </TableCell>
                  <TableCell>
                    <div className="flex gap-1">
                      <Button variant="outline" size="xs" onClick={() => openEdit(n)}>Edit</Button>
                      <Button variant="destructive" size="xs" onClick={() => { if (confirm('Delete?')) del.mutate(n.id) }}>Delete</Button>
                    </div>
                  </TableCell>
                </TableRow>
                {showConfig === n.id && (
                  <TableRow key={`${n.id}-cfg`}>
                    <TableCell colSpan={5} className="bg-muted/30">
                      <pre className="text-xs font-mono whitespace-pre-wrap break-all">
                        {(() => {
                          try {
                            const parsed = typeof n.config === 'string' ? JSON.parse(n.config) : n.config
                            return JSON.stringify(parsed, null, 2)
                          } catch {
                            return typeof n.config === 'string' ? n.config : JSON.stringify(n.config)
                          }
                        })()}
                      </pre>
                    </TableCell>
                  </TableRow>
                )}
              </>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
