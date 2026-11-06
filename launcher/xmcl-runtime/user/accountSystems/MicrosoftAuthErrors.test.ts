import { expect, test } from 'vitest'
import { classifyMicrosoftAppSetupProblem } from './MicrosoftAuthErrors'
import { MICROSOFT_XBOX_LAUNCHER_SCOPES } from './MicrosoftOAuthClient'

test('a client id Microsoft does not know is told apart from a network failure', () => {
  const error = Object.assign(new Error('unauthorized_client: The client does not exist or is not enabled for consumers.'), { error: 'unauthorized_client' })
  expect(classifyMicrosoftAppSetupProblem(error)).toBe('APP_NOT_FOUND')
  expect(classifyMicrosoftAppSetupProblem(new Error('AADSTS700016: Application with identifier was not found'))).toBe('APP_NOT_FOUND')
})

test('an unregistered redirect uri and a refused scope are named', () => {
  expect(classifyMicrosoftAppSetupProblem(new Error("invalid_request: The provided value for the input parameter 'redirect_uri' is not valid. The expected value is a URI which matches a redirect URI registered for this client application."))).toBe('REDIRECT_URI_NOT_REGISTERED')
  expect(classifyMicrosoftAppSetupProblem(new Error('AADSTS70011: The provided request must include a scope that is not valid'))).toBe('INVALID_SCOPE')
})

test('ordinary failures are not blamed on the app registration', () => {
  expect(classifyMicrosoftAppSetupProblem(new Error('network_error'))).toBeUndefined()
  expect(classifyMicrosoftAppSetupProblem(new Error('Microsoft authorization window was closed.'))).toBeUndefined()
  expect(classifyMicrosoftAppSetupProblem(undefined)).toBeUndefined()
})

test('the sign-in asks only for the Xbox sign-in scope and a refresh token', () => {
  expect([...MICROSOFT_XBOX_LAUNCHER_SCOPES]).toEqual(['XboxLive.signin', 'offline_access'])
})
