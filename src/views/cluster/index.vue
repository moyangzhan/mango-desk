<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { EditOutlined, PlusOutlined } from '@vicons/material'
import { t } from '@/locales'
import { useAppStore } from '@/stores/app'
import DeviceList from './DeviceList.vue'
import AppTag from '@/components/AppTag.vue'

const router = useRouter()
const appStore = useAppStore()

const devices = ref<Device[]>([])
const pairingRequests = ref<PairingRequest[]>([])
const selectedPairingRequestIds = ref<number[]>([])
const loading = ref(false)
const showAddDeviceModal = ref(false)
const showEditLocalDeviceModal = ref(false)
const showDeviceDetailModal = ref(false)
const selectedDevice = ref<Device | null>(null)

// Cluster setting for local device info
const clusterSetting = ref<ClusterSetting>({
  enabled: false,
  port: 7890,
  device_name: '',
  allow_to_be_discovered: true,
  auto_request_pairing: false,
  auto_accept_pairing: false,
  online_check_interval: 30,
})

// Edit local device form
const editPort = ref(7890)
const editDeviceName = ref('')

const localDeviceName = computed(() => {
  return clusterSetting.value.device_name || t('cluster.localDevice')
})

// Port error from global state
const portError = computed(() => {
  const error = appStore.getClusterPortError
  if (error) {
    return t('cluster.portBindError', { port: error.port })
  }
  return null
})

function dismissPortError() {
  appStore.clearClusterPortError()
}

// Add device form
const newDeviceName = ref('')
const newDeviceIp = ref('')
const newDevicePort = ref(7890)
const addingDevice = ref(false)

// Cluster enabled status
const clusterEnabled = ref(false)

async function loadData() {
  loading.value = true
  try {
    // Load cluster setting to check if enabled and get local device info
    const setting = await invoke<ClusterSetting>('load_cluster_setting')
    clusterSetting.value = setting
    clusterEnabled.value = setting?.enabled ?? false

    await Promise.all([loadDevices(), loadPairingRequests()])
  }
  catch (e) {
    console.error('Failed to load data:', e)
  }
  finally {
    loading.value = false
  }
}

async function loadDevices() {
  try {
    const result = await invoke<Device[]>('load_devices')
    devices.value = result || []
  }
  catch (e) {
    console.error('Failed to load devices:', e)
  }
}

async function loadPairingRequests() {
  try {
    const result = await invoke<PairingRequest[]>('load_pairing_requests')
    pairingRequests.value = result || []
  }
  catch (e) {
    console.error('Failed to load pairing requests:', e)
  }
}

