/**
 * NDA project password-gate helpers.
 *
 * Uses Web Crypto (available in both Cloudflare Workers and Node) so it
 * runs identically in local dev and production — no Node-only APIs.
 */

const encoder = new TextEncoder();

function bufToHex(buf: ArrayBuffer): string {
    return Array.from(new Uint8Array(buf))
        .map((b) => b.toString(16).padStart(2, "0"))
        .join("");
}

/** SHA-256 hex digest of a plaintext string (used to hash NDA passwords). */
export async function sha256Hex(input: string): Promise<string> {
    const digest = await crypto.subtle.digest("SHA-256", encoder.encode(input));
    return bufToHex(digest);
}

async function hmacKey(secret: string) {
    return crypto.subtle.importKey(
        "raw",
        encoder.encode(secret),
        { name: "HMAC", hash: "SHA-256" },
        false,
        ["sign"],
    );
}

/** Deterministic per-slug unlock token, only computable with the server secret. */
export async function createUnlockToken(
    slug: string,
    secret: string,
): Promise<string> {
    const key = await hmacKey(secret);
    const sig = await crypto.subtle.sign("HMAC", key, encoder.encode(slug));
    return bufToHex(sig);
}

/** Verifies a cookie-supplied token matches the expected token for this slug. */
export async function verifyUnlockToken(
    slug: string,
    token: string | undefined,
    secret: string,
): Promise<boolean> {
    if (!token) return false;
    const expected = await createUnlockToken(slug, secret);
    if (expected.length !== token.length) return false;
    let diff = 0;
    for (let i = 0; i < expected.length; i++) {
        diff |= expected.charCodeAt(i) ^ token.charCodeAt(i);
    }
    return diff === 0;
}
