import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useSettings, useUpsertSetting, useUsers, useCreateUser, useDeleteUser, useChangePassword, clearToken } from '../api'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'

export default function Settings() {
  const { data: settings = [] } = useSettings()
  const upsert = useUpsertSetting()
  const [key, setKey] = useState('')
  const [value, setValue] = useState('')

  // User management
  const { data: users = [] } = useUsers()
  const createUser = useCreateUser()
  const deleteUser = useDeleteUser()
  const navigate = useNavigate()

  const [newUsername, setNewUsername] = useState('')
  const [newUserPassword, setNewUserPassword] = useState('')

  // Change own password
  const changePw = useChangePassword()
  const [cpwUsername, setCpwUsername] = useState('')
  const [cpwCurrent, setCpwCurrent] = useState('')
  const [cpwNew, setCpwNew] = useState('')
  const [cpwConfirm, setCpwConfirm] = useState('')
  const [cpwError, setCpwError] = useState('')

  function handleSave(e: React.FormEvent) {
    e.preventDefault()
    upsert.mutate({ key, value }, { onSuccess: () => { setKey(''); setValue('') } })
  }

  function handleCreateUser(e: React.FormEvent) {
    e.preventDefault()
    createUser.mutate({ username: newUsername, password: newUserPassword }, {
      onSuccess: () => { setNewUsername(''); setNewUserPassword('') }
    })
  }

  function handleChangePassword(e: React.FormEvent) {
    e.preventDefault()
    setCpwError('')
    if (cpwNew !== cpwConfirm) {
      setCpwError('Passwords do not match')
      return
    }
    changePw.mutate({ username: cpwUsername, current_password: cpwCurrent, new_password: cpwNew }, {
      onSuccess: () => {
        setCpwUsername(''); setCpwCurrent(''); setCpwNew(''); setCpwConfirm('')
      }
    })
  }

  function handleLogout() {
    clearToken()
    navigate('/login')
  }

  return (
    <div>
      <h1 className="text-xl font-bold mb-4">Settings</h1>

      {/* Change Password */}
      <Card className="mb-4">
        <CardHeader>
          <CardTitle>Change Password</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleChangePassword} className="flex flex-col gap-3">
            {(changePw.isError || cpwError) && (
              <p className="text-destructive text-sm">{cpwError || changePw.error?.message}</p>
            )}
            {changePw.isSuccess && <p className="text-green-400 text-sm">Password changed successfully!</p>}
            <div className="flex gap-3">
              <div className="flex-1 flex flex-col gap-1.5">
                <Label htmlFor="cpw-username">Username</Label>
                <Input id="cpw-username" value={cpwUsername} onChange={e => setCpwUsername(e.target.value)} />
              </div>
              <div className="flex-1 flex flex-col gap-1.5">
                <Label htmlFor="cpw-current">Current Password</Label>
                <Input id="cpw-current" type="password" value={cpwCurrent} onChange={e => setCpwCurrent(e.target.value)} />
              </div>
              <div className="flex-1 flex flex-col gap-1.5">
                <Label htmlFor="cpw-new">New Password</Label>
                <Input id="cpw-new" type="password" value={cpwNew} onChange={e => setCpwNew(e.target.value)} />
              </div>
              <div className="flex-1 flex flex-col gap-1.5">
                <Label htmlFor="cpw-confirm">Confirm</Label>
                <Input id="cpw-confirm" type="password" value={cpwConfirm} onChange={e => setCpwConfirm(e.target.value)} />
              </div>
              <div className="flex items-end gap-2">
                <Button type="submit" disabled={changePw.isPending}>Save</Button>
                <Button type="button" variant="ghost" onClick={handleLogout}>Logout</Button>
              </div>
            </div>
          </form>
        </CardContent>
      </Card>

      {/* User Management */}
      <Card className="mb-4">
        <CardHeader>
          <CardTitle>Users</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleCreateUser} className="flex items-end gap-3 mb-4">
            <div className="flex-1 flex flex-col gap-1.5">
              <Label htmlFor="new-username">Username</Label>
              <Input id="new-username" placeholder="username" value={newUsername} onChange={e => setNewUsername(e.target.value)} />
            </div>
            <div className="flex-1 flex flex-col gap-1.5">
              <Label htmlFor="new-password">Password</Label>
              <Input id="new-password" type="password" placeholder="password" value={newUserPassword} onChange={e => setNewUserPassword(e.target.value)} />
            </div>
            <Button type="submit" disabled={createUser.isPending}>Add User</Button>
          </form>
          {createUser.isError && <p className="text-destructive text-sm mb-3">{createUser.error?.message}</p>}

          <div className="rounded-lg border overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Username</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="w-[80px]">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {users.map(u => (
                  <TableRow key={u.id}>
                    <TableCell>{u.username}</TableCell>
                    <TableCell>
                      {u.must_change_password !== 0
                        ? <Badge className="bg-yellow-600 text-white hover:bg-yellow-700 border-0">Must change</Badge>
                        : <Badge className="bg-green-600 text-white hover:bg-green-700 border-0">Active</Badge>
                      }
                    </TableCell>
                    <TableCell className="text-muted-foreground text-sm">{u.created_at || '—'}</TableCell>
                    <TableCell>
                      <Button
                        size="sm"
                        variant="destructive"
                        disabled={users.length <= 1 || deleteUser.isPending}
                        onClick={() => deleteUser.mutate(u.id)}
                      >
                        Delete
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>

      {/* Settings KV */}
      <Card className="mb-4">
        <CardHeader>
          <CardTitle>Add / Update Setting</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSave} className="flex items-end gap-3">
            <div className="flex-1 flex flex-col gap-1.5">
              <Label htmlFor="key">Key</Label>
              <Input id="key" placeholder="e.g. poll_interval" value={key} onChange={e => setKey(e.target.value)} />
            </div>
            <div className="flex-[2] flex flex-col gap-1.5">
              <Label htmlFor="value">Value</Label>
              <Input id="value" placeholder="value" value={value} onChange={e => setValue(e.target.value)} />
            </div>
            <Button type="submit" disabled={upsert.isPending}>Save</Button>
          </form>
        </CardContent>
      </Card>

      <div className="rounded-lg border overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Key</TableHead>
              <TableHead>Value</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {settings.map(s => (
              <TableRow key={s.key}>
                <TableCell><code className="text-primary text-xs">{s.key}</code></TableCell>
                <TableCell>{s.value}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
