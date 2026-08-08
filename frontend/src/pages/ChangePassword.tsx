import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useChangePassword, clearToken } from '../api'
import { Card, CardContent } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

export default function ChangePassword() {
  const [username, setUsername] = useState('admin')
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [confirmError, setConfirmError] = useState('')
  const ch = useChangePassword()
  const navigate = useNavigate()

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setConfirmError('')
    if (newPassword !== confirm) {
      setConfirmError('Passwords do not match')
      return
    }
    if (newPassword.length < 1) {
      setConfirmError('Password cannot be empty')
      return
    }
    ch.mutate({ username, current_password: currentPassword, new_password: newPassword }, {
      onSuccess: () => navigate('/'),
    })
  }

  function handleLogout() {
    clearToken()
    navigate('/login')
  }

  return (
    <div className="flex justify-center items-center min-h-screen bg-background">
      <form onSubmit={handleSubmit}>
        <Card className="w-[400px]">
          <CardContent className="flex flex-col gap-4">
            <h2 className="text-xl font-bold">🔒 Change Password</h2>
            <p className="text-sm text-muted-foreground">
              You must change your password before continuing.
            </p>

            {ch.isError && <p className="text-destructive text-sm">{ch.error?.message || 'Error'}</p>}
            {confirmError && <p className="text-destructive text-sm">{confirmError}</p>}

            <div className="flex flex-col gap-1.5">
              <Label htmlFor="username">Username</Label>
              <Input id="username" value={username} onChange={e => setUsername(e.target.value)} />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="currentPassword">Current Password</Label>
              <Input id="currentPassword" type="password" value={currentPassword} onChange={e => setCurrentPassword(e.target.value)} />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="newPassword">New Password</Label>
              <Input id="newPassword" type="password" value={newPassword} onChange={e => setNewPassword(e.target.value)} />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="confirm">Confirm New Password</Label>
              <Input id="confirm" type="password" value={confirm} onChange={e => setConfirm(e.target.value)} />
            </div>

            <div className="flex gap-2">
              <Button type="submit" disabled={ch.isPending} className="flex-1">
                {ch.isPending ? '...' : 'Change Password'}
              </Button>
              <Button type="button" variant="ghost" onClick={handleLogout}>
                Logout
              </Button>
            </div>
          </CardContent>
        </Card>
      </form>
    </div>
  )
}
