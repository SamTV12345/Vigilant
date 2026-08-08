import { useEffect, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { io, Socket } from 'socket.io-client'

// -- Auth --
const TOKEN_KEY = 'vigilant_token'

export function getToken(): string | null { return localStorage.getItem(TOKEN_KEY) }
export function setToken(t: string) { localStorage.setItem(TOKEN_KEY, t) }
export function clearToken() { localStorage.removeItem(TOKEN_KEY) }
export function useAuth(): boolean { return !!getToken() }

async function authFetch(url: string, opts: RequestInit = {}): Promise<Response> {
  const token = getToken()
  const headers: Record<string, string> = { 'Content-Type': 'application/json', ...(opts.headers as Record<string, string> || {}) }
  if (token) headers['Authorization'] = `Bearer ${token}`
  return fetch(url, { ...opts, headers })
}

// -- Types --
export interface Monitor {
  id: string; name: string; type: string; url: string;
  interval_secs: number; timeout_secs: number;
  method?: string; headers?: string; body?: string; script?: string;
  active: boolean; current_status: string;
  created_at: string; updated_at: string;
}
export interface Check {
  id: number; monitor_id: string; status: string;
  response_time_ms?: number; status_code?: number; error?: string;
  checked_at: string;
}
export interface UptimeData {
  monitor_id: string; period_hours: number; uptime_percent: number;
  total_checks: number; healthy_checks: number; sick_checks: number; dead_checks: number;
}
export interface Notification {
  id: string; name: string; type: string; config: string;
  reminders_only: boolean; active: boolean;
}
export interface Setting { key: string; value: string }
export interface Announcement { id: string; title: string; text: string; created_at: string }
export interface PublicMonitor { id: string; name: string; type: string; url: string; status: string; active: boolean }
export interface StatusResponse { status: string; monitors: PublicMonitor[] }
export interface DailyUptime { date: string; uptime_percent: number; healthy: number; sick: number; dead: number }
export interface Incident { id: string; monitor_id: string; started_at: string; resolved_at: string | null; status: string }

export interface MonitorInput {
  name: string; type: string; url: string;
  interval_secs: number; timeout_secs: number;
  method?: string;
}

// -- Auth mutation --
export function useLogin() {
  return useMutation({
    mutationFn: async ({ username, password }: { username: string; password: string }) => {
      const r = await fetch('/api/auth/login', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      })
      if (!r.ok) throw new Error('Invalid credentials')
      const data = await r.json()
      setToken(data.token)
      return data as { token: string; must_change_password: boolean }
    },
  })
}

export function useChangePassword() {
  return useMutation({
    mutationFn: async ({ username, current_password, new_password }: { username: string; current_password: string; new_password: string }) => {
      const r = await fetch('/api/auth/change-password', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, current_password, new_password }),
      })
      if (!r.ok) {
        const e = await r.json().catch(() => ({ error: 'Failed' }))
        throw new Error(e.error || 'Failed to change password')
      }
      return r.json()
    },
  })
}

// -- Monitor queries / mutations --
export function useMonitors() {
  return useQuery({
    queryKey: ['monitors'],
    queryFn: async (): Promise<Monitor[]> => {
      const r = await authFetch('/api/admin/monitors')
      if (!r.ok) throw new Error('Failed')
      return r.json()
    },
  })
}

export function useMonitor(id: string) {
  const { data: monitors = [] } = useMonitors()
  return { data: monitors.find(m => m.id === id), isLoading: false, isError: false }
}

export function useChecks(monitorId: string, limit = 50) {
  return useQuery({
    queryKey: ['checks', monitorId, limit],
    queryFn: async (): Promise<Check[]> => {
      const r = await fetch(`/api/monitors/${monitorId}/checks?limit=${limit}`)
      return r.json()
    },
    refetchInterval: 10000,
  })
}

export function useUptime(monitorId: string, period = 24) {
  return useQuery({
    queryKey: ['uptime', monitorId, period],
    queryFn: async (): Promise<UptimeData> => {
      const r = await fetch(`/api/monitors/${monitorId}/uptime?period=${period}`)
      return r.json()
    },
  })
}

