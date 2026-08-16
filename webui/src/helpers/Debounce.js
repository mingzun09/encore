/**
 * Wraps an async save function with debounce + flush semantics so callers
 * can trigger frequent updates (e.g. toggle changes) while only persisting
 * once things settle, and still force an immediate write when needed
 * (route leave, unmount, before a destructive action like reboot).
 *
 * @param {() => Promise<any>} saveFn - function that performs the actual save
 * @param {number} [delay=500] - debounce delay in milliseconds
 * @returns {{ trigger: () => void, flush: () => Promise<void>, cancel: () => void }}
 */
export function createDebouncedSave(saveFn, delay = 500) {
  let timeoutId = null
  let pendingPromise = null

  function trigger() {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }

    timeoutId = setTimeout(() => {
      timeoutId = null
      pendingPromise = Promise.resolve(saveFn()).catch((error) => {
        console.error('Debounced save failed:', error)
      })
    }, delay)
  }

  async function flush() {
    if (timeoutId) {
      clearTimeout(timeoutId)
      timeoutId = null
      pendingPromise = Promise.resolve(saveFn()).catch((error) => {
        console.error('Debounced save flush failed:', error)
      })
    }

    if (pendingPromise) {
      await pendingPromise
    }
  }

  function cancel() {
    if (timeoutId) {
      clearTimeout(timeoutId)
      timeoutId = null
    }
  }

  return { trigger, flush, cancel }
}
