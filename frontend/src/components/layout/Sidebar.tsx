import { Link, useLocation } from '@tanstack/react-router'
import { motion } from 'framer-motion'
import {
  LayoutDashboard,
  Database,
  Disc3,
  Zap,
  Layers,
  Settings,
} from 'lucide-react'

const navItems = [
  { path: '/', label: 'Dashboard', icon: LayoutDashboard },
  { path: '/usb', label: 'USB Devices', icon: Database },
  { path: '/iso', label: 'ISO Library', icon: Disc3 },
  { path: '/flash', label: 'Flash Queue', icon: Zap },
  { path: '/batch', label: 'Batch Flash', icon: Layers },
  { path: '/settings', label: 'Settings', icon: Settings },
]

export function Sidebar() {
  const location = useLocation()

  return (
    <aside className="w-64 border-r border-border bg-card glass">
      <div className="p-6">
        <h1 className="text-xl font-bold tracking-tight text-primary">
          USB Station
        </h1>
        <p className="text-xs text-muted-foreground mt-1">Bootable USB Manager</p>
      </div>
      <nav className="px-3 space-y-1">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path
          return (
            <Link
              key={item.path}
              to={item.path}
              className="relative flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors"
            >
              {isActive && (
                <motion.div
                  layoutId="sidebar-active"
                  className="absolute inset-0 bg-primary/10 rounded-lg"
                  transition={{ type: 'spring', stiffness: 380, damping: 30 }}
                />
              )}
              <item.icon className="w-5 h-5 relative z-10" />
              <span className="relative z-10">{item.label}</span>
            </Link>
          )
        })}
      </nav>
    </aside>
  )
}
