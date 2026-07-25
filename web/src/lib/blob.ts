/** Build an object URL from a Uint8Array. Casts around TS 5.7+'s stricter
 *  `Uint8Array<ArrayBufferLike>` vs DOM `BlobPart` typing (runtime-correct). */
export function bytesToObjectUrl(bytes: Uint8Array, type: string): string {
  return URL.createObjectURL(new Blob([bytes as unknown as BlobPart], { type }));
}
