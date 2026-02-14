import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type Provider, type RecordingTrigger } from '@/bindings'

interface SaveAppConfigParams {
  activeProvider?: Provider | null
  recordingTrigger?: RecordingTrigger
  postProcessEnabled?: boolean
  postProcessModel?: string
  postProcessPrompt?: string
  minSpeechDurationMs?: number
}

export function useSaveAppConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveAppConfigParams): Promise<void> => {
      const result = await commands.saveAppConfig(
        params.activeProvider ?? null,
        params.recordingTrigger ?? null,
        params.postProcessEnabled ?? null,
        params.postProcessModel ?? null,
        params.postProcessPrompt ?? null,
        params.minSpeechDurationMs ?? null
      )
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      // Invalidate the appConfig query to refetch fresh data
      queryClient.invalidateQueries({ queryKey: ['appConfig'] })
    },
  })
}
