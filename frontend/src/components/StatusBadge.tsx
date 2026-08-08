import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

const variants: Record<string, string> = {
  healthy: 'bg-green-600 text-white hover:bg-green-700',
  sick: 'bg-yellow-600 text-white hover:bg-yellow-700',
  partial: 'bg-partial text-white hover:bg-partial/90',
  dead: 'bg-destructive text-white hover:bg-destructive/90',
}

export default function StatusBadge({ status }: { status: string }) {
  return (
    <Badge className={cn('border-0', variants[status] || 'bg-border text-muted-foreground')}>
      {status}
    </Badge>
  )
}
