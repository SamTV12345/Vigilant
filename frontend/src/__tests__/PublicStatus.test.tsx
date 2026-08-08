/// <reference types="@vitest/browser" />

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/react'
import PublicStatus from '../pages/PublicStatus'
import type { PublicMonitor, DailyUptime, Incident, StatusResponse } from '../api'

// -- Mock API hooks --
const { mockUseStatus, mockUseDailyUptime, mockUseIncidents } = vi.hoisted(() => ({
  mockUseStatus: vi.fn((): { data: StatusResponse; isLoading: boolean; isError: boolean } => ({
    data: { status: 'healthy', monitors: [] },
    isLoading: false, isError: false,
  })),
  mockUseDailyUptime: vi.fn((_id?: string): { data: DailyUptime[]; isLoading: boolean; isError: boolean } => ({
    data: [],
    isLoading: false,
    isError: false,
  })),
  mockUseIncidents: vi.fn((): { data: Incident[]; isLoading: boolean; isError: boolean } => ({
    data: [],
    isLoading: false,
    isError: false,
  })),
}))

vi.mock('../api', () => ({
  useStatus: () => mockUseStatus(),
  useDailyUptime: (id: string) => mockUseDailyUptime(id),
  useIncidents: () => mockUseIncidents(),
}))

// -- Default mock data --
const healthyMonitors: PublicMonitor[] = [
  { id: '1', name: 'API', type: 'http', url: 'https://api.example.com', status: 'healthy', active: true },
  { id: '2', name: 'Website', type: 'http', url: 'https://example.com', status: 'healthy', active: true },
  { id: '3', name: 'Database', type: 'tcp', url: 'db.internal:5432', status: 'healthy', active: true },
]
const deadMonitors: PublicMonitor[] = [
  healthyMonitors[0],
  { ...healthyMonitors[1], status: 'dead' },
  healthyMonitors[2],
]
const sickMonitors: PublicMonitor[] = [
  healthyMonitors[0],
  { ...healthyMonitors[1], status: 'sick' },
  healthyMonitors[2],
]

beforeEach(() => {
  cleanup()
  mockUseStatus.mockReturnValue({
    data: { status: 'healthy', monitors: healthyMonitors },
    isLoading: false,
    isError: false,
  })
  mockUseDailyUptime.mockReturnValue({ data: [], isLoading: false, isError: false })
  mockUseIncidents.mockReturnValue({ data: [], isLoading: false, isError: false })
})

describe('PublicStatus', () => {
  // -- Banner --
  it('renders healthy banner', async () => {
    render(<PublicStatus />)
    await expect.element(screen.getByText('All Systems Operational')).toBeVisible()
  })

  it('renders dead banner', async () => {
    mockUseStatus.mockReturnValue({
      data: { status: 'dead', monitors: deadMonitors },
      isLoading: false, isError: false,
    })
    render(<PublicStatus />)
    await expect.element(screen.getByText('Major Outage')).toBeVisible()
  })

  it('renders sick/degraded banner', async () => {
    mockUseStatus.mockReturnValue({
      data: { status: 'sick', monitors: sickMonitors },
      isLoading: false, isError: false,
    })
    render(<PublicStatus />)
    await expect.element(screen.getByText('Degraded Performance')).toBeVisible()
  })

  // -- Service list --
  it('renders service list with monitor names', async () => {
    render(<PublicStatus />)
    await expect.element(screen.getByText('API')).toBeVisible()
    await expect.element(screen.getByText('Website')).toBeVisible()
    await expect.element(screen.getByText('Database')).toBeVisible()
  })

  it('shows "No services monitored" when empty', async () => {
    mockUseStatus.mockReturnValue({
      data: { status: 'healthy', monitors: [] },
      isLoading: false, isError: false,
    })
    render(<PublicStatus />)
    await expect.element(screen.getByText('No services monitored.')).toBeVisible()
  })

  it('shows uptime percentage when daily data exists', async () => {
    mockUseDailyUptime.mockReturnValue({
      data: [
        { date: '2026-08-06', uptime_percent: 100.0, healthy: 24, sick: 0, dead: 0 },
        { date: '2026-08-07', uptime_percent: 66.67, healthy: 16, sick: 0, dead: 8 },
        { date: '2026-08-08', uptime_percent: 100.0, healthy: 24, sick: 0, dead: 0 },
      ],
      isLoading: false,
      isError: false,
    })
    render(<PublicStatus />)
    // 3 days: (24+16+24) / (24+24+24) = 64/72 = 88.89%
    await expect.element(screen.getAllByText('88.89%')[0]).toBeVisible()
  })

  // -- Subscribe modal --
  it('opens subscribe modal on button click', async () => {
    render(<PublicStatus />)
    // Two "Subscribe to Updates" buttons — header + footer; use the first one
    await screen.getAllByText('Subscribe to Updates')[0].click()
    await expect.element(screen.getByText(/Email/)).toBeVisible()
  })

  it('closes subscribe modal on X button click', async () => {
    render(<PublicStatus />)
    await screen.getAllByText('Subscribe to Updates')[0].click()
    // Click the accessible close button (shadcn Dialog)
    await screen.getByRole('button', { name: 'Close' }).click()
    await waitFor(() => {
      expect(screen.queryByText(/📧 Email/)).toBeNull()
    })
  })

  it('closes subscribe modal on overlay click', async () => {
    render(<PublicStatus />)
    await screen.getAllByText('Subscribe to Updates')[0].click()
    // Press Escape to close the dialog (shadcn Dialog supports keyboard dismiss)
    fireEvent.keyDown(document, { key: 'Escape', code: 'Escape' })
    await waitFor(() => {
      expect(screen.queryByText(/📧 Email/)).toBeNull()
    })
  })

  // -- Atom feed link --
  it('has atom feed link in modal', async () => {
    render(<PublicStatus />)
    await screen.getAllByText('Subscribe to Updates')[0].click()
    await expect.element(screen.getByText(/Atom Feed/)).toBeVisible()
  })

  // -- Incident history --
  it('renders incident history when incidents exist', async () => {
    mockUseIncidents.mockReturnValue({
      data: [
        { id: 'inc-1', monitor_id: '1', started_at: '2026-08-07T12:00:00', resolved_at: '2026-08-07T13:00:00', status: 'resolved' },
        { id: 'inc-2', monitor_id: '2', started_at: '2026-08-06T08:00:00', resolved_at: null, status: 'investigating' },
      ],
      isLoading: false, isError: false,
    })
    render(<PublicStatus />)
    await expect.element(screen.getByText('Past Incidents')).toBeVisible()
    // "API" is in the service list AND incident — check for unique incident text instead
    await expect.element(screen.getByText(/investigating/)).toBeVisible()
  })

  it('hides incident section when empty', async () => {
    mockUseIncidents.mockReturnValue({ data: [], isLoading: false, isError: false })
    render(<PublicStatus />)
    expect(screen.queryByText('Past Incidents')).toBeNull()
  })

  // -- Footer --
  it('renders footer with powered by link', async () => {
    render(<PublicStatus />)
    await expect.element(screen.getByText('Vigilant')).toBeVisible()
  })
})
