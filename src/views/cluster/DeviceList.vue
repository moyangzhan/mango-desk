<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { t } from '@/locales'
import { formatTime } from '@/utils/functions'

const props = defineProps<{
  devices: Device[]
  loading?: boolean
}>()

const emit = defineEmits<{
  view: [device: Device]
  reject: [device: Device]
  unreject: [device: Device]
  resetPairingStatus: [device: Device]
  requestPairing: [device: Device]
  acceptPairing: [device: Device]
  rejectPairing: [device: Device]
  checkDevices: []
}>()

type DeviceFilter = 'all' | 'pending_in' | 'pending_out' | 'paired' | 'rejected' | 'blocked' | 'none'
const deviceFilter = ref<DeviceFilter>('pending_in')
const initialCheckDone = ref(false)
const checking = ref(false)

function handleCheckDevices() {
  checking.value = true
  emit('checkDevices')
  setTimeout(() => {
    checking.value = false
  }, 3000)
}

watch(() => props.devices, (devices) => {
  if (!initialCheckDone.value && devices.length > 0) {
    initialCheckDone.value = true
    if (deviceFilter.value === 'pending_in' && devices.filter(d => d.pairing_status === 'pending_in').length === 0)
      deviceFilter.value = 'all'
  }
}, { immediate: true })

const filteredDevices = computed(() => {
  if (deviceFilter.value === 'all')
    return props.devices
  return props.devices.filter(d => d.pairing_status === deviceFilter.value)
})

const deviceCounts = computed(() => {
  const counts = { pending_in: 0, pending_out: 0, paired: 0, rejected: 0, blocked: 0, none: 0 }
  for (const d of props.devices)
    counts[d.pairing_status as keyof typeof counts]++
  return { all: props.devices.length, ...counts }
})

function getStatusDescription(status: string): string {
  const descMap: Record<string, string> = {
    pending_in: t('cluster.pairing.pendingInDesc'),
    pending_out: t('cluster.pairing.pendingOutDesc'),
    paired: t('cluster.pairing.pairedDesc'),
    rejected: t('cluster.pairing.rejectedDesc'),
    blocked: t('cluster.pairing.blockedDesc'),
  }
  return descMap[status] || ''
}
</script>

