// Take a list of every deployed bridge from the relayer ?
// Hardcoded dummy tests fields for now
import messageData from '$lib/database/explorerData.json'

export async function load({ fetch }) {
    return { messageData };
}