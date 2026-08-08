import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useLogin } from '../api'
import { Card, CardContent } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

export default function Login() {
  const [username, setUsername] = useState('admin')
  const [password, setPassword] = useState('')
  const login = useLogin()
  const navigate = useNavigate()

  return (
    <div className="flex justify-center items-center min-h-screen bg-background">
      <form
        onSubmit={e => {
          e.preventDefault()
          login.mutate({ username, password }, {
            onSuccess: (data) => {
              if (data.must_change_password) {
                navigate('/change-password')
              } else {
                navigate('/')
              }
            }
          })
        }}
      >
        <Card className="w-[360px]">
          <CardContent className="flex flex-col gap-4">
            <h2 className="text-xl font-bold">⚡ Vigilant Login</h2>
            {login.isError && <p className="text-destructive text-sm">Invalid credentials</p>}
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="username">Username</Label>
              <Input id="username" value={username} onChange={e => setUsername(e.target.value)} />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="password">Password</Label>
              <Input id="password" type="password" value={password} onChange={e => setPassword(e.target.value)} />
            </div>
            <Button type="submit" disabled={login.isPending}>
              {login.isPending ? '...' : 'Login'}
            </Button>
            <p className="text-xs text-muted-foreground">Default: admin / admin</p>
          </CardContent>
        </Card>
      </form>
    </div>
  )
}