<template>
  <NCard size="small">
    <template #header>
      <div class="flex items-center gap-2">
        <span>{{ t('cluster.devices') }}</span>
        <AppTag v-if="deviceCounts.pending_in > 0" type="warning" size="small">
          {{ deviceCounts.pending_in }} {{ t('cluster.pairing.pending') }}
        </AppTag>
      </div>
    </template>
    <template #header-extra>
      <div class="flex items-center gap-2">
        <NButton size="small" ghost :loading="checking" @click="handleCheckDevices">
          {{ checking ? t('cluster.checking') : t('cluster.checkDevices') }}
        </NButton>
        <slot name="header-extra" />
      </div>
    </template>

    <!-- Filter Buttons -->
    <div class="flex flex-wrap gap-2 mb-3">
      <NButton
        :type="deviceFilter === 'all' ? 'primary' : 'default'" :ghost="deviceFilter === 'all'" size="small"
        @click="deviceFilter = 'all'"
      >
        {{ t('cluster.pairing.all') }} ({{ deviceCounts.all }})
      </NButton>
      <NButton
        v-if="deviceCounts.pending_in > 0" type="warning" ghost size="small"
        @click="deviceFilter = 'pending_in'"
      >
        {{ t('cluster.pairing.pendingIn') }} ({{ deviceCounts.pending_in }})
      </NButton>
      <NButton v-if="deviceCounts.pending_out > 0" type="info" ghost size="small" @click="deviceFilter = 'pending_out'">
        {{ t('cluster.pairing.pendingOut') }} ({{ deviceCounts.pending_out }})
      </NButton>
      <NButton
        :type="deviceFilter === 'paired' ? 'success' : 'default'" :ghost="deviceFilter === 'paired'" size="small"
        @click="deviceFilter = 'paired'"
      >
        {{ t('cluster.pairing.paired') }} ({{ deviceCounts.paired }})
      </NButton>
      <NButton
        v-if="deviceCounts.none > 0" :ghost="deviceFilter === 'none'" size="small"
        @click="deviceFilter = 'none'"
      >
        {{ t('cluster.pairing.none') }} ({{ deviceCounts.none }})
      </NButton>
      <NButton v-if="deviceCounts.rejected > 0" type="error" ghost size="small" @click="deviceFilter = 'rejected'">
        {{ t('cluster.pairing.rejected') }} ({{ deviceCounts.rejected }})
      </NButton>
      <NButton v-if="deviceCounts.blocked > 0" type="default" ghost size="small" @click="deviceFilter = 'blocked'">
        {{ t('cluster.pairing.blocked') }} ({{ deviceCounts.blocked }})
      </NButton>
    </div>

    <!-- Device List -->
    <NSpin :show="loading">
      <div v-if="filteredDevices.length > 0" class="space-y-2">
        <div
          v-for="device in filteredDevices" :key="device.device_id" class="p-3 border rounded-lg cursor-pointer"
          :class="{
            'border-red-300 bg-red-50 dark:bg-red-900/20': device.pairing_status === 'rejected',
            'border-gray-300 bg-gray-50 dark:bg-gray-800': device.pairing_status === 'blocked',
            'border-green-300 bg-green-50 dark:bg-green-900/20': device.pairing_status === 'paired',
            'border-yellow-300 bg-yellow-50 dark:bg-yellow-900/20': device.pairing_status === 'pending_in',
            'border-blue-300 bg-blue-50 dark:bg-blue-900/20': device.pairing_status === 'pending_out',
          }" @click="emit('view', device)"
        >
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-3 min-w-0 flex-1">
              <div
                class="w-3 h-3 rounded-full shrink-0" :class="{
                  'bg-green-500': device.online_status === 'online',
                  'bg-gray-400': device.online_status === 'offline',
                  'bg-gray-300': device.online_status === 'unknown',
                }"
              />
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="font-medium text-link truncate">
                    {{ device.name }}
                  </span>
                  <AppTag v-if="device.pairing_status === 'paired'" type="primary" size="small">
                    {{ t('cluster.pairing.paired') }}
                  </AppTag>
                  <AppTag v-else-if="device.pairing_status === 'pending_in'" type="warning" size="small">
                    {{ t('cluster.pairing.pendingIn') }}
                  </AppTag>
                  <AppTag v-else-if="device.pairing_status === 'pending_out'" type="info" size="small">
                    {{ t('cluster.pairing.pendingOut') }}
                  </AppTag>
                  <AppTag v-else-if="device.pairing_status === 'rejected'" type="error" size="small">
                    {{ t('cluster.pairing.rejected') }}
                  </AppTag>
                  <AppTag v-else-if="device.pairing_status === 'blocked'" type="default" size="small">
                    {{ t('cluster.pairing.blocked') }}
                  </AppTag>
                </div>
                <div class="text-xs text-gray-500 truncate flex items-center gap-1.5">
                  <AppTag v-if="device.online_status === 'online'" type="success" size="tiny">
                    {{ t('cluster.onlineStatusOnline') }}
                  </AppTag>
                  <AppTag v-else-if="device.online_status === 'offline'" type="default" size="tiny">
                    {{ t('cluster.onlineStatusOffline') }}
                  </AppTag>
                  <AppTag v-else type="default" size="tiny">
                    {{ t('cluster.onlineStatusUnknown') }}
                  </AppTag>
                  <span>{{ device.ip_address }}:{{ device.port }}</span>
                  <span>·</span>
                  <AppTag v-if="device.discovery_method === 'manual'" type="default" size="tiny">
                    {{ t('cluster.discoveryMethodManual') }}
                  </AppTag>
                  <AppTag v-else type="info" size="tiny">
                    {{ t('cluster.discoveryMethodMdns') }}
                  </AppTag>
                  <span>·</span>
                  <span>{{ formatTime(device.last_seen, t) }}</span>
                </div>
                <div v-if="device.pairing_status !== 'none'" class="text-xs text-gray-400 truncate mt-0.5">
                  {{ getStatusDescription(device.pairing_status) }}
                </div>
              </div>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <!-- Action Buttons for pending_in -->
              <template v-if="device.pairing_status === 'pending_in'">
                <NTooltip :disabled="device.online_status === 'online'">
                  <template #trigger>
                    <NButton size="small" ghost :disabled="device.online_status !== 'online'" @click.stop="emit('rejectPairing', device)">
                      {{ t('common.reject') }}
                    </NButton>
                  </template>
                  {{ t('cluster.deviceOfflineTooltip') }}
                </NTooltip>
                <NTooltip :disabled="device.online_status === 'online'">
                  <template #trigger>
                    <NButton size="small" type="primary" ghost :disabled="device.online_status !== 'online'" @click.stop="emit('acceptPairing', device)">
                      {{ t('common.accept') }}
                    </NButton>
                  </template>
                  {{ t('cluster.deviceOfflineTooltip') }}
                </NTooltip>
              </template>

              <!-- Action Buttons for pending_out -->
              <template v-if="device.pairing_status === 'pending_out'">
                <NTooltip :disabled="device.online_status === 'online'">
                  <template #trigger>
                    <NButton size="small" type="primary" ghost :disabled="device.online_status !== 'online'" @click.stop="emit('requestPairing', device)">
                      {{ t('cluster.retryRequestPairing') }}
                    </NButton>
                  </template>
                  {{ t('cluster.deviceOfflineTooltip') }}
                </NTooltip>
              </template>

              <!-- Action Buttons for none -->
              <template v-if="device.pairing_status === 'none'">
                <NTooltip :disabled="device.online_status === 'online'">
                  <template #trigger>
                    <NButton size="small" type="primary" ghost :disabled="device.online_status !== 'online'" @click.stop="emit('requestPairing', device)">
                      {{ t('cluster.requestPairing') }}
                    </NButton>
                  </template>
                  {{ t('cluster.deviceOfflineTooltip') }}
                </NTooltip>
              </template>

              <!-- Action Buttons for rejected (I rejected them, can reset) -->
              <template v-if="device.pairing_status === 'rejected'">
                <NTooltip :disabled="device.online_status === 'online'">
                  <template #trigger>
                    <NButton size="small" ghost :disabled="device.online_status !== 'online'" @click.stop="emit('unreject', device)">
                      {{ t('cluster.resetPairingStatus') }}
                    </NButton>
                  </template>
                  {{ t('cluster.deviceOfflineTooltip') }}
                </NTooltip>
              </template>

              <!-- No buttons for blocked (they blocked me, cannot do anything) -->

              <!-- Action Buttons for paired -->
              <template v-if="device.pairing_status === 'paired'">
                <NTooltip :disabled="device.online_status === 'online'">
                  <template #trigger>
                    <NButton size="small" ghost :disabled="device.online_status !== 'online'" @click.stop="emit('resetPairingStatus', device)">
                      {{ t('cluster.unpair') }}
                    </NButton>
                  </template>
                  {{ t('cluster.deviceOfflineTooltip') }}
                </NTooltip>
              </template>
            </div>
          </div>
        </div>
      </div>
      <div v-else class="text-center py-8 text-gray-400">
        {{ t('common.noData') }}
      </div>
    </NSpin>
  </NCard>
</template>
