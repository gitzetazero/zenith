// zenith: minisign verification (https://jedisct1.github.io/minisign/) for registry files whose project has a `key`.
// Legacy ("Ed", over the file) and prehashed ("ED", over BLAKE2b-512 of the file) signatures are accepted, like Horizon does.
import { blake2b512 } from './blake2b'
import { createPublicKey, verify } from 'crypto'

const SPKI_ED25519_PREFIX = Buffer.from('302a300506032b6570032100', 'hex')

export interface MinisignKey { keyId: Buffer; publicKey: Buffer }

export function parsePublicKey(base64: string): MinisignKey {
  const raw = Buffer.from(base64.trim(), 'base64')
  if (raw.length !== 42 || raw.toString('latin1', 0, 2) !== 'Ed') throw new Error('Not a minisign public key.')
  return { keyId: raw.subarray(2, 10), publicKey: raw.subarray(10) }
}

/** The signature line of a `.minisig` text (its second line). */
export function parseSignature(text: string): { algorithm: 'Ed' | 'ED'; keyId: Buffer; signature: Buffer } {
  const line = text.split(/\r?\n/)[1]?.trim()
  const raw = line ? Buffer.from(line, 'base64') : Buffer.alloc(0)
  const algorithm = raw.toString('latin1', 0, 2)
  if (raw.length !== 74 || (algorithm !== 'Ed' && algorithm !== 'ED')) throw new Error('Not a minisign signature.')
  return { algorithm, keyId: raw.subarray(2, 10), signature: raw.subarray(10) }
}

/** True when `signatureText` is a valid signature of `data` by the key; never throws on a malformed signature. */
export function verifyMinisign(publicKeyBase64: string, signatureText: string, data: Uint8Array): boolean {
  try {
    const key = parsePublicKey(publicKeyBase64)
    const sig = parseSignature(signatureText)
    if (!sig.keyId.equals(key.keyId)) return false
    const message = sig.algorithm === 'ED' ? Buffer.from(blake2b512(data)) : Buffer.from(data)
    const pub = createPublicKey({ key: Buffer.concat([SPKI_ED25519_PREFIX, key.publicKey]), format: 'der', type: 'spki' })
    return verify(null, message, pub, sig.signature)
  } catch {
    return false
  }
}
