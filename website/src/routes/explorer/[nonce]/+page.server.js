// Hardcoded dummy tests fields for now
import messageData from '$lib/database/explorerData.json'

export async function load({ fetch }) {
    return { messageData };
}