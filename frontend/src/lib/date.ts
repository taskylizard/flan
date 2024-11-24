export function formatDate(dateString: string) {
  const date = new Date(dateString)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (days === 0) {
    return (
      `Today at ${
        date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}`
    )
  }
  else if (days === 1) {
    return (
      `Yesterday at ${
        date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}`
    )
  }
  else if (days < 7) {
    return date.toLocaleDateString(undefined, { weekday: 'long' })
  }
  else {
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  }
}
