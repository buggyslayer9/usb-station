import { Zap } from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent } from '../ui/card'

export function FlashQueuePage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Flash Queue</h2>
          <p className="text-muted-foreground mt-1">Active and recent flash operations</p>
        </div>
      </div>

      <Card className="glass">
        <CardHeader>
          <CardTitle>Active Jobs</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8 text-muted-foreground">
            <Zap className="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p>No active flash jobs</p>
            <p className="text-sm mt-1">
              Select a USB device and ISO from the dashboard to start flashing
            </p>
          </div>
        </CardContent>
      </Card>

      <Card className="glass">
        <CardHeader>
          <CardTitle>History</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm">Flash history will appear here</p>
        </CardContent>
      </Card>
    </div>
  )
}
