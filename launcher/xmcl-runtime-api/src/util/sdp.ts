export function createOfferLauncherUrl(description: string) {
  return `zenith-launcher://peer/offer/${description}`
}

export function createAnswerLauncherUrl(description: string) {
  return `zenith-launcher://peer/answer/${description}`
}

export function createOfferAppUrl(description: string, inviter: string) {
  return `zenith-launcher://peer/offer/${description}`
}

export function createAnswerAppUrl(description: string, inviter: string) {
  return `zenith-launcher://peer/answer/${description}`
}