export function useDailyUptime(monitorId: string, days = 90) {
  return useQuery({
    queryKey: ['dailyUptime', monitorId, days],
    queryFn: async (): Promise<DailyUptime[]> => {
      const r = await fetch(`/api/monitors/${monitorId}/uptime/daily?days=${days}`)
      return r.json()
    },
  })
}

export function useIncidents(limit = 50) {
  return useQuery({
    queryKey: ['incidents', limit],
    queryFn: async (): Promise<Incident[]> => {
      const r = await fetch(`/api/incidents?limit=${limit}`)
      return r.json()
    },
    refetchInterval: 30000,
  })
}

export function useStatus() {
  return useQuery({
    queryKey: ['status'],
    queryFn: async (): Promise<StatusResponse> => {
      const r = await fetch('/api/status')
      return r.json()
    },
    refetchInterval: 10000,
  })
}

export function useCreateMonitor() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (data: MonitorInput) => {
      const r = await authFetch('/api/admin/monitors', { method: 'POST', body: JSON.stringify(data) })
      return r.json()
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['monitors'] }),
  })
}

export function useUpdateMonitor() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Partial<MonitorInput> }) => {
      const r = await authFetch(`/api/admin/monitors/${id}`, { method: 'PUT', body: JSON.stringify(data) })
      return r.json()
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['monitors'] }),
  })
}

export function useDeleteMonitor() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => { await authFetch(`/api/admin/monitors/${id}`, { method: 'DELETE' }) },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['monitors'] }),
  })
}

// -- Notification queries / mutations --
export function useNotifications() {
  return useQuery({
    queryKey: ['notifications'],
    queryFn: async (): Promise<Notification[]> => {
      const r = await authFetch('/api/admin/notifications')
      return r.json()
    },
  })
}

export function useCreateNotification() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (data: Partial<Notification>) => {
      const r = await authFetch('/api/admin/notifications', { method: 'POST', body: JSON.stringify(data) })
      return r.json()
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notifications'] }),
  })
}

export function useDeleteNotification() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => { await authFetch(`/api/admin/notifications/${id}`, { method: 'DELETE' }) },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notifications'] }),
  })
}

// -- Settings --
export function useSettings() {
  return useQuery({
    queryKey: ['settings'],
    queryFn: async (): Promise<Setting[]> => {
      const r = await authFetch('/api/admin/settings')
      return r.json()
    },
  })
}

export function useUpsertSetting() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ key, value }: { key: string; value: string }) => {
      await authFetch('/api/admin/settings', { method: 'PUT', body: JSON.stringify({ key, value }) })
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['settings'] }),
  })
}

// -- Users --
export interface UserInfo {
  id: string; username: string; must_change_password: number; created_at: string;
}

export function useUsers() {
  return useQuery({
    queryKey: ['users'],
    queryFn: async (): Promise<UserInfo[]> => {
      const r = await authFetch('/api/admin/users')
      if (!r.ok) throw new Error('Failed')
      return r.json()
    },
  })
}

export function useCreateUser() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ username, password }: { username: string; password: string }) => {
      const r = await authFetch('/api/admin/users', { method: 'POST', body: JSON.stringify({ username, password }) })
      if (!r.ok) {
        const e = await r.json().catch(() => ({ error: 'Failed' }))
        throw new Error(e.error || 'Failed to create user')
      }
      return r.json()
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['users'] }),
  })
}

export function useDeleteUser() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      const r = await authFetch(`/api/admin/users/${id}`, { method: 'DELETE' })
      if (!r.ok) {
        const e = await r.json().catch(() => ({ error: 'Failed' }))
        throw new Error(e.error || 'Failed to delete user')
      }
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['users'] }),
  })
}

// -- Socket.IO --
export function useSocket(): Socket | null {
  const [s, setS] = useState<Socket | null>(null)
  useEffect(() => {
    const sock = io('/', { transports: ['websocket', 'polling'] })
    setS(sock)
    return () => { sock.disconnect() }
  }, [])
  return s
}
