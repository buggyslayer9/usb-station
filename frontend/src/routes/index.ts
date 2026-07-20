import { createRootRoute, createRoute } from '@tanstack/react-router'
import { AppShell } from '../components/layout/AppShell'
import { DashboardPage } from '../components/pages/DashboardPage'
import { UsbPage } from '../components/pages/UsbPage'
import { IsoPage } from '../components/pages/IsoPage'
import { FlashQueuePage } from '../components/pages/FlashQueuePage'
import { BatchPage } from '../components/pages/BatchPage'
import { SettingsPage } from '../components/pages/SettingsPage'

const rootRoute = createRootRoute({
  component: AppShell,
})

const dashboardRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: DashboardPage,
})

const usbRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/usb',
  component: UsbPage,
})

const isoRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/iso',
  component: IsoPage,
})

const flashQueueRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/flash',
  component: FlashQueuePage,
})

const batchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/batch',
  component: BatchPage,
})

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: SettingsPage,
})

const routeTree = rootRoute.addChildren([
  dashboardRoute,
  usbRoute,
  isoRoute,
  flashQueueRoute,
  batchRoute,
  settingsRoute,
])

export { routeTree }