async function clearAllPairingRequests() {
  const confirmed = await new Promise<boolean>((resolve) => {
    window.$dialog.warning({
      title: t('cluster.clearPairingRequests'),
      content: t('cluster.clearPairingRequestsConfirm'),
      positiveText: t('common.confirm'),
      negativeText: t('common.cancel'),
      onPositiveClick: () => resolve(true),
      onNegativeClick: () => resolve(false),
      onClose: () => resolve(false),
    })
  })

  if (!confirmed)
    return

  try {
    await invoke('clear_pairing_requests')
    window.$message.success(t('common.operationSuccess'))
    selectedPairingRequestIds.value = []
    await loadPairingRequests()
  }
  catch (e) {
    console.error('Failed to clear pairing requests:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

async function deleteSelectedPairingRequests() {
  if (selectedPairingRequestIds.value.length === 0)
    return

  try {
    await invoke('delete_pairing_requests', { ids: selectedPairingRequestIds.value })
    window.$message.success(t('common.operationSuccess'))
    selectedPairingRequestIds.value = []
    await loadPairingRequests()
  }
  catch (e) {
    console.error('Failed to delete pairing requests:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

function handlePairingRequestCheck(rowKeys: (string | number)[]) {
  selectedPairingRequestIds.value = rowKeys as number[]
}

function handleDeviceView(device: Device) {
  selectedDevice.value = device
  showDeviceDetailModal.value = true
}

async function handleDeviceReject(device: Device) {
  // Show confirmation dialog with warning about what rejecting does
  const confirmed = await new Promise<boolean>((resolve) => {
    window.$dialog.warning({
      title: t('cluster.rejectDevice'),
      content: t('cluster.rejectDeviceConfirm'),
      positiveText: t('common.confirm'),
      negativeText: t('common.cancel'),
      onPositiveClick: () => resolve(true),
      onNegativeClick: () => resolve(false),
      onClose: () => resolve(false),
    })
  })

  if (!confirmed)
    return

  try {
    await invoke('reject_device', { deviceId: device.device_id })
    window.$message.success(t('common.operationSuccess'))
    await loadDevices()
  }
  catch (e) {
    console.error('Failed to reject device:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

async function handleUnrejectDevice(device: Device) {
  try {
    const updatedDevice = await invoke<Device>('unreject_device', { deviceId: device.device_id })
    if (updatedDevice.online_status === 'online')
      window.$message.success(t('cluster.unrejectSuccess'))
    else
      window.$message.warning(t('cluster.unrejectButOffline'))

    await loadDevices()
  }
  catch (e) {
    console.error('Failed to unreject device:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

async function handleResetPairingStatus(device: Device) {
  try {
    const updatedDevice = await invoke<Device>('reset_pairing_status', { deviceId: device.device_id })
    if (updatedDevice.online_status === 'online')
      window.$message.success(t('cluster.resetPairingStatusSuccess'))
    else
      window.$message.warning(t('cluster.resetPairingStatusSuccessButOffline'))

    await loadDevices()
  }
  catch (e) {
    console.error('Failed to reset pairing status:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

async function handleRequestPairing(device: Device) {
  try {
    await invoke('send_pairing_request', {
      deviceId: device.device_id,
      deviceName: device.name,
      ip: device.ip_address,
      port: device.port,
    })
    window.$message.success(t('common.operationSuccess'))
    await loadDevices()
  }
  catch (e) {
    console.error('Failed to send pairing request:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

async function handleAcceptPairing(device: Device) {
  try {
    // Get pending incoming request for this device
    const requests = await invoke<PairingRequest[]>('load_pending_pairing_requests')
    const request = requests.find(r => r.device_id === device.device_id && r.direction === 'in')

    if (request) {
      await invoke('respond_pairing_request', { id: request.id, accept: true })
      window.$message.success(t('common.operationSuccess'))
      await loadDevices()
    }
    else {
      window.$message.warning(t('common.noData'))
    }
  }
  catch (e) {
    console.error('Failed to accept pairing:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

async function handleRejectPairing(device: Device) {
  try {
    // Get pending incoming request for this device
    const requests = await invoke<PairingRequest[]>('load_pending_pairing_requests')
    const request = requests.find(r => r.device_id === device.device_id && r.direction === 'in')

    if (request) {
      await invoke('respond_pairing_request', { id: request.id, accept: false })
      window.$message.success(t('common.operationSuccess'))
      await loadDevices()
    }
    else {
      window.$message.warning(t('common.noData'))
    }
  }
  catch (e) {
    console.error('Failed to reject pairing:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

async function handleCheckDevices() {
  try {
    await invoke('check_devices')
    await loadDevices()
    window.$message.success(t('common.operationSuccess'))
  }
  catch (e) {
    console.error('Failed to check devices:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)

  if (diffMins < 1)
    return t('common.just')
  if (diffMins < 60)
    return `${diffMins}${t('common.minutes')}${t('common.ago')}`

  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24)
    return `${diffHours}${t('common.hours')}${t('common.ago')}`

  return `${date.toLocaleDateString()} ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`
}

async function addDeviceManually() {
  if (!newDeviceName.value.trim() || !newDeviceIp.value.trim()) {
    window.$message.warning(t('cluster.invalidDeviceInfo'))
    return
  }

  addingDevice.value = true
  try {
    await invoke('add_device_manually', {
      name: newDeviceName.value.trim(),
      ipAddress: newDeviceIp.value.trim(),
      port: newDevicePort.value,
    })
    window.$message.success(t('common.addSuccess'))
    showAddDeviceModal.value = false
    newDeviceName.value = ''
    newDeviceIp.value = ''
    newDevicePort.value = 7890
    await loadDevices()
  }
  catch (e) {
    console.error('Failed to add device:', e)
    window.$message.error(t('common.operationFailed'))
  }
  finally {
    addingDevice.value = false
  }
}

function openEditLocalDeviceModal() {
  editPort.value = clusterSetting.value.port
  editDeviceName.value = clusterSetting.value.device_name
  showEditLocalDeviceModal.value = true
}

async function saveLocalDevice() {
  try {
    const updatedSetting = {
      ...clusterSetting.value,
      port: editPort.value,
      device_name: editDeviceName.value,
    }
    await invoke('update_cluster_setting', { setting: updatedSetting })
    clusterSetting.value = updatedSetting
    window.$message.success(t('common.saveSuccess'))
    showEditLocalDeviceModal.value = false
  }
  catch (e) {
    console.error('Failed to save local device:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

onMounted(() => {
  loadData()

  // Listen for pairing request handled events (covers all statuses)
  listen<PairingRequestHandledPayload>('pairing-request-handled', () => {
    loadDevices()
    loadPairingRequests()
  })

  // Listen for pairing request sent events (outgoing requests)
  listen('pairing-request-sent', () => {
    loadPairingRequests()
  })

  // Listen for pairing response events
  listen<PairingResponsePayload>('pairing-response-received', () => {
    loadDevices()
    loadPairingRequests()
  })

  // Listen for pairing reset events (remote device reset their pairing status)
  listen('pairing-reset-received', () => {
    loadDevices()
    loadPairingRequests()
  })

  // Listen for device discovered events
  listen<Device>('device-discovered', () => {
    loadDevices()
  })

  // Listen for device online/offline events
  listen<string>('device-online', (event) => {
    const deviceId = event.payload
    const device = devices.value.find(d => d.device_id === deviceId)
    if (device)
      device.online_status = 'online'
  })

  listen<string>('device-offline', (event) => {
    const deviceId = event.payload
    const device = devices.value.find(d => d.device_id === deviceId)
    if (device)
      device.online_status = 'offline'
  })

  // Listen for cluster enabled/disabled events from settings page
  listen<{ enabled: boolean }>('cluster-enabled-changed', (event) => {
    clusterEnabled.value = event.payload.enabled
    if (event.payload.enabled)
      loadData()
  })

  // Listen for device rejected events (refresh lists when user rejects a device)
  listen<string>('device-rejected', () => {
    loadDevices()
    loadPairingRequests()
  })
})

function goToSetting() {
  router.push({ name: 'Setting', query: { tab: 'cluster' } })
}

interface PairingResponsePayload {
  requester_id: string
  responder_id: string
  approved: boolean
}

interface PairingRequestHandledPayload {
  status: 'already_paired' | 'auto_rejected' | 'auto_accepted' | 'pending'
  device_id: string
  device_name: string
  ip_address: string
  port: number
}

// Pairing request table columns
const pairingRequestColumns = [
  {
    type: 'selection' as const,
  },
  {
    title: () => t('cluster.deviceName'),
    key: 'device_name',
    ellipsis: { tooltip: true },
  },
  {
    title: () => t('cluster.ipAddress'),
    key: 'ip_address',
    width: 150,
    render: (row: PairingRequest) => `${row.ip_address}:${row.port}`,
  },
  {
    title: () => t('cluster.pairing.direction'),
    key: 'direction',
    width: 80,
    render: (row: PairingRequest) => {
      const isOut = row.direction === 'out'
      return h(AppTag, { type: isOut ? 'info' : 'warning', size: 'small' }, () =>
        isOut ? t('cluster.pairing.outgoing') : t('cluster.pairing.incoming'),
      )
    },
  },
  {
    title: () => t('common.event'),
    key: 'event',
    width: 120,
    render: (row: PairingRequest) => {
      const typeMap: Record<string, 'warning' | 'success' | 'error' | 'info' | 'default'> = {
        pending: 'warning',
        accepted: 'success',
        rejected: 'error',
        expired: 'default',
        auto_rejected: 'default',
      }
      // 根据状态和方向确定事件类型
      const eventKeyMap: Record<string, string> = {
        'pending-out': 'requestSent',
        'pending-in': 'requestReceived',
        'accepted-out': 'requestAccepted',
        'accepted-in': 'requestAccepted',
        'rejected-out': 'requestRejected',
        'rejected-in': 'requestRejected',
        'expired-': 'requestExpired',
        'auto_rejected-': 'autoRejected',
      }
      const eventKey = eventKeyMap[`${row.status}-${row.direction}`] || row.status
      return h(AppTag, { type: typeMap[row.status] || 'default', size: 'small' }, () =>
        t(`cluster.pairingEvent.${eventKey}`),
      )
    },
  },
  {
    title: () => t('common.createTime'),
    key: 'create_time',
    width: 150,
    render: (row: PairingRequest) => formatTime(row.create_time),
  },
  {
    title: () => t('cluster.remark'),
    key: 'remark',
    ellipsis: { tooltip: true },
    render: (row: PairingRequest) => row.remark || '-',
  },
]
</script>

<template>
  <div class="p-6 h-full overflow-auto">
    <div class="max-w-4xl mx-auto space-y-4">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold">
          {{ t('menu.deviceManagement') }}
        </h2>
      </div>

      <!-- Disabled State -->
      <div v-if="!clusterEnabled" class="text-center py-12 text-gray-500 bg-gray-50 dark:bg-gray-800 rounded-lg">
        <div>{{ t('cluster.clusterDisabled') }}</div>
        <div class="mt-2 text-sm">
          {{ t('cluster.enableClusterFirst') }}
          <NButton text type="primary" class="text-link" @click="goToSetting">
            {{ t('cluster.goToSetting') }}
          </NButton>
        </div>
        <div class="mt-6 text-sm text-gray-400 dark:text-gray-500 text-center">
          <div class="inline-block text-left">
            <div class="font-medium mb-2">{{ t('cluster.howToUse.title') }}</div>
            <div>
              <div>1. {{ t('cluster.howToUse.step1') }}</div>
              <div>2. {{ t('cluster.howToUse.step2') }}</div>
              <div>3. {{ t('cluster.howToUse.step3') }}</div>
            </div>
          </div>
        </div>
      </div>

      <template v-else>
        <!-- Local Device Info -->
        <NCard size="small">
          <template #header>
            <div class="flex items-center gap-2">
              <span>{{ t('cluster.localDeviceInfo') }}</span>
            </div>
          </template>

          <!-- Port Error Alert -->
          <NAlert v-if="portError" type="error" closable class="mb-3" @close="dismissPortError">
            <template #header>
              {{ t('cluster.portBindErrorTitle') }}
            </template>
            {{ portError }}
          </NAlert>

          <div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-3 h-3 rounded-full" :class="portError ? 'bg-red-500' : 'bg-green-500'" />
                <div>
                  <div class="font-medium">
                    {{ localDeviceName }}
                  </div>
                  <div class="text-xs text-gray-500">
                    localhost:{{ clusterSetting.port }}
                  </div>
                </div>
              </div>
              <NButton size="small" ghost @click="openEditLocalDeviceModal">
                <template #icon>
                  <NIcon>
                    <EditOutlined />
                  </NIcon>
                </template>
                {{ t('common.edit') }}
              </NButton>
            </div>
          </div>
        </NCard>

        <!-- Devices -->
        <DeviceList
          :devices="devices"
          :loading="loading"
          @view="handleDeviceView"
          @reject="handleDeviceReject"
          @unreject="handleUnrejectDevice"
          @reset-pairing-status="handleResetPairingStatus"
          @request-pairing="handleRequestPairing"
          @accept-pairing="handleAcceptPairing"
          @reject-pairing="handleRejectPairing"
          @check-devices="handleCheckDevices"
        >
          <template #header-extra>
            <NButton size="small" ghost @click="showAddDeviceModal = true">
              <template #icon>
                <NIcon>
                  <PlusOutlined />
                </NIcon>
              </template>
              {{ t('cluster.addDevice') }}
            </NButton>
          </template>
        </DeviceList>

        <!-- Pairing Request Log -->
        <NCard size="small">
          <template #header>
            <div class="flex items-center justify-between">
              <span>{{ t('cluster.pairingRequests') }}</span>
              <div class="flex items-center gap-2">
                <NButton
                  v-if="selectedPairingRequestIds.length > 0"
                  size="small"
                  type="error"
                  ghost
                  @click="deleteSelectedPairingRequests"
                >
                  {{ t('common.deleteSelected') }} ({{ selectedPairingRequestIds.length }})
                </NButton>
                <NButton
                  v-if="pairingRequests.length > 0"
                  size="small"
                  ghost
                  @click="clearAllPairingRequests"
                >
                  {{ t('common.clearAll') }}
                </NButton>
              </div>
            </div>
          </template>
          <NDataTable
            :columns="pairingRequestColumns"
            :data="pairingRequests"
            :bordered="false"
            :row-key="(row: PairingRequest) => row.id"
            :checked-row-keys="selectedPairingRequestIds"
            size="small"
            max-height="300"
            @update:checked-row-keys="handlePairingRequestCheck"
          />
        </NCard>
      </template>

      <!-- Add Device Modal -->
      <NModal v-model:show="showAddDeviceModal" preset="card" :title="t('cluster.addDevice')" style="width: 400px">
        <NForm label-placement="left" label-width="80">
          <NFormItem :label="t('cluster.deviceName')">
            <NInput v-model:value="newDeviceName" :placeholder="t('cluster.deviceNamePlaceholder')" />
          </NFormItem>
          <NFormItem :label="t('cluster.ipAddress')">
            <NInput v-model:value="newDeviceIp" placeholder="192.168.1.100" />
          </NFormItem>
          <NFormItem :label="t('cluster.port')">
            <NInputNumber v-model:value="newDevicePort" :min="1" :max="65535" />
          </NFormItem>
        </NForm>
        <template #footer>
          <div class="flex justify-end gap-2">
            <NButton ghost @click="showAddDeviceModal = false">
              {{ t('common.cancel') }}
            </NButton>
            <NButton type="primary" ghost :loading="addingDevice" @click="addDeviceManually">
              {{ t('common.add') }}
            </NButton>
          </div>
        </template>
      </NModal>

      <!-- Edit Local Device Modal -->
      <NModal v-model:show="showEditLocalDeviceModal" preset="card" :title="t('cluster.editLocalDevice')" style="width: 400px">
        <NForm label-placement="left" label-width="80">
          <NFormItem :label="t('cluster.deviceName')">
            <NInput v-model:value="editDeviceName" :placeholder="t('cluster.deviceNamePlaceholder')" />
          </NFormItem>
          <NFormItem :label="t('cluster.port')">
            <NInputNumber v-model:value="editPort" :min="1024" :max="65535" />
          </NFormItem>
        </NForm>
        <template #footer>
          <div class="flex justify-end gap-2">
            <NButton ghost @click="showEditLocalDeviceModal = false">
              {{ t('common.cancel') }}
            </NButton>
            <NButton type="primary" ghost @click="saveLocalDevice">
              {{ t('common.save') }}
            </NButton>
          </div>
        </template>
      </NModal>

      <!-- Device Detail Modal -->
      <NModal v-model:show="showDeviceDetailModal" preset="card" :title="t('cluster.deviceInfo')" style="width: 450px">
        <template v-if="selectedDevice">
          <NDescriptions label-placement="left" :column="1" bordered size="small">
            <NDescriptionsItem :label="t('cluster.deviceName')">
              {{ selectedDevice.name || '-' }}
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.deviceId')">
              <NText code>
                {{ selectedDevice.device_id }}
              </NText>
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.ipAddress')">
              {{ selectedDevice.ip_address }}:{{ selectedDevice.port }}
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.status.online')">
              <AppTag :type="selectedDevice.online_status === 'online' ? 'success' : 'default'" size="small">
                {{ selectedDevice.online_status }}
              </AppTag>
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.pairingStatus')">
              <AppTag
                :type="selectedDevice.pairing_status === 'paired' ? 'success' : selectedDevice.pairing_status === 'pending_in' ? 'warning' : selectedDevice.pairing_status === 'rejected' ? 'error' : 'default'"
                size="small"
              >
                {{ t(`cluster.pairing.${selectedDevice.pairing_status}`) }}
              </AppTag>
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.version')">
              {{ selectedDevice.version || '-' }}
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.indexCount')">
              {{ selectedDevice.index_count ?? '-' }}
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.discoveryMethod')">
              {{ selectedDevice.discovery_method || '-' }}
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.firstDiscovered')">
              {{ selectedDevice.first_discovered ? formatTime(selectedDevice.first_discovered) : '-' }}
            </NDescriptionsItem>
            <NDescriptionsItem :label="t('cluster.lastSeen')">
              {{ selectedDevice.last_seen ? formatTime(selectedDevice.last_seen) : '-' }}
            </NDescriptionsItem>
          </NDescriptions>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <!-- paired: 取消互联 -->
            <NTooltip v-if="selectedDevice?.pairing_status === 'paired'" :disabled="selectedDevice?.online_status === 'online'">
              <template #trigger>
                <NButton
                  type="warning"
                  ghost
                  :disabled="selectedDevice?.online_status !== 'online'"
                  @click="handleResetPairingStatus(selectedDevice!); showDeviceDetailModal = false"
                >
                  {{ t('cluster.unpair') }}
                </NButton>
              </template>
              {{ t('cluster.deviceOfflineTooltip') }}
            </NTooltip>
            <!-- pending_in / pending_out: 拒绝 -->
            <NTooltip v-else-if="['pending_in', 'pending_out'].includes(selectedDevice?.pairing_status || '')" :disabled="selectedDevice?.online_status === 'online'">
              <template #trigger>
                <NButton
                  type="warning"
                  ghost
                  :disabled="selectedDevice?.online_status !== 'online'"
                  @click="handleDeviceReject(selectedDevice!); showDeviceDetailModal = false"
                >
                  {{ t('cluster.rejectDevice') }}
                </NButton>
              </template>
              {{ t('cluster.deviceOfflineTooltip') }}
            </NTooltip>
            <!-- rejected: 取消拒绝 -->
            <NTooltip v-else-if="selectedDevice?.pairing_status === 'rejected'" :disabled="selectedDevice?.online_status === 'online'">
              <template #trigger>
                <NButton
                  type="primary"
                  ghost
                  :disabled="selectedDevice?.online_status !== 'online'"
                  @click="handleUnrejectDevice(selectedDevice!); showDeviceDetailModal = false"
                >
                  {{ t('cluster.unrejectDevice') }}
                </NButton>
              </template>
              {{ t('cluster.deviceOfflineTooltip') }}
            </NTooltip>
            <!-- none / blocked: 无操作按钮，只显示关闭 -->
            <NButton ghost @click="showDeviceDetailModal = false">
              {{ t('common.close') }}
            </NButton>
          </div>
        </template>
      </NModal>
    </div>
  </div>
</template>
